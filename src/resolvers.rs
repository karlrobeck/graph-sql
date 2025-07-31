use async_graphql::{
    Value,
    dynamic::{FieldFuture, ResolverContext},
};
use sea_query::{
    Alias, ColumnDef, ColumnSpec, ColumnType, Expr, Func, Query, QueryStatementWriter, SimpleExpr,
    SqliteQueryBuilder,
};
use sqlx::SqlitePool;

use crate::types::{SqliteTable, ToGraphQL};

pub fn list_resolver<'a>(table_info: &SqliteTable, ctx: &ResolverContext<'a>) -> FieldFuture<'a> {
    let db = ctx.data::<SqlitePool>().unwrap();
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
        .unwrap()
        .to_owned();

    FieldFuture::new(async move {
        let result = sqlx::query_as::<_, (i64,)>(&format!(
            r#"
              select {} from {};
            "#,
            pk_col.get_column_name(),
            table_name
        ))
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|(val,)| val.into())
        .collect::<Vec<_>>();

        Ok(Some(Value::List(result)))
    })
}

pub fn column_resolver<'a>(
    table_name: String,
    col: &ColumnDef,
    ctx: &ResolverContext<'a>,
) -> FieldFuture<'a> {
    let col = col.to_owned();

    let db = ctx.data::<SqlitePool>().unwrap().to_owned();

    let val = ctx.parent_value.as_value().unwrap().clone();

    let val = val.into_json().unwrap();

    let id = val.as_i64().unwrap();

    FieldFuture::new(async move {
        let value = sqlx::query_as::<_, (serde_json::Value,)>(&format!(
            "select json_object('{}',{}) from {} where id = ?",
            col.get_column_name(),
            col.get_column_name(),
            table_name
        ))
        .bind(id)
        .fetch_one(&db)
        .await
        .map(|(map_val,)| map_val.as_object().unwrap().clone())
        .map(|val| val.get(&col.get_column_name()).unwrap().clone())?;

        Ok(Some(Value::from_json(value)?))
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
                .unwrap()
                .get_column_type()
                .unwrap();

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
