use async_graphql::dynamic::{Field, InputObject, InputValue, Object, TypeRef};
use sea_query::{Alias, ColumnDef, ColumnSpec, ColumnType};
use sqlx::{SqlitePool, prelude::FromRow};

use crate::resolvers::{column_resolver, insert_resolver, update_resolver};

#[derive(Debug, Clone)]
pub struct SqliteTable {
    pub table_info: TableInfo,
    pub column_info: Vec<ColumnDef>,
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

        for table in tables {
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

            sqlite_tables.push(SqliteTable {
                table_info: table,
                column_info: columns,
            });
        }

        Ok(sqlite_tables)
    }

    pub fn to_graphql_object(&self) -> Object {
        let table_name = self.table_info.name.clone();

        let mut table_obj = Object::new(table_name.clone());

        for col in self.column_info.clone() {
            table_obj = table_obj.field(col.graphql_field(table_name.clone()));
        }

        table_obj
    }

    pub fn to_graphql_insert_mutation(&self) -> (InputObject, Field) {
        let mut input = InputObject::new(format!("insert_{}_input", self.table_info.name));

        for col in self.column_info.iter() {
            input = input.field(col.graphql_input(false));
        }

        let table_clone = self.clone();
        let insert_mutation_field = Field::new(
            format!("insert_{}", table_clone.table_info.name),
            TypeRef::named_nn(table_clone.table_info.name.clone()),
            move |ctx| insert_resolver(table_clone.clone(), ctx),
        )
        .argument(InputValue::new(
            "input",
            TypeRef::named_nn(input.type_name()),
        ));

        (input, insert_mutation_field)
    }

    pub fn to_graphql_update_mutation(&self) -> (InputObject, Field) {
        let mut input = InputObject::new(format!("update_{}_input", self.table_info.name));

        let pk_col = self
            .column_info
            .iter()
            .find(|col| {
                col.get_column_spec()
                    .iter()
                    .any(|spec| matches!(spec, ColumnSpec::PrimaryKey))
            })
            .unwrap();

        let pk_input = InputValue::new("id", TypeRef::named_nn(pk_col.type_name()));

        for col in self.column_info.iter() {
            input = input.field(col.graphql_input(true));
        }

        let table_clone = self.clone();
        let insert_mutation_field = Field::new(
            format!("update_{}", table_clone.table_info.name),
            TypeRef::named_nn(table_clone.table_info.name.clone()),
            move |ctx| update_resolver(table_clone.clone(), ctx),
        )
        .argument(pk_input)
        .argument(InputValue::new(
            "input",
            TypeRef::named_nn(input.type_name()),
        ));

        (input, insert_mutation_field)
    }
}

pub trait ToGraphQL {
    fn type_name(&self) -> impl Into<String>;
    fn type_ref(&self) -> TypeRef;
    fn graphql_field(&self, table_name: String) -> Field;
    fn graphql_input(&self, force_nullable: bool) -> InputValue;
}

impl ToGraphQL for ColumnDef {
    fn type_ref(&self) -> TypeRef {
        let type_name = self.type_name();
        if self
            .get_column_spec()
            .iter()
            .find(|spec| {
                matches!(spec, sea_query::ColumnSpec::NotNull)
                    || !matches!(spec, ColumnSpec::Default(_))
            })
            .is_some()
        {
            TypeRef::named_nn(type_name)
        } else {
            TypeRef::named(type_name)
        }
    }

    fn graphql_field(&self, table_name: String) -> Field {
        let column_name = self.get_column_name().to_string();
        let column_def = self.clone();
        let table_name = table_name.clone();

        Field::new(&column_name, self.type_ref(), move |ctx| {
            column_resolver(table_name.clone(), column_def.clone(), ctx)
        })
    }

    fn graphql_input(&self, force_nullable: bool) -> InputValue {
        let type_name = self.type_name();

        let mut specs = self.get_column_spec().iter();

        let is_not_null = specs
            .find(|spec| matches!(spec, ColumnSpec::NotNull))
            .is_some();

        let has_default_val = specs
            .find(|spec| matches!(spec, ColumnSpec::Default(_)))
            .is_some();

        if force_nullable {
            return InputValue::new(self.get_column_name(), TypeRef::named(type_name));
        }

        if is_not_null && !has_default_val {
            InputValue::new(self.get_column_name(), TypeRef::named_nn(type_name))
        } else {
            InputValue::new(self.get_column_name(), TypeRef::named(type_name))
        }
    }

    fn type_name(&self) -> impl Into<String> {
        match self.get_column_type().unwrap() {
            ColumnType::Text => TypeRef::STRING,
            ColumnType::Float => TypeRef::FLOAT,
            ColumnType::Blob => TypeRef::STRING,
            ColumnType::Integer | ColumnType::Boolean => TypeRef::INT,
            _ => TypeRef::STRING,
        }
    }
}
