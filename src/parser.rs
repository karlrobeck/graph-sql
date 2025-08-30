use anyhow::anyhow;
use async_graphql::dynamic::{
    Enum, EnumItem, Field, InputObject, InputValue, Object, Scalar, TypeRef,
};
use sqlparser::ast::{ColumnDef, ColumnOption, CreateTable, DataType, Expr, TableConstraint};
use sqlx::SqlitePool;
use tracing::{debug, instrument, warn};

use crate::{
    resolvers::{
        column_resolver_gen, delete_resolver, foreign_key_resolver, insert_resolver,
        list_resolver_gen, update_resolver, view_resolver_gen,
    },
    traits::{
        GraphQLObjectOutput, ToGraphqlEnumExt, ToGraphqlFieldExt, ToGraphqlInputValueExt,
        ToGraphqlMutations, ToGraphqlNode, ToGraphqlObject, ToGraphqlQueries, ToGraphqlScalarExt,
        ToGraphqlTypeRefExt,
    },
    utils::{find_primary_key_column, strip_id_suffix},
};

pub trait Introspector
where
    Self: Sized,
{
    fn introspect(
        pool: &SqlitePool,
    ) -> impl std::future::Future<Output = async_graphql::Result<Vec<Self>>> + Send;
}

#[derive(Clone)]
pub struct TableDef {
    pub name: String,                // name of the table
    pub columns: Vec<ColDef>,        // column definitions
    pub description: Option<String>, // table description
}

#[derive(Clone)]
pub struct ColDef {
    pub table_name: String,          // name of the table that it belongs to
    pub name: String,                // name of the column
    pub data_type: ColDataType,      // data type of the column
    pub not_null: bool,              // has not null constraint
    pub is_primary: bool,            // is primary key
    pub description: Option<String>, // column description / comment
    pub relationship: Option<ForeignColDef>,
}

#[derive(Clone)]
pub struct ForeignColDef {
    pub table: String, // The name of the parent table referenced by the foreign key.
    pub from: String,  // The name of the column in the child table (the table you're querying).
    pub to: String,    // The name of the column in the parent table that is referenced.
}

#[derive(Clone)]
pub enum ColDataType {
    String,
    Integer,
    Float,
    Boolean,
}

pub struct ListQuery(async_graphql::dynamic::Field);

pub struct ViewQuery(async_graphql::dynamic::Field);

pub struct NodeInputValues(
    async_graphql::dynamic::InputValue,
    async_graphql::dynamic::InputValue,
);

pub struct InsertMutation(
    async_graphql::dynamic::Field,
    Vec<async_graphql::dynamic::InputObject>,
);

pub struct UpdateMutation(
    async_graphql::dynamic::Field,
    Vec<async_graphql::dynamic::InputObject>,
);

pub struct DeleteMutation(async_graphql::dynamic::Field);

impl TryFrom<String> for ColDataType {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "text" => Ok(Self::String),
            "integer" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            "boolean" => Ok(Self::Boolean),
            _ => Err(anyhow!("unsupported data type")),
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
        let description = value.description.clone();

        // if let Some(_) = value.relationship {
        //     todo!("return foreign key resolver")
        // }

        Field::new(
            value.name.clone(),
            TypeRef::from(value.clone()),
            move |ctx| column_resolver_gen(value.clone(), ctx),
        )
        .description(description.unwrap_or_default())
    }
}

