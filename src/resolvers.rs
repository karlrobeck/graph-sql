use anyhow::anyhow;
use async_graphql::{
    Value,
    dataloader::DataLoader,
    dynamic::{FieldFuture, ResolverContext},
};
use base64::{Engine as _, engine::general_purpose};
use sea_query::{Alias, ColumnDef, Expr, Query, SqliteQueryBuilder};
use sqlparser::ast::{ColumnOption, CreateTable};
use sqlx::SqlitePool;
use tracing::debug;

use crate::{
    loader::{ColumnRowDef, ColumnRowLoader},
    traits::ToSimpleExpr,
};

pub fn list_resolver_ext(table_info: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!("Executing list resolver for table: {:?}", table_info.name);

        let db = ctx.data::<SqlitePool>()?;

        let table_name = table_info.name.to_string();

        let pk_col = table_info
            .columns
            .iter()
            .find(|spec| {
                spec.options.iter().any(|spec| {
                    if let ColumnOption::Unique {
                        is_primary,
                        characteristics: _,
                    } = spec.option
                    {
                        is_primary
                    } else {
                        false
                    }
                })
            })
            .ok_or(anyhow!("Unable to find primary key"))?;

        let input = ctx.args.try_get("input")?.object()?;

        let page = input.try_get("page")?.u64()?;
        let limit = input.try_get("limit")?.u64()?;

        debug!("List query parameters - page: {}, limit: {}", page, limit);

        let query = Query::select()
            .from(Alias::new(table_name))
            .column(Alias::new(pk_col.name.to_string()))
            .offset((page - 1) * limit)
            .limit(limit)
            .to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_all(db)
            .await
            .map_err(|e| {
                debug!("Database query failed: {}", e);
                e
            })?
            .into_iter()
            .map(|(val,)| {
                serde_json::json!({
                  "name":pk_col.name.to_string(),
                  "id":val,
                })
            })
            .map(|val| Value::from_json(val).unwrap())
            .collect::<Vec<_>>();

        debug!("List resolver returned {} items", result.len());
        Ok(Some(Value::List(result)))
    })
}

pub fn view_resolver_ext(table_info: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!("Executing view resolver for table: {:?}", table_info.name);

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

        let table_name = table_info.name;

        let pk_col = table_info
            .columns
            .iter()
            .find(|spec| {
                spec.options.iter().any(|spec| {
                    if let ColumnOption::Unique {
                        is_primary,
                        characteristics: _,
                    } = spec.option
                    {
                        is_primary
                    } else {
                        false
                    }
                })
            })
            .ok_or(anyhow!("Unable to find primary key"))?;

        let query = Query::select()
            .from(Alias::new(table_name.to_string()))
            .column(Alias::new(pk_col.name.to_string()))
            .and_where(Expr::col(Alias::new(pk_col.name.to_string())).eq(id))
            .to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map_err(|e| {
                debug!("Database query failed: {}", e);
                e
            })
            .map(|(val,)| {
                serde_json::json!({
                  "name":pk_col.name.to_string(),
                  "id":val,
                })
            })
            .map(|val| Value::from_json(val).unwrap())?;

        debug!("View resolver found record with ID: {}", id);
        Ok(Some(result))
    })
}

