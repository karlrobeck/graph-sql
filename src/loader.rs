use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::*;
use sea_query::{Alias, Expr, Iden, Query, SqliteQueryBuilder};
use sqlx::SqlitePool;
use tracing::{debug, instrument};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnRowDef {
    pub table: Alias,
    pub column: Alias,
    pub value: serde_json::Value,
    pub primary_column: Alias,
}

pub struct ColumnRowLoader {
    pub pool: SqlitePool,
}

impl Loader<ColumnRowDef> for ColumnRowLoader {
    type Error = Arc<sqlx::Error>;
    type Value = serde_json::Value;

    #[instrument(skip(self), level = "debug")]
    async fn load(
        &self,
        keys: &[ColumnRowDef],
    ) -> Result<std::collections::HashMap<ColumnRowDef, Self::Value>, Self::Error> {
        debug!("Loading {} keys", keys.len());
        let mut grouped_keys: HashMap<(Alias, Alias, Alias), Vec<serde_json::Value>> =
            HashMap::new();

        for key in keys {
            let group = (
                key.table.clone(),
                key.primary_column.clone(),
                key.column.clone(),
            );
            grouped_keys
                .entry(group)
                .or_default()
                .push(key.value.clone());
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
                .expr(Expr::cust(format!(
                    "json_object('id', {}, 'value', {})",
                    pk_col.to_string(),
                    val_col.to_string()
                )))
                .and_where(Expr::col(pk_col.clone()).is_in(pk_values))
                .to_string(SqliteQueryBuilder);

            debug!("Generated SQL: {}", sql);
            let rows = sqlx::query_as::<_, (serde_json::Value,)>(&sql)
                .fetch_all(&self.pool)
                .await?;
            debug!("Fetched {} rows from database", rows.len());

            for (row,) in rows.iter() {
                final_results.insert(
                    ColumnRowDef {
                        table: table.clone(),
                        primary_column: pk_col.clone(),
                        column: val_col.clone(),
                        value: row.get("id").unwrap().clone(),
                    },
                    row.get("value").unwrap().clone(),
                );
            }
        }

        debug!("Returning {} results", final_results.len());

        Ok(final_results)
    }
}
