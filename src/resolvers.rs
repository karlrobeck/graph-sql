use anyhow::anyhow;
use async_graphql::{
    Value,
    dynamic::{FieldFuture, ResolverContext},
};
use sea_query::{Alias, ColumnDef, ColumnSpec, Expr, Query, SqliteQueryBuilder};
use sqlx::SqlitePool;
use tracing::debug;

use crate::types::{ForeignKeyInfo, SqliteTable, ToSeaQueryValue};

pub fn list_resolver(table_info: SqliteTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing list resolver for table: {:?}",
            table_info.table_name()
        );

        let db = ctx.data::<SqlitePool>()?;
        let table_name = table_info.table_name();
        let pk_col = table_info.primary_key()?;

        let input = ctx.args.try_get("input")?.object()?;

        let page = input.try_get("page")?.u64()?;
        let limit = input.try_get("limit")?.u64()?;

        debug!("List query parameters - page: {}, limit: {}", page, limit);

        let query = Query::select()
            .from(table_name)
            .column(Alias::new(pk_col.get_column_name()))
            .offset((page - 1) * limit)
            .limit(limit)
            .to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|(val,)| {
                serde_json::json!({
                  "name":pk_col.get_column_name(),
                  "id":val,
                })
            })
            .map(|val| Value::from_json(val).unwrap())
            .collect::<Vec<_>>();

        debug!("List resolver returned {} items", result.len());
        Ok(Some(Value::List(result)))
    })
}

pub fn view_resolver(table_info: SqliteTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing view resolver for table: {:?}",
            table_info.table_name()
        );

        let db = ctx.data::<SqlitePool>()?;

        let id = ctx
            .args
            .get("input")
            .ok_or(anyhow::anyhow!("Unable to get id"))?
            .object()?
            .get("id")
            .ok_or(anyhow!("Unable to get id"))?
            .i64()?;

        debug!("View query for ID: {}", id);

        let table_name = table_info.table_name();

        let pk_col = table_info.primary_key()?;

        let query = Query::select()
            .from(table_name)
            .column(Alias::new(pk_col.get_column_name()))
            .and_where(Expr::col(Alias::new(pk_col.get_column_name())).eq(id))
            .to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| {
                serde_json::json!({
                  "name":pk_col.get_column_name(),
                  "id":val,
                })
            })
            .map(|val| Value::from_json(val).unwrap())?;

        debug!("View resolver found record with ID: {}", id);
        Ok(Some(result))
    })
}

pub fn foreign_key_resolver(
    table_name: String,
    f_col: ForeignKeyInfo,
    ctx: ResolverContext<'_>,
) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing foreign key resolver for table: {} -> {}",
            table_name, f_col.table
        );

        let db = ctx.data::<SqlitePool>()?;

        let parent_value = ctx
            .parent_value
            .as_value()
            .ok_or(anyhow::anyhow!("Unable to get parent value"))?
            .clone();

        let parent_value = parent_value.into_json()?;

        let json_object = parent_value
            .as_object()
            .ok_or(anyhow::anyhow!("Unable to get json object"))?;

        let pk_name = json_object
            .get("name")
            .map(|val| val.as_str())
            .ok_or(anyhow::anyhow!("Unable to get primary key column name"))?
            .ok_or(anyhow::anyhow!("Unable to cast column name as str"))?;

        let pk_id = json_object
            .get("id")
            .map(|v| v.as_i64())
            .ok_or(anyhow::anyhow!("Unable to get primary key id"))?
            .ok_or(anyhow::anyhow!("Unable to cast id into i64"))?;

        let query = Query::select()
            .from_as(Alias::new(f_col.table.clone()), Alias::new("f"))
            .expr(Expr::cust_with_values(
                format!("json_object(?,f.{})", f_col.to),
                [f_col.to.clone()],
            ))
            .inner_join(
                Alias::new(table_name.clone()),
                Expr::col((
                    Alias::new(table_name.clone()),
                    Alias::new(f_col.from.clone()),
                ))
                .equals((Alias::new("f"), Alias::new(f_col.to.clone()))),
            )
            .and_where(Expr::col((Alias::new(table_name.clone()), Alias::new(pk_name))).eq(pk_id))
            .to_string(SqliteQueryBuilder);

        let result = sqlx::query_as::<_, (serde_json::Value,)>(&query)
            .fetch_one(db)
            .await
            .map(|(map_val,)| map_val.as_object().unwrap().clone())
            .map(|val| {
                serde_json::json!({
                    "name":f_col.to,
                    "id":val.get(&f_col.to).unwrap()
                })
            })
            .map(Value::from_json)?;

        Ok(Some(result?))
    })
}

