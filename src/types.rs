use anyhow::{Result, anyhow};
use async_graphql::dynamic::{
    Field, InputObject, InputValue, Object, Scalar, TypeRef, ValueAccessor,
};
use sea_query::{Alias, ColumnDef, ColumnSpec, ColumnType, Iden, SimpleExpr};
use sqlx::{SqlitePool, prelude::FromRow};

use crate::{
    resolvers::{
        column_resolver, delete_resolver, foreign_key_resolver, insert_resolver, list_resolver,
        update_resolver, view_resolver,
    },
    traits::{
        ToGraphqlFieldExt, ToGraphqlInputValueExt, ToGraphqlMutations, ToGraphqlNode,
        ToGraphqlObject, ToGraphqlQueries, ToGraphqlScalarExt, ToGraphqlTypeRefExt,
    },
};

#[derive(Debug, Clone)]
pub struct SqliteTable {
    pub table_info: TableInfo,
    pub column_info: Vec<ColumnDef>,
    pub foreign_key_info: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TableInfo {
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ColumnInfo {
    pub name: String,
    pub r#type: String,
    pub notnull: i16,
    pub pk: i16,
    pub dflt_value: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ForeignKeyInfo {
    pub table: String,
    pub from: String,
    pub to: String,
}

impl SqliteTable {
    pub async fn introspect(db: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let tables = sqlx::query_as::<_, TableInfo>(
            "SELECT name,sql FROM sqlite_master WHERE type='table' and name not in  ('_sqlx_migrations','sqlite_sequence')",
        )
        .fetch_all(db)
        .await?;

        if tables.is_empty() {
            return Err(anyhow::anyhow!("No tables found in the database"));
        }

        let mut sqlite_tables = Vec::new();

        // columns
        for table in tables.iter() {
            let columns = sqlx::query_as::<_, ColumnInfo>(
                r#"
              select name,type,"notnull",pk,dflt_value from pragma_table_info(?)
            "#,
            )
            .bind(&table.name)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|col| {
                let mut col_def = ColumnDef::new(Alias::new(col.name));

                match col.r#type.to_lowercase().as_str() {
                    "text" => col_def.text(),
                    "real" | "numeric" => col_def.float(),
                    "blob" => col_def.blob(),
                    "boolean" => col_def.boolean(),
                    "integer" => col_def.integer(),
                    _ => col_def.text(),
                };

                if col.notnull == 1 {
                    col_def.not_null();
                }

                if col.pk == 1 {
                    col_def.primary_key();
                }

                if col.dflt_value.is_some() {
                    col_def.default("");
                }

                col_def
            })
            .collect::<Vec<_>>();

            let query = r#"
                select "table","from","to" from pragma_foreign_key_list(?)
            "#;

            let foreign_keys = sqlx::query_as::<_, ForeignKeyInfo>(query)
                .bind(table.name.clone())
                .fetch_all(db)
                .await?;

            sqlite_tables.push(SqliteTable {
                table_info: table.clone(),
                column_info: columns,
                foreign_key_info: foreign_keys,
            });
        }

        Ok(sqlite_tables)
    }

    // helpers
    pub fn primary_key(&self) -> Result<&ColumnDef> {
        self.column_info
            .iter()
            .find(|col| {
                col.get_column_spec()
                    .iter()
                    .find(|spec| matches!(spec, sea_query::ColumnSpec::PrimaryKey))
                    .is_some()
            })
            .ok_or(anyhow::anyhow!("Unable to get primary key"))
    }

    pub fn table_name(&self) -> Alias {
        Alias::new(self.table_info.name.clone())
    }
}

pub trait ToSeaQueryValue {
    fn to_sea_query(&self, col_type: &ColumnType) -> async_graphql::Result<SimpleExpr>;
}

impl ToSeaQueryValue for ValueAccessor<'_> {
    fn to_sea_query(&self, col_type: &ColumnType) -> async_graphql::Result<SimpleExpr> {
        match col_type {
            ColumnType::Text => self.string().map(|val| Into::<SimpleExpr>::into(val)),
            ColumnType::Integer => self.i64().map(|val| Into::<SimpleExpr>::into(val)),
            ColumnType::Boolean => self.boolean().map(|val| Into::<SimpleExpr>::into(val)),
            ColumnType::Float => self.f64().map(|val| Into::<SimpleExpr>::into(val)),
            _ => self.string().map(|val| Into::<SimpleExpr>::into(val)),
        }
    }
}