pub fn foreign_key_resolver_ext(
    table_name: String,
    foreign_table: String,
    reffered_column: String,
    col: sqlparser::ast::ColumnDef,
    ctx: ResolverContext<'_>,
) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing foreign key resolver for table: {} -> {}",
            table_name, foreign_table
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
            .from_as(Alias::new(table_name.clone()), Alias::new("f"))
            .expr(Expr::cust_with_values(
                format!("json_object(?,f.{})", reffered_column),
                [reffered_column.clone()],
            ))
            .inner_join(
                Alias::new(table_name.clone()),
                Expr::col((
                    Alias::new(table_name.clone()),
                    Alias::new(col.name.to_string()),
                ))
                .equals((Alias::new("f"), Alias::new(reffered_column.clone()))),
            )
            .and_where(Expr::col((Alias::new(table_name.clone()), Alias::new(pk_name))).eq(pk_id))
            .to_string(SqliteQueryBuilder);

        let result = sqlx::query_as::<_, (serde_json::Value,)>(&query)
            .fetch_one(db)
            .await
            .map(|(map_val,)| map_val.as_object().unwrap().clone())
            .map(|val| {
                serde_json::json!({
                    "name":reffered_column,
                    "id":val.get(&reffered_column).unwrap()
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

pub fn column_resolver_ext(
    table_name: String,
    col: sqlparser::ast::ColumnDef,
    ctx: ResolverContext<'_>,
) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!(
            "Executing column resolver for table: {} column: {}",
            table_name,
            col.name.to_string()
        );

        let loader = ctx.data::<DataLoader<ColumnRowLoader>>()?;

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

        let result = loader
            .load_one(ColumnRowDef {
                column: Alias::new(col.name.to_string()),
                primary_column: Alias::new(pk_name),
                table: Alias::new(table_name),
                value: pk_id,
            })
            .await?
            .map(|val| {
                // Try to convert Vec<u8> to string first (for text/numeric data)
                // If that fails, fall back to base64 (for binary data)
                match String::from_utf8(val.clone()) {
                    Ok(string_val) => {
                        // Successfully converted to string, try to parse as JSON value
                        // First try to parse as number (integer or float)
                        if let Ok(int_val) = string_val.parse::<i64>() {
                            Value::from_json(serde_json::Value::Number(serde_json::Number::from(
                                int_val,
                            )))
                            .map_err(|e| {
                                anyhow!("Unable to convert integer to GraphQL value: {}", e)
                            })
                        } else if let Ok(float_val) = string_val.parse::<f64>() {
                            if let Some(num) = serde_json::Number::from_f64(float_val) {
                                Value::from_json(serde_json::Value::Number(num)).map_err(|e| {
                                    anyhow!("Unable to convert float to GraphQL value: {}", e)
                                })
                            } else {
                                Value::from_json(serde_json::Value::String(string_val)).map_err(
                                    |e| anyhow!("Unable to convert string to GraphQL value: {}", e),
                                )
                            }
                        } else if let Ok(bool_val) = string_val.parse::<bool>() {
                            Value::from_json(serde_json::Value::Bool(bool_val)).map_err(|e| {
                                anyhow!("Unable to convert boolean to GraphQL value: {}", e)
                            })
                        } else {
                            // Just use as string
                            Value::from_json(serde_json::Value::String(string_val)).map_err(|e| {
                                anyhow!("Unable to convert string to GraphQL value: {}", e)
                            })
                        }
                    }
                    Err(_) => {
                        // Not valid UTF-8, likely binary data - encode as base64
                        let base64_string = general_purpose::STANDARD.encode(&val);
                        Value::from_json(serde_json::Value::String(base64_string)).map_err(|e| {
                            anyhow!("Unable to convert base64 string to GraphQL value: {}", e)
                        })
                    }
                }
            })
            .transpose()?
            .ok_or(anyhow!("No value found for column"))?;

        Ok(Some(result))
    })
}

pub fn insert_resolver_ext(table: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!("Executing insert resolver for table: {:?}", table.name);

        let db = ctx.data::<SqlitePool>()?;
        let table_name = table.name.to_string();

        let input = ctx.args.try_get("input")?;

        let input = input.object()?;

        debug!("Insert data: {} fields", input.len());

        let mut binding = Query::insert();

        let pk_col = table
            .columns
            .iter()
            .find(|spec| {
                spec.options.iter().any(|spec| {
                    if let ColumnOption::Unique {
                        is_primary,
                        characteristics: _,
                    } = spec.option
                    {
                        is_primary
                    } else {
                        false
                    }
                })
            })
            .ok_or(anyhow!("Unable to find primary key"))?;

        let query = binding
            .into_table(Alias::new(table_name))
            .columns(input.iter().map(|(name, _)| Alias::new(name.to_string())));

        let mut values = vec![];

        for (key, val) in input.iter() {
            debug!("Processing field: {}", key);

            let col_type = &table
                .columns
                .iter()
                .find(|col| col.name.to_string() == *key)
                .ok_or(anyhow::anyhow!("Unable to get column"))?
                .data_type;

            values.push(val.to_simple_expr(col_type)?);
        }

        let query = query.returning(Query::returning().column(Alias::new(pk_col.name.to_string())));

        let query = query.values(values)?.to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map_err(|e| {
                debug!("Insert query failed: {}", e);
                anyhow::anyhow!("Insert operation failed: {}", e)
            })
            .map(|(val,)| {
                serde_json::json!({
                    "name": pk_col.name.to_string(),
                    "id": val
                })
            })?;

        debug!("Insert completed, new ID: {:?}", result);

        Ok(Some(Value::from_json(result)?))
    })
}