pub fn column_resolver(
    table_name: String,
    col: ColumnDef,
    ctx: ResolverContext<'_>,
) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing column resolver for table: {} column: {}",
            table_name,
            col.get_column_name()
        );

        let db = ctx.data::<SqlitePool>()?;

        let parent_value = ctx
            .parent_value
            .as_value()
            .ok_or(anyhow::anyhow!("Unable to get parent value"))?
            .clone();

        let parent_value = parent_value.into_json()?;

        let json_object = parent_value
            .as_object()
            .ok_or(anyhow::anyhow!("Unable to get json object"))?;

        let pk_name = json_object
            .get("name")
            .map(|val| val.as_str())
            .ok_or(anyhow::anyhow!("Unable to get primary key column name"))?
            .ok_or(anyhow::anyhow!("Unable to cast column name as str"))?;

        let pk_id = json_object
            .get("id")
            .map(|v| v.as_i64())
            .ok_or(anyhow::anyhow!("Unable to get primary key id"))?
            .ok_or(anyhow::anyhow!("Unable to cast id into i64"))?;

        let query = Query::select()
            .from(Alias::new(table_name))
            .expr(Expr::cust_with_values(
                format!("json_object(?,{})", col.get_column_name()),
                [col.get_column_name()],
            ))
            .and_where(Expr::col(Alias::new(pk_name)).eq(pk_id))
            .to_string(SqliteQueryBuilder);

        let result = sqlx::query_as::<_, (serde_json::Value,)>(&query)
            .fetch_one(db)
            .await
            .map(|(map_val,)| map_val.as_object().unwrap().clone())
            .map(|val| val.get(&col.get_column_name()).unwrap().clone())
            .map(Value::from_json)?;

        Ok(Some(result?))
    })
}

pub fn insert_resolver(table: SqliteTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing insert resolver for table: {:?}",
            table.table_name()
        );

        let db = ctx.data::<SqlitePool>()?;
        let table_name = table.table_name();

        let input = ctx.args.try_get("input")?;

        let input = input.object()?;

        debug!("Insert data: {} fields", input.len());

        let mut binding = Query::insert();

        let pkey_col = table
            .column_info
            .iter()
            .find(|col| {
                col.get_column_spec()
                    .iter()
                    .any(|spec| matches!(spec, ColumnSpec::PrimaryKey))
            })
            .unwrap();

        let query = binding
            .into_table(table_name)
            .columns(input.iter().map(|(name, _)| Alias::new(name.to_string())));

        let mut values = vec![];

        for (key, val) in input.iter() {
            debug!("Processing field: {}", key);
            let col_type = table
                .column_info
                .iter()
                .find(|col| col.get_column_name() == *key)
                .ok_or(anyhow::anyhow!("Unable to get column"))?
                .get_column_type()
                .ok_or(anyhow::anyhow!("Unable to get column type"))?;

            values.push(val.to_sea_query(col_type)?);
        }

        let query =
            query.returning(Query::returning().column(Alias::new(pkey_col.get_column_name())));

        let query = query.values(values)?.to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| {
                serde_json::json!({
                    "name": pkey_col.get_column_name(),
                    "id": val
                })
            })?;

        debug!("Insert completed, new ID: {:?}", result);
        Ok(Some(Value::from_json(result)?))
    })
}

pub fn update_resolver(table: SqliteTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing update resolver for table: {:?}",
            table.table_name()
        );

        let table_name = table.table_name();

        let pk_col = table.primary_key()?;

        let db = ctx.data::<SqlitePool>()?;

        let id = ctx.args.try_get("id")?.i64()?;

        debug!("Update query for ID: {}", id);

        let input = ctx.args.try_get("input")?.object()?;

        debug!("Update data: {} fields", input.len());

        let mut binding = Query::update();

        // Build the update query
        let mut query = binding.table(table_name);

        // Collect columns and values to update
        let mut values = vec![];

        for (key, val) in input.iter() {
            debug!("Processing field: {}", key);
            let col_type = table
                .column_info
                .iter()
                .find(|col| col.get_column_name() == *key)
                .ok_or(anyhow::anyhow!("Unable to get column"))?
                .get_column_type()
                .ok_or(anyhow::anyhow!("Unable to get column type"))?;

            values.push((Alias::new(key.to_string()), val.to_sea_query(col_type)?));
        }

        // Set values to update
        query = query.values(values);

        // Add WHERE clause for primary key
        query = query.and_where(Expr::col(Alias::new(pk_col.get_column_name())).eq(id));

        let query =
            query.returning(Query::returning().column(Alias::new(pk_col.get_column_name())));

        let query = query.to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| {
                serde_json::json!({
                    "name":pk_col.get_column_name(),
                    "id":val
                })
            })?;

        debug!("Update completed for ID: {}", id);
        Ok(Some(Value::from_json(result)?))
    })
}

pub fn delete_resolver(table: SqliteTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing delete resolver for table: {:?}",
            table.table_name()
        );

        let table_name = table.table_name();

        let pk_col = table.primary_key()?;

        let db = ctx.data::<SqlitePool>()?;

        let id = ctx.args.try_get("id")?.i64()?;

        debug!("Delete query for ID: {}", id);

        let query = Query::delete()
            .from_table(table_name)
            .and_where(Expr::col(Alias::new(pk_col.get_column_name())).eq(id))
            .to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query(&query).execute(db).await?;

        debug!(
            "Delete completed, rows affected: {}",
            result.rows_affected()
        );

        Ok(Some(Value::from_json(
            serde_json::json!({"rows_affected":result.rows_affected()}),
        )?))
    })
}