impl ToGraphqlScalarExt for ColumnDef {
    fn to_scalar(&self) -> async_graphql::Result<Scalar> {
        let scalar = match self
            .get_column_type()
            .ok_or(anyhow!("Unable to get type"))?
        {
            ColumnType::Text => Scalar::new(TypeRef::STRING),
            ColumnType::Boolean => Scalar::new(TypeRef::BOOLEAN),
            ColumnType::Integer => Scalar::new(TypeRef::INT),
            ColumnType::Float => Scalar::new(TypeRef::FLOAT),
            ColumnType::Custom(r#type) => Scalar::new(r#type.to_string()),
            _ => Scalar::new(TypeRef::STRING),
        };

        Ok(scalar)
    }
}

impl ToGraphqlInputValueExt for ColumnDef {
    fn to_input_value(&self, force_nullable: bool) -> async_graphql::Result<InputValue> {
        let scalar = self.to_scalar()?;

        let mut specs = self.get_column_spec().iter();

        let is_not_null = specs
            .find(|spec| matches!(spec, ColumnSpec::NotNull))
            .is_some();

        let has_default_val = specs
            .find(|spec| matches!(spec, ColumnSpec::Default(_)))
            .is_some();

        if force_nullable {
            return Ok(InputValue::new(
                self.get_column_name(),
                TypeRef::named(scalar.type_name()),
            ));
        }

        if is_not_null && !has_default_val {
            Ok(InputValue::new(
                self.get_column_name(),
                TypeRef::named_nn(scalar.type_name()),
            ))
        } else {
            Ok(InputValue::new(
                self.get_column_name(),
                TypeRef::named(scalar.type_name()),
            ))
        }
    }
}

impl ToGraphqlTypeRefExt for ColumnDef {
    fn to_type_ref(&self) -> async_graphql::Result<TypeRef> {
        let scalar = self.to_scalar()?;

        if self
            .get_column_spec()
            .iter()
            .find(|spec| {
                matches!(spec, sea_query::ColumnSpec::NotNull)
                    || !matches!(spec, ColumnSpec::Default(_))
            })
            .is_some()
        {
            Ok(TypeRef::named_nn(scalar.type_name()))
        } else {
            Ok(TypeRef::named(scalar.type_name()))
        }
    }
}

impl ToGraphqlFieldExt for ColumnDef {
    fn to_field(
        &self,
        table_name: String,
        f_col: Option<ForeignKeyInfo>,
    ) -> async_graphql::Result<Field> {
        let column_name = self.get_column_name();
        let column_def = self.clone();
        let table_name = table_name.clone();

        if let Some(f_col) = f_col {
            let stripped_name = if column_name.ends_with("_id") {
                column_name.trim_end_matches("_id").to_string()
            } else {
                column_name.clone()
            };

            let type_ref = if self
                .get_column_spec()
                .iter()
                .find(|spec| {
                    matches!(spec, sea_query::ColumnSpec::NotNull)
                        || !matches!(spec, ColumnSpec::Default(_))
                })
                .is_some()
            {
                TypeRef::named_nn(format!("{}_node", f_col.table))
            } else {
                TypeRef::named(format!("{}_node", f_col.table))
            };

            Ok(Field::new(&stripped_name, type_ref, move |ctx| {
                foreign_key_resolver(table_name.clone(), f_col.clone(), ctx)
            }))
        } else {
            Ok(Field::new(&column_name, self.to_type_ref()?, move |ctx| {
                column_resolver(table_name.clone(), column_def.clone(), ctx)
            }))
        }
    }
}

impl ToGraphqlMutations for SqliteTable {
    fn to_insert_mutation(&self) -> async_graphql::Result<(InputObject, Field)> {
        let mut input = InputObject::new(format!("insert_{}_input", self.table_info.name));
        let table_name = self.table_info.name.clone();

        for col in self.column_info.iter() {
            input = input.field(col.to_input_value(false)?);
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

        Ok((input, insert_mutation_field))
    }

    fn to_update_mutation(&self) -> async_graphql::Result<(InputObject, Field)> {
        let mut input = InputObject::new(format!("update_{}_input", self.table_info.name));
        let table_name = self.table_info.name.to_string();

        let pk_col = self
            .column_info
            .iter()
            .find(|col| {
                col.get_column_spec()
                    .iter()
                    .any(|spec| matches!(spec, ColumnSpec::PrimaryKey))
            })
            .unwrap()
            .to_scalar()?;

        let pk_input = InputValue::new("id", TypeRef::named_nn(pk_col.type_name()));

        for col in self.column_info.iter() {
            input = input.field(col.to_input_value(true)?);
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
        let pk_col = self.primary_key()?;
        let table_name = self.table_info.name.clone();

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

impl ToGraphqlNode for SqliteTable {
    fn to_node(&self) -> async_graphql::Result<Object> {
        let table_name = self.table_info.name.clone();

        let mut node_obj = Object::new(format!("{}_node", table_name.clone()));

        for col in self.column_info.clone() {
            if let Some(f_col) = self
                .foreign_key_info
                .iter()
                .find(|f_col| f_col.from == col.get_column_name())
            {
                node_obj = node_obj.field(col.to_field(table_name.clone(), Some(f_col.clone()))?);
            } else {
                node_obj = node_obj.field(col.to_field(table_name.clone(), None)?);
            }
        }

        Ok(node_obj)
    }
}

impl ToGraphqlQueries for SqliteTable {
    fn to_list_query(&self) -> async_graphql::Result<(InputObject, Field)> {
        let table_name = self.table_info.name.clone();

        let input = InputObject::new(format!("list_{}_input", table_name))
            .field(InputValue::new("page", TypeRef::named_nn(TypeRef::INT)))
            .field(InputValue::new("limit", TypeRef::named_nn(TypeRef::INT)));

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
        let table_name = self.table_info.name.clone();

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

impl ToGraphqlObject for SqliteTable {
    fn to_object(
        &self,
    ) -> async_graphql::Result<(Object, Vec<Object>, Vec<Field>, Vec<InputObject>)> {
        let mut inputs = vec![];
        let mut mutations = vec![];
        let mut queries = vec![];

        let table_node = self.to_node()?;
        let table_name = self.table_name();

        let insert_mutation = self.to_insert_mutation()?;
        let update_mutation = self.to_update_mutation()?;
        let delete_mutation = self.to_delete_mutation()?;

        let list_query = self.to_list_query()?;
        let view_query = self.to_view_query()?;

        queries.push(
            Object::new(table_name.to_string())
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

        Ok((table_node, queries, mutations, inputs))
    }
}
