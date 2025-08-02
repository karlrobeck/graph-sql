use async_graphql::{
    Value,
    dynamic::{FieldFuture, ResolverContext},
};
use sea_query::{
    Alias, ColumnDef, ColumnSpec, ConditionalStatement, Expr, Query, SqliteQueryBuilder,
};
use sqlx::SqlitePool;

use crate::types::{SqliteTable, ToSeaQueryValue};

pub fn list_resolver<'a>(table_info: SqliteTable, ctx: ResolverContext<'a>) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let db = ctx.data::<SqlitePool>()?;
        let table_name = table_info.table_name();
        let pk_col = table_info.primary_key()?;

        let query = Query::select()
            .from(table_name)
            .column(Alias::new(pk_col.get_column_name()))
            .to_string(SqliteQueryBuilder);

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

        Ok(Some(Value::List(result)))
    })
}

pub fn view_resolver<'a>(table_info: SqliteTable, ctx: ResolverContext<'a>) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let db = ctx.data::<SqlitePool>()?;

        let id = ctx
            .args
            .get("id")
            .ok_or(anyhow::anyhow!("Unable to get id"))?
            .i64()?;

        let table_name = table_info.table_name();

        let pk_col = table_info.primary_key()?;

        let query = Query::select()
            .from(table_name)
            .column(Alias::new(pk_col.get_column_name()))
            .and_where(Expr::col(Alias::new(pk_col.get_column_name())).eq(id))
            .to_string(SqliteQueryBuilder);

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

        Ok(Some(result))
    })
}

pub fn column_resolver<'a>(
    table_name: String,
    col: ColumnDef,
    ctx: ResolverContext<'a>,
) -> FieldFuture<'a> {
    FieldFuture::new(async move {
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
            .map(|val| Value::from_json(val))?;

        Ok(Some(result?))
    })
}

pub fn insert_resolver<'a>(table: SqliteTable, ctx: ResolverContext<'a>) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let db = ctx.data::<SqlitePool>()?;
        let table_name = table.table_name();

        let input = ctx.args.try_get("input")?;

        let input = input.object()?;

        let mut binding = Query::insert();

        let pkey_col = table
            .column_info
            .iter()
            .find(|col| {
                col.get_column_spec()
                    .iter()
                    .find(|spec| matches!(spec, ColumnSpec::PrimaryKey))
                    .is_some()
            })
            .unwrap();

        let query = binding
            .into_table(table_name)
            .columns(input.iter().map(|(name, _)| Alias::new(name.to_string())));

        let mut values = vec![];

        for (key, val) in input.iter() {
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

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| {
                serde_json::json!({
                    "name": pkey_col.get_column_name(),
                    "id": val
                })
            })?;

        Ok(Some(Value::from_json(result)?))
    })
}

pub fn update_resolver<'a>(table: SqliteTable, ctx: ResolverContext<'a>) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let table_name = table.table_name();

        let pk_col = table.primary_key()?;

        let db = ctx.data::<SqlitePool>()?;

        let id = ctx.args.try_get("id")?.i64()?;

        let input = ctx.args.try_get("input")?.object()?;

        let mut binding = Query::update();

        // Build the update query
        let mut query = binding.table(table_name);

        // Collect columns and values to update
        let mut values = vec![];

        for (key, val) in input.iter() {
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

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| {
                serde_json::json!({
                    "name":pk_col.get_column_name(),
                    "id":val
                })
            })?;

        Ok(Some(Value::from_json(result)?))
    })
}
