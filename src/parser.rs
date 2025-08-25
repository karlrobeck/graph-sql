use anyhow::anyhow;
use async_graphql::dynamic::{
    Enum, EnumItem, Field, InputObject, InputValue, Object, Scalar, TypeRef,
};
use sqlparser::ast::{ColumnDef, ColumnOption, CreateTable, DataType, Expr, TableConstraint};
use tracing::{debug, instrument, warn};

use crate::{
    resolvers::{
        column_resolver, delete_resolver, foreign_key_resolver, insert_resolver, list_resolver,
        update_resolver, view_resolver,
    },
    traits::{
        GraphQLObjectOutput, ToGraphqlEnumExt, ToGraphqlFieldExt, ToGraphqlInputValueExt,
        ToGraphqlMutations, ToGraphqlNode, ToGraphqlObject, ToGraphqlQueries, ToGraphqlScalarExt,
        ToGraphqlTypeRefExt,
    },
    utils::{find_primary_key_column, strip_id_suffix},
};

pub struct TableDef {
    name: String,                //  name of the table
    columns: Vec<ColDef>,        // column definitions
    description: Option<String>, // table description
}

pub struct ColDef {
    table_name: String,          // name of the table that it belongs to
    name: String,                // name of the column
    data_type: ColDataType,      // data type of the column
    not_null: bool,              // has not null constraint
    is_primary: bool,            // is primary key
    description: Option<String>, // column description / comment
    relationship: Option<ForeignColDef>,
}

pub struct ForeignColDef {
    table: String, // The name of the parent table referenced by the foreign key.
    from: String,  // The name of the column in the child table (the table you're querying).
    to: String,    // The name of the column in the parent table that is referenced.
}

pub enum ColDataType {
    String,
    Integer,
    Float,
    Boolean,
}

impl TryFrom<String> for ColDataType {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "text" => Ok(Self::String),
            "integer" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            "boolean" => Ok(Self::Boolean),
            _ => Err(anyhow!("Unsupported DataType")),
        }
    }
}

impl From<ColDataType> for async_graphql::dynamic::Scalar {
    fn from(value: ColDataType) -> Self {
        match value {
            ColDataType::String => Scalar::new(TypeRef::STRING),
            ColDataType::Integer => Scalar::new(TypeRef::INT),
            ColDataType::Float => Scalar::new(TypeRef::FLOAT),
            ColDataType::Boolean => Scalar::new(TypeRef::BOOLEAN),
        }
    }
}

impl From<ColDef> for async_graphql::dynamic::TypeRef {
    fn from(value: ColDef) -> Self {
        let graphql_type = Scalar::from(value.data_type);

        if value.not_null {
            TypeRef::named_nn(graphql_type.type_name())
        } else {
            TypeRef::named(graphql_type.type_name())
        }
    }
}

impl From<ColDef> for async_graphql::dynamic::Field {
    fn from(value: ColDef) -> Self {
        if let Some(rel) = value.relationship {
            todo!("return foreign key resolver")
        }

        Field::new(value.name.clone(), TypeRef::from(value), move |_| {
            todo!("implement proper column resolver here")
        })
    }
}

impl From<ColDef>
    for (
        async_graphql::dynamic::InputValue, // insert input value
        async_graphql::dynamic::InputValue, // update input value
    )
{
    fn from(value: ColDef) -> Self {
        let graphql_type = Scalar::from(value.data_type);

        let type_ref = if value.not_null {
            TypeRef::named_nn(graphql_type.type_name())
        } else {
            TypeRef::named(graphql_type.type_name())
        };

        (
            InputValue::new(value.name.to_string(), type_ref),
            InputValue::new(
                value.name.to_string(),
                TypeRef::named(graphql_type.type_name()),
            ),
        )
    }
}

impl From<TableDef> for async_graphql::dynamic::Object {
    fn from(value: TableDef) -> Self {
        let mut table_node = Object::new(format!("{}_node", value.name));

        for col in value.columns {
            table_node = table_node.field(Field::from(col));
        }

        table_node
    }
}

// old code

