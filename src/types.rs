use async_graphql::dynamic::TypeRef;
use sqlx::{SqlitePool, prelude::FromRow};

#[derive(Debug)]
pub struct SqliteTable {
    pub table_info: TableInfo,
    pub column_info: Vec<ColumnInfo>,
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
}

impl SqliteTable {
    pub async fn introspect(db: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let tables = sqlx::query_as::<_, TableInfo>(
            "SELECT name FROM sqlite_master WHERE type='table' and name not in  ('_sqlx_migrations','sqlite_sequence')",
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
              select name,type,"notnull",pk from pragma_table_info(?)
            "#,
            )
            .bind(&table.name)
            .fetch_all(db)
            .await?;

            sqlite_tables.push(SqliteTable {
                table_info: table,
                column_info: columns,
            });
        }

        Ok(sqlite_tables)
    }
}

pub fn create_graphql_type_ref(column: &ColumnInfo) -> TypeRef {
    let type_name = match column.r#type.as_str() {
        "INTEGER" => TypeRef::INT,
        "TEXT" => TypeRef::STRING,
        "REAL" => TypeRef::FLOAT,
        "BLOB" => TypeRef::STRING,
        _ => TypeRef::STRING,
    };

    match column.notnull {
        1 => TypeRef::named_nn(type_name),
        _ => TypeRef::named(type_name),
    }
}

pub fn create_graphql_field(col: &ColumnInfo) {
    let name = col.name.clone();
}
