use async_graphql::{
    Value,
    dynamic::{FieldFuture, ResolverContext},
};
use sea_query::{
    Alias, ColumnDef, ColumnSpec, ColumnType, Expr, Query, QueryStatementWriter, SimpleExpr,
    SqliteQueryBuilder,
};
use sqlx::SqlitePool;

use crate::types::SqliteTable;

pub fn list_resolver<'a>(table_info: SqliteTable, ctx: ResolverContext<'a>) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let db = ctx.data::<SqlitePool>()?;
        let table_name = table_info.table_info.name.clone().to_owned();
        let pk_col = table_info
            .column_info
            .iter()
            .find(|col| {
                col.get_column_spec()
                    .iter()
                    .find(|spec| matches!(spec, sea_query::ColumnSpec::PrimaryKey))
                    .is_some()
            })
            .ok_or(anyhow::anyhow!("Unable to get primary key"))?;

        let query = Query::select()
            .from(Alias::new(table_name))
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

pub fn column_resolver<'a>(
    table_name: String,
    col: ColumnDef,
    ctx: ResolverContext<'a>,
) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let db = ctx.data::<SqlitePool>()?;

        let col = col.to_owned();

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

        println!("{}", query);

        let result = sqlx::query_as::<_, (serde_json::Value,)>(&query)
            .fetch_one(db)
            .await
            .map(|(map_val,)| map_val.as_object().unwrap().clone())
            .map(|val| val.get(&col.get_column_name()).unwrap().clone())
            .map(|val| Value::from_json(val))??;

        Ok(Some(result))
    })
}

pub fn insert_resolver<'a>(table: SqliteTable, ctx: ResolverContext<'a>) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let db = ctx.data::<SqlitePool>()?;

        let input = ctx.args.try_get("input")?;

        let input = input.object()?;

        let mut binding = Query::insert();

        let query = binding
            .into_table(Alias::new(table.table_info.name))
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

            let val = match col_type {
                ColumnType::Text => val.string().map(|val| Into::<SimpleExpr>::into(val)),
                ColumnType::Integer => val.i64().map(|val| Into::<SimpleExpr>::into(val)),
                ColumnType::Boolean => val.boolean().map(|val| Into::<SimpleExpr>::into(val)),
                ColumnType::Float => val.f64().map(|val| Into::<SimpleExpr>::into(val)),
                _ => val.string().map(|val| Into::<SimpleExpr>::into(val)),
            };

            values.push(val?);
        }

        let query = query.returning(
            Query::returning().column(
                table
                    .column_info
                    .iter()
                    .find(|col| {
                        col.get_column_spec()
                            .iter()
                            .find(|spec| matches!(spec, ColumnSpec::PrimaryKey))
                            .is_some()
                    })
                    .map(|col| Alias::new(col.get_column_name()))
                    .unwrap(),
            ),
        );

        let query = query.values(values)?.to_string(SqliteQueryBuilder);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| val.into())?;

        Ok(Some(Value::Number(result)))
    })
}

pub fn update_resolver<'a>(table: SqliteTable, ctx: ResolverContext<'a>) -> FieldFuture<'a> {
    FieldFuture::new(async move {
        let db = ctx.data::<SqlitePool>()?;

        let id = ctx.args.try_get("id")?.i64()?;

        let input = ctx.args.try_get("input")?;

        let input = input.object()?;

        let mut binding = Query::update();

        // Find the primary key column name
        let pk_col = table
            .column_info
            .iter()
            .find(|col| {
                col.get_column_spec()
                    .iter()
                    .find(|spec| matches!(spec, ColumnSpec::PrimaryKey))
                    .is_some()
            })
            .ok_or(anyhow::anyhow!("Unable to get primary key"))?;

        // Build the update query
        let mut query = binding.table(Alias::new(table.table_info.name));

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

            let val = match col_type {
                ColumnType::Text => val.string().map(|val| Into::<SimpleExpr>::into(val)),
                ColumnType::Integer => val.i64().map(|val| Into::<SimpleExpr>::into(val)),
                ColumnType::Boolean => val.boolean().map(|val| Into::<SimpleExpr>::into(val)),
                ColumnType::Float => val.f64().map(|val| Into::<SimpleExpr>::into(val)),
                _ => val.string().map(|val| Into::<SimpleExpr>::into(val)),
            };

            values.push((Alias::new(key.to_string()), val?));
        }

        // Set values to update
        query = query.values(values);

        // Add WHERE clause for primary key
        query = query.and_where(Expr::col(Alias::new(pk_col.get_column_name())).eq(id));

        let query = query.returning(
            Query::returning().column(
                table
                    .column_info
                    .iter()
                    .find(|col| {
                        col.get_column_spec()
                            .iter()
                            .find(|spec| matches!(spec, ColumnSpec::PrimaryKey))
                            .is_some()
                    })
                    .map(|col| Alias::new(col.get_column_name()))
                    .unwrap(),
            ),
        );

        let query = query.to_string(SqliteQueryBuilder);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| val.into())?;

        Ok(Some(Value::Number(result)))
    })
}