// done
impl ToGraphqlScalarExt for ColumnDef {
    fn to_scalar(&self) -> async_graphql::Result<async_graphql::dynamic::Scalar> {
        let scalar = match &self.data_type {
            // Text types
            DataType::Text | DataType::Varchar(_) | DataType::Char(_) | DataType::String(_) => {
                Scalar::new(TypeRef::STRING)
            }

            // Integer types
            DataType::Int(_)
            | DataType::Integer(_)
            | DataType::TinyInt(_)
            | DataType::SmallInt(_)
            | DataType::MediumInt(_)
            | DataType::BigInt(_) => Scalar::new(TypeRef::INT),

            // Floating point types
            DataType::Real
            | DataType::Float(_)
            | DataType::DoublePrecision
            | DataType::Decimal(_)
            | DataType::Numeric(_) => Scalar::new(TypeRef::FLOAT),

            // Boolean type
            DataType::Boolean => Scalar::new(TypeRef::BOOLEAN),

            // Binary data - represent as String (base64 encoded)
            DataType::Blob(_) | DataType::Binary(_) | DataType::Varbinary(_) | DataType::Bytea => {
                warn!(
                    "Binary data type {:?} mapped to String (base64 encoded)",
                    self.data_type
                );
                Scalar::new(TypeRef::STRING)
            }

            // Date/Time types - represent as String (ISO 8601 format)
            DataType::Date
            | DataType::Time(_, _)
            | DataType::Timestamp(_, _)
            | DataType::Datetime(_) => {
                debug!(
                    "Date/time type {:?} mapped to String (ISO 8601 format)",
                    self.data_type
                );
                Scalar::new(TypeRef::STRING)
            }

            // JSON types - represent as String
            DataType::JSON => {
                debug!("JSON type mapped to String");
                Scalar::new(TypeRef::STRING)
            }

            // Custom types - validate and warn if unknown
            DataType::Custom(name, _) => {
                // Check for known SQLite type names that might be custom parsed
                let type_name = name.to_string().to_lowercase();
                match type_name.as_str() {
                    "integer" | "int" => Scalar::new(TypeRef::INT),
                    "real" | "float" | "double" => Scalar::new(TypeRef::FLOAT),
                    "text" | "varchar" | "char" | "string" => Scalar::new(TypeRef::STRING),
                    "boolean" | "bool" => Scalar::new(TypeRef::BOOLEAN),
                    "blob" | "binary" => {
                        warn!("Binary custom type '{}' mapped to String", name);
                        Scalar::new(TypeRef::STRING)
                    }
                    "json" | "jsonb" => {
                        debug!("JSON custom type '{}' mapped to String", name);
                        Scalar::new(TypeRef::STRING)
                    }
                    _ => {
                        warn!("Unknown custom type '{}', defaulting to String", name);
                        Scalar::new(TypeRef::STRING)
                    }
                }
            }

            // Array types - not directly supported, map to String (JSON array)
            DataType::Array(_) => {
                warn!(
                    "Array type {:?} mapped to String (JSON array format)",
                    self.data_type
                );
                Scalar::new(TypeRef::STRING)
            }

            // Unsupported types - default to String with warning
            unsupported_type => {
                warn!(
                    "Unsupported data type: {:?}, defaulting to String",
                    unsupported_type
                );
                Scalar::new(TypeRef::STRING)
            }
        };

        Ok(scalar)
    }
}

// to next version
impl ToGraphqlEnumExt for ColumnDef {
    fn to_enum(&self, table_name: &str) -> async_graphql::Result<Enum> {
        let mut graphql_enum = if self.data_type.to_string().starts_with("enum_text") {
            Enum::new(format!("{}_{}_enum", table_name, self.name))
        } else {
            return Err(async_graphql::Error::new("Cannot convert into enum"));
        };

        // get the check constraint
        for option in self.options.iter() {
            let check_expr_items = match &option.option {
                ColumnOption::Check(expr) => match expr {
                    Expr::InList {
                        expr: _,
                        list,
                        negated,
                    } => {
                        if !negated {
                            list
                        } else {
                            return Err(async_graphql::Error::new("Cannot convert into enum"));
                        }
                    }
                    _ => return Err(async_graphql::Error::new("Cannot convert into enum")),
                },
                _ => continue,
            };

            graphql_enum = graphql_enum.items(
                check_expr_items
                    .iter()
                    .map(|expr| EnumItem::new(expr.to_string().replace("'", ""))),
            );
        }

        Ok(graphql_enum)
    }
}

// done
impl ToGraphqlTypeRefExt for ColumnDef {
    fn to_type_ref(&self, table_name: &str) -> async_graphql::Result<TypeRef> {
        let graphql_type = if self.data_type.to_string().starts_with("enum_text") {
            let enum_value = self.to_enum(table_name)?;
            enum_value.type_name().to_owned()
        } else {
            self.to_scalar()?.type_name().to_owned()
        };

        if self
            .options
            .iter()
            .any(|spec| matches!(spec.option, ColumnOption::NotNull))
        {
            Ok(TypeRef::named_nn(graphql_type))
        } else {
            Ok(TypeRef::named(graphql_type))
        }
    }
}

