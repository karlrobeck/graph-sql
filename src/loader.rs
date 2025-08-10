use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::*;
use sea_query::{Alias, Expr, Query, SqliteQueryBuilder};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use tracing::{debug, instrument};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnRowDef {
    pub table: Alias,
    pub column: Alias,
    pub value: i64,
    pub primary_column: Alias,
}

pub struct ColumnRowLoader {
    pub pool: SqlitePool,
}

impl Loader<ColumnRowDef> for ColumnRowLoader {
    type Error = Arc<sqlx::Error>;
    type Value = Vec<u8>;

    #[instrument(skip(self), level = "debug")]
    async fn load(
        &self,
        keys: &[ColumnRowDef],
    ) -> Result<std::collections::HashMap<ColumnRowDef, Self::Value>, Self::Error> {
        debug!("Loading {} keys", keys.len());
        let mut grouped_keys: HashMap<(Alias, Alias, Alias), Vec<i64>> = HashMap::new();

        for key in keys {
            let group = (
                key.table.clone(),
                key.primary_column.clone(),
                key.column.clone(),
            );
            grouped_keys.entry(group).or_default().push(key.value);
        }

        debug!("Grouped keys into {} queries", grouped_keys.len());
        let mut final_results: HashMap<ColumnRowDef, Self::Value> = HashMap::new();

        for ((table, pk_col, val_col), pk_values) in grouped_keys {
            debug!(
                "Processing query for table: {:?}, pk_col: {:?}, val_col: {:?}, {} values",
                table,
                pk_col,
                val_col,
                pk_values.len()
            );

            let sql = Query::select()
                .from(table.clone())
                .exprs([
                    Expr::col(pk_col.clone()),
                    Expr::expr(Expr::col(val_col.clone()).cast_as("blob")),
                ])
                .and_where(Expr::col(pk_col.clone()).is_in(pk_values))
                .to_string(SqliteQueryBuilder);

            debug!("Generated SQL: {}", sql);
            let rows: Vec<SqliteRow> = sqlx::query(&sql).fetch_all(&self.pool).await?;
            debug!("Fetched {} rows from database", rows.len());

            for row in rows {
                let pk_val: i64 = row.try_get(0)?; // First column is the primary key
                let fetched_val: Self::Value = row.try_get(1)?; // Second column is the desired value

                let original_key = ColumnRowDef {
                    table: table.clone(),
                    primary_column: pk_col.clone(),
                    column: val_col.clone(),
                    value: pk_val,
                };
                final_results.insert(original_key, fetched_val);
            }
        }

        debug!("Returning {} results", final_results.len());
        Ok(final_results)
    }
}