pub fn update_resolver_ext(table: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!("Executing update resolver for table: {:?}", table.name);

        let table_name = table.name.to_string();

        let pk_col = table
            .columns
            .iter()
            .find(|spec| {
                spec.options.iter().any(|spec| {
                    if let ColumnOption::Unique {
                        is_primary,
                        characteristics: _,
                    } = spec.option
                    {
                        is_primary
                    } else {
                        false
                    }
                })
            })
            .ok_or(anyhow!("Unable to find primary key"))?;

        let db = ctx.data::<SqlitePool>()?;

        let id = ctx.args.try_get("id")?.i64()?;

        debug!("Update query for ID: {}", id);

        let input = ctx.args.try_get("input")?.object()?;

        debug!("Update data: {} fields", input.len());

        let mut binding = Query::update();

        // Build the update query
        let mut query = binding.table(Alias::new(table_name));

        // Collect columns and values to update
        let mut values = vec![];

        for (key, val) in input.iter() {
            debug!("Processing field: {}", key);

            let col_type = &table
                .columns
                .iter()
                .find(|col| col.name.to_string() == *key)
                .ok_or(anyhow::anyhow!("Unable to get column"))?
                .data_type;

            values.push((Alias::new(key.to_string()), val.to_simple_expr(col_type)?));
        }

        // Set values to update
        query = query.values(values);

        // Add WHERE clause for primary key
        query = query.and_where(Expr::col(Alias::new(pk_col.name.to_string())).eq(id));

        let query = query.returning(Query::returning().column(Alias::new(pk_col.name.to_string())));

        let query = query.to_string(SqliteQueryBuilder);

        debug!("Generated SQL query: {}", query);

        let result = sqlx::query_as::<_, (i64,)>(&query)
            .fetch_one(db)
            .await
            .map(|(val,)| {
                serde_json::json!({
                    "name":pk_col.name.to_string(),
                    "id":val
                })
            })?;

        debug!("Update completed for ID: {}", id);
        Ok(Some(Value::from_json(result)?))
    })
}

pub fn delete_resolver_ext(table: CreateTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    FieldFuture::new(async move {
        debug!("Executing delete resolver for table: {:?}", table.name);

        let table_name = table.name.to_string();

        let pk_col = table
            .columns
            .iter()
            .find(|spec| {
                spec.options.iter().any(|spec| {
                    if let ColumnOption::Unique {
                        is_primary,
                        characteristics: _,
                    } = spec.option
                    {
                        is_primary
                    } else {
                        false
                    }
                })
            })
            .ok_or(anyhow!("Unable to find primary key"))?;

        let db = ctx.data::<SqlitePool>()?;

        let id = ctx.args.try_get("id")?.i64()?;

        debug!("Delete query for ID: {}", id);

        let query = Query::delete()
            .from_table(Alias::new(table_name))
            .and_where(Expr::col(Alias::new(pk_col.name.to_string())).eq(id))
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