// done
impl ToGraphqlFieldExt for ColumnDef {
    fn to_field(&self, table_name: String) -> async_graphql::Result<async_graphql::dynamic::Field> {
        let name = self.name.to_string();
        let column_def = self.clone();

        // Check for foreign key relationships in column options
        for column in column_def.options.iter() {
            if let ColumnOption::ForeignKey {
                foreign_table,
                referred_columns,
                on_delete: _,
                on_update: _,
                characteristics: _,
            } = &column.option
            {
                debug!("Foreign key found {}", column_def.name);
                let stripped_name = strip_id_suffix(&column_def.name.to_string());

                let is_not_null = column_def
                    .options
                    .iter()
                    .any(|spec| matches!(spec.option, ColumnOption::NotNull));

                let type_ref = if is_not_null {
                    TypeRef::named_nn(format!("{}_node", foreign_table))
                } else {
                    TypeRef::named(format!("{}_node", foreign_table))
                };

                if let Some(referred_column) = referred_columns.first() {
                    return Ok(Field::new(&stripped_name, type_ref, {
                        let foreign_table = foreign_table.to_string();
                        let referred_column = referred_column.to_string();
                        let column_def = column_def.clone();
                        let table_name = table_name.clone();
                        move |ctx| {
                            foreign_key_resolver(
                                table_name.clone(),
                                foreign_table.clone(),
                                referred_column.clone(),
                                column_def.clone(),
                                ctx,
                            )
                        }
                    }));
                } else {
                    warn!(
                        "Foreign key on column '{}' has no referred columns",
                        column_def.name
                    );
                }
            }
        }

        Ok(Field::new(
            &name,
            self.to_type_ref(&table_name)?,
            move |ctx| column_resolver(table_name.clone(), column_def.clone(), ctx),
        ))
    }
}

impl ToGraphqlInputValueExt for ColumnDef {
    fn to_input_value(
        &self,
        table_name: &str,
        force_nullable: bool,
    ) -> async_graphql::Result<async_graphql::dynamic::InputValue> {
        let graphql_type = if self.data_type.to_string().starts_with("enum_text") {
            let enum_value = self.to_enum(table_name)?;
            enum_value.type_name().to_owned()
        } else {
            self.to_scalar()?.type_name().to_owned()
        };

        let mut specs = self.options.iter();

        let is_not_null = specs.any(|spec| matches!(spec.option, ColumnOption::NotNull));

        let has_default_val = specs.any(|spec| matches!(spec.option, ColumnOption::Default(_)));

        if force_nullable {
            return Ok(InputValue::new(
                self.name.to_string(),
                TypeRef::named(graphql_type),
            ));
        }

        if is_not_null && !has_default_val {
            Ok(InputValue::new(
                self.name.to_string(),
                TypeRef::named_nn(graphql_type),
            ))
        } else {
            Ok(InputValue::new(
                self.name.to_string(),
                TypeRef::named(graphql_type),
            ))
        }
    }

    fn to_filter_condition(&self) -> async_graphql::Result<InputValue> {
        match self.to_scalar()?.type_name() {
            TypeRef::STRING => Ok(InputValue::new(
                self.name.to_string(),
                TypeRef::named("string_filter"),
            )),
            _ => Ok(InputValue::new(
                self.name.to_string(),
                TypeRef::named("string_filter"),
            )),
        }
    }
}