impl From<ColDef> for NodeInputValues {
    fn from(value: ColDef) -> Self {
        let graphql_type = Scalar::from(value.data_type);

        let type_ref = if value.not_null {
            TypeRef::named_nn(graphql_type.type_name())
        } else {
            TypeRef::named(graphql_type.type_name())
        };

        NodeInputValues(
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

        table_node.description(value.description.unwrap_or_default())
    }
}

// old code

impl From<TableDef> for ListQuery {
    fn from(value: TableDef) -> Self {
        let description = value.description.clone().unwrap_or_default();

        let field = Field::new(
            pluralizer::pluralize(&value.name.clone(), 2, false), // todo: make this plural properly
            TypeRef::named_list(format!("{}_node", value.name)),
            move |ctx| list_resolver_gen(value.clone(), ctx),
        )
        .argument(InputValue::new("page", TypeRef::named_nn(TypeRef::INT)))
        .argument(InputValue::new("perPage", TypeRef::named_nn(TypeRef::INT)));

        ListQuery(field.description(description))
    }
}

impl From<TableDef> for ViewQuery {
    fn from(value: TableDef) -> Self {
        let description = value.clone().description.unwrap_or_default();

        let pk_col = value
            .columns
            .iter()
            .find(|col| col.is_primary)
            .expect("Primary column required")
            .clone();

        let field = Field::new(
            pluralizer::pluralize(&value.name.clone(), 1, false), // todo: make this plural properly
            TypeRef::named(format!("{}_node", value.name)),
            move |ctx| view_resolver_gen(value.clone(), ctx),
        )
        .argument(InputValue::new(
            pk_col.name,
            TypeRef::named_nn(Scalar::from(pk_col.data_type).type_name()),
        ));

        ViewQuery(field.description(description))
    }
}

impl From<TableDef> for InsertMutation {
    fn from(value: TableDef) -> Self {
        let mut input = InputObject::new(format!("insert_{}_input", value.name));

        for col in value.columns.iter() {
            let NodeInputValues(insert, _) = NodeInputValues::from(col.clone());
            input = input.field(insert);
        }

        let field = Field::new(
            format!("insert_{}", value.name.clone()), // todo: make this plural properly
            TypeRef::named(format!("{}_node", value.name)),
            move |ctx| insert_resolver(value.clone(), ctx),
        )
        .argument(InputValue::new(
            "value",
            TypeRef::named_nn(input.type_name()),
        ));

        InsertMutation(field, vec![input])
    }
}

impl From<TableDef> for UpdateMutation {
    fn from(value: TableDef) -> Self {
        let mut input = InputObject::new(format!("update_{}_input", value.name));

        let pk_col = value
            .columns
            .iter()
            .find(|col| col.is_primary)
            .expect("Primary column required")
            .clone();

        for col in value.columns.iter() {
            let NodeInputValues(_, update) = NodeInputValues::from(col.clone());
            input = input.field(update);
        }

        let field = Field::new(
            format!("update_{}", value.name.clone()), // todo: make this plural properly
            TypeRef::named(format!("{}_node", value.name)),
            move |ctx| update_resolver(value.clone(), ctx),
        )
        .argument(InputValue::new(
            pk_col.name,
            TypeRef::named_nn(Scalar::from(pk_col.data_type).type_name()),
        ))
        .argument(InputValue::new(
            "value",
            TypeRef::named_nn(input.type_name()),
        ));

        UpdateMutation(field, vec![input])
    }
}

impl From<TableDef> for DeleteMutation {
    fn from(value: TableDef) -> Self {
        let pk_col = value
            .columns
            .iter()
            .find(|col| col.is_primary)
            .expect("Primary column required")
            .clone();

        let field = Field::new(
            format!("delete_{}", value.name.clone()), // todo: make this plural properly
            TypeRef::named(format!("{}_node", value.name)),
            move |ctx| delete_resolver(value.clone(), ctx),
        )
        .argument(InputValue::new(
            pk_col.name,
            TypeRef::named_nn(Scalar::from(pk_col.data_type).type_name()),
        ));

        DeleteMutation(field)
    }
}

impl From<TableDef> for crate::traits::GraphQLObjectOutput {
    fn from(value: TableDef) -> Self {
        let mut inputs = vec![];
        let mut mutations = vec![];
        let mut queries = vec![];

        let table_obj_node = Object::from(value.clone());

        let insert_mutation = InsertMutation::from(value.clone());
        let update_mutation = UpdateMutation::from(value.clone());
        let delete_mutation = DeleteMutation::from(value.clone());

        let list_query = ListQuery::from(value.clone());
        let view_query = ViewQuery::from(value.clone());

        queries.push(list_query.0);
        queries.push(view_query.0);

        mutations.push(insert_mutation.0);
        mutations.push(update_mutation.0);
        mutations.push(delete_mutation.0);

        inputs.push(insert_mutation.1);
        inputs.push(update_mutation.1);

        GraphQLObjectOutput {
            table: table_obj_node,
            queries,
            mutations,
            inputs: inputs.into_iter().flatten().collect::<Vec<_>>(),
            enums: vec![],
        }
    }
}

impl Introspector for TableDef {
    async fn introspect(pool: &SqlitePool) -> async_graphql::Result<Vec<Self>> {
        // get the table info and its column
        let table_query =
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'";

        let tables = sqlx::query_as::<_, (String,)>(table_query)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|(table,)| table)
            .collect::<Vec<_>>();

        let mut result = Vec::new();

        for table_name in tables {
            // Get column information using pragma_table_info
            let column_query =
                "SELECT cid, name, type, \"notnull\", dflt_value, pk FROM pragma_table_info(?)";

            let column_rows =
                sqlx::query_as::<_, (i32, String, String, i32, Option<String>, i32)>(column_query)
                    .bind(&table_name)
                    .fetch_all(pool)
                    .await?;

            let mut columns = Vec::new();

            for (_, col_name, col_type, not_null, _default_value, is_primary) in column_rows {
                // Convert SQLite type to our ColDataType
                let data_type = match col_type.to_lowercase().as_str() {
                    "text" | "varchar" | "char" | "string" => ColDataType::String,
                    "integer" | "int" | "bigint" | "smallint" => ColDataType::Integer,
                    "real" | "float" | "double" | "numeric" => ColDataType::Float,
                    "boolean" | "bool" => ColDataType::Boolean,
                    _ => {
                        // Default to string for unknown types
                        debug!(
                            "Unknown column type '{}' for column '{}', defaulting to String",
                            col_type, col_name
                        );
                        ColDataType::String
                    }
                };

                // Get foreign key information for this column
                let fk_query = "SELECT \"table\", \"from\", \"to\" FROM pragma_foreign_key_list(?) WHERE \"from\" = ?";
                let fk_rows = sqlx::query_as::<_, (String, String, String)>(fk_query)
                    .bind(&table_name)
                    .bind(&col_name)
                    .fetch_all(pool)
                    .await?;

                let relationship = fk_rows.first().map(|(table, from, to)| ForeignColDef {
                    table: table.clone(),
                    from: from.clone(),
                    to: to.clone(),
                });

                let col_def = ColDef {
                    table_name: table_name.clone(),
                    name: col_name,
                    data_type,
                    not_null: not_null == 1,
                    is_primary: is_primary == 1,
                    description: None, // Skip description for now
                    relationship,
                };

                columns.push(col_def);
            }

            let table_def = TableDef {
                name: table_name,
                columns,
                description: None, // Skip description for now
            };

            result.push(table_def);
        }

        Ok(result)
    }
}