impl ToGraphqlNode for CreateTable {
    #[instrument(skip(self), fields(table_name = %self.name), level = "debug")]
    fn to_node(&self) -> async_graphql::Result<async_graphql::dynamic::Object> {
        let name = self.name.to_string();

        debug!("Creating node object for table '{}'", name);

        let mut table_node = Object::new(format!("{}_node", name));
        let mut foreign_columns = vec![];

        // Process table-level foreign key constraints
        for constraint in self.constraints.iter() {
            if let TableConstraint::ForeignKey {
                name: _,
                index_name: _,
                columns,
                foreign_table,
                referred_columns,
                on_delete: _,
                on_update: _,
                characteristics: _,
            } = constraint
            {
                for from_col in columns.iter() {
                    if let Some(referred_column) = referred_columns.first() {
                        debug!(
                            "Foreign key constraint: {} -> {}.{}",
                            from_col, foreign_table, referred_column
                        );

                        let stripped_name = strip_id_suffix(&from_col.to_string());

                        if let Some(column_def) =
                            self.columns.iter().find(|col| &col.name == from_col)
                        {
                            let is_not_null = column_def
                                .options
                                .iter()
                                .any(|spec| matches!(spec.option, ColumnOption::NotNull));

                            let type_ref = if is_not_null {
                                TypeRef::named_nn(format!("{}_node", foreign_table))
                            } else {
                                TypeRef::named(format!("{}_node", foreign_table))
                            };

                            table_node = table_node.field(Field::new(&stripped_name, type_ref, {
                                let foreign_table = foreign_table.to_string();
                                let referred_column = referred_column.to_string();
                                let column_def = column_def.clone();
                                let table_name = self.name.to_string();
                                move |ctx| {
                                    foreign_key_resolver(
                                        table_name.clone(),
                                        foreign_table.clone(),
                                        referred_column.clone(),
                                        column_def.clone(),
                                        ctx,
                                    )
                                }
                            }));

                            foreign_columns.push(column_def.clone());
                        } else {
                            warn!(
                                "Foreign key constraint references non-existent column '{}' in table '{}'",
                                from_col, name
                            );
                        }
                    } else {
                        warn!(
                            "Foreign key constraint for column '{}' has no referred columns",
                            from_col
                        );
                    }
                }
            }
        }

        // Add regular column fields (excluding those already added as foreign key fields)
        for col in self.columns.iter() {
            if !foreign_columns.contains(col) {
                let field = col.to_field(name.clone())?;
                debug!("Adding field '{}'", col.name);
                table_node = table_node.field(field);
            }
        }

        debug!(
            "Created node object for table '{}' with {} fields",
            self.name,
            self.columns.len()
        );

        Ok(table_node)
    }
}

impl ToGraphqlQueries for CreateTable {
    fn to_list_query(&self) -> async_graphql::Result<(async_graphql::dynamic::InputObject, Field)> {
        let table_name = self.name.to_string();

        let input = InputObject::new(format!("list_{}_input", table_name))
            .field(InputValue::new("page", TypeRef::named_nn(TypeRef::INT)))
            .field(InputValue::new("limit", TypeRef::named_nn(TypeRef::INT)))
            .field(InputValue::new(
                "where",
                TypeRef::named(format!("{}_filter_logic", self.name.to_string())),
            ));

        let table = self.clone();

        let list_field = Field::new(
            "list",
            TypeRef::named_list(format!("{}_node", table_name)),
            move |ctx| list_resolver(table.clone(), ctx),
        )
        .argument(InputValue::new(
            "input",
            TypeRef::named_nn(input.type_name()),
        ));

        Ok((input, list_field))
    }

    fn to_view_query(&self) -> async_graphql::Result<(InputObject, Field)> {
        let table_name = self.name.to_string();

        let input = InputObject::new(format!("view_{}_input", table_name))
            .field(InputValue::new("id", TypeRef::named_nn(TypeRef::INT)));

        let table = self.clone();

        let view_query = Field::new(
            "view",
            TypeRef::named(format!("{}_node", table_name)),
            move |ctx| view_resolver(table.clone(), ctx),
        )
        .argument(InputValue::new(
            "input",
            TypeRef::named_nn(input.type_name()),
        ));

        Ok((input, view_query))
    }
}

impl ToGraphqlMutations for CreateTable {
    #[instrument(skip(self), fields(table_name = %self.name), level = "debug")]
    fn to_insert_mutation(&self) -> async_graphql::Result<(InputObject, Field)> {
        debug!("Generating insert mutation for table '{}'", self.name);

        let mut input = InputObject::new(format!("insert_{}_input", self.name));

        let table_name = self.name.to_string();

        for col in self.columns.iter() {
            input = input.field(col.to_input_value(&self.name.to_string(), false)?);
        }

        let table_clone = self.clone();

        let insert_mutation_field = Field::new(
            format!("insert_{}", table_name),
            TypeRef::named_nn(format!("{}_node", table_name)),
            move |ctx| insert_resolver(table_clone.clone(), ctx),
        )
        .argument(InputValue::new(
            "input",
            TypeRef::named_nn(input.type_name()),
        ));

        debug!("Generated insert mutation for table '{}'", self.name);

        Ok((input, insert_mutation_field))
    }

    fn to_update_mutation(&self) -> async_graphql::Result<(InputObject, Field)> {
        let mut input = InputObject::new(format!("update_{}_input", self.name));
        let table_name = self.name.to_string();

        let pk_col = find_primary_key_column(self)?;

        let pk_input = InputValue::new("id", TypeRef::named_nn(pk_col.to_scalar()?.type_name()));

        for col in self.columns.iter() {
            input = input.field(col.to_input_value(&self.name.to_string(), true)?);
        }

        let table_clone = self.clone();

        let update_mutation_field = Field::new(
            format!("update_{}", table_name),
            TypeRef::named_nn(format!("{}_node", table_name)),
            move |ctx| update_resolver(table_clone.clone(), ctx),
        )
        .argument(pk_input)
        .argument(InputValue::new(
            "input",
            TypeRef::named_nn(input.type_name()),
        ));

        Ok((input, update_mutation_field))
    }

    fn to_delete_mutation(&self) -> async_graphql::Result<(InputObject, Field)> {
        let table_name = self.name.to_string();

        let pk_col = find_primary_key_column(self)?;

        let input = InputObject::new(format!("delete_{}_input", table_name)).field(
            InputValue::new("id", TypeRef::named_nn(pk_col.to_scalar()?.type_name())),
        );

        let table_clone = self.clone();

        let delete_mutation_field = Field::new(
            format!("delete_{}", table_name),
            TypeRef::named_nn(TypeRef::BOOLEAN),
            move |ctx| delete_resolver(table_clone.clone(), ctx),
        )
        .argument(InputValue::new(
            "input",
            TypeRef::named_nn(input.type_name()),
        ));

        Ok((input, delete_mutation_field))
    }
}

impl ToGraphqlObject for CreateTable {
    #[instrument(skip(self), fields(table_name = %self.name), level = "debug")]
    fn to_object(&self) -> async_graphql::Result<crate::traits::GraphQLObjectOutput> {
        debug!("Converting table '{}' to GraphQL object", self.name);

        let mut inputs = vec![];
        let mut mutations = vec![];
        let mut queries = vec![];
        let enums = self
            .columns
            .iter()
            .filter_map(|col| col.to_enum(&self.name.to_string()).ok())
            .collect::<Vec<_>>();

        let table_node = self.to_node()?;
        let table_name = self.name.to_string();

        debug!("Generating mutations for table '{}'", table_name);
        let insert_mutation = self.to_insert_mutation()?;
        let update_mutation = self.to_update_mutation()?;
        let delete_mutation = self.to_delete_mutation()?;

        let filters = self
            .columns
            .iter()
            .filter_map(|col| col.to_filter_condition().ok())
            .collect::<Vec<_>>();

        let mut filter_condition_input =
            InputObject::new(format!("{}_filter_condition", self.name.to_string()));

        for filter in filters {
            filter_condition_input = filter_condition_input.field(filter);
        }

        let filter_logic_input_name = format!("{}_filter_logic", self.name.to_string());

        let filter_logic_input = InputObject::new(filter_logic_input_name.clone())
            .field(InputValue::new(
                "and",
                TypeRef::named_list(filter_logic_input_name.clone()),
            ))
            .field(InputValue::new(
                "or",
                TypeRef::named_list(filter_logic_input_name.clone()),
            ))
            .field(InputValue::new(
                "not",
                TypeRef::named_list(filter_logic_input_name.clone()),
            ))
            .field(InputValue::new(
                "condition",
                TypeRef::named(filter_condition_input.type_name()),
            ));

        debug!("Generating queries for table '{}'", table_name);
        let list_query = self.to_list_query()?;
        let view_query = self.to_view_query()?;

        queries.push(
            Object::new(table_name.clone())
                .field(list_query.1)
                .field(view_query.1),
        );

        mutations.push(insert_mutation.1);
        mutations.push(update_mutation.1);
        mutations.push(delete_mutation.1);

        inputs.push(insert_mutation.0);
        inputs.push(update_mutation.0);
        inputs.push(delete_mutation.0);
        inputs.push(list_query.0);
        inputs.push(view_query.0);

        // filters
        inputs.push(filter_logic_input);
        inputs.push(filter_condition_input);

        debug!(
            "Generated GraphQL object for table '{}': {} queries, {} mutations, {} inputs",
            table_name,
            queries.len(),
            mutations.len(),
            inputs.len()
        );

        Ok(GraphQLObjectOutput {
            table: table_node,
            queries,
            mutations,
            inputs,
            enums,
        })
    }
}
