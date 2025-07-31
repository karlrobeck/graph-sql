use async_graphql::{
    Value,
    dynamic::{FieldFuture, FieldValue, ResolverContext},
};
use sqlx::SqlitePool;

use crate::types::{ColumnInfo, SqliteTable};

pub fn id_resolver<'a>(ctx: &ResolverContext<'_>) -> FieldFuture<'a> {
    FieldFuture::new(async move { Ok(Some(Value::from("resolved_id_value"))) })
}

pub fn list_resolver<'a>(table_info: &SqliteTable, ctx: &ResolverContext<'a>) -> FieldFuture<'a> {
    let db = ctx.data::<SqlitePool>().unwrap();
    let table_name = table_info.table_info.name.clone().to_owned();
    let pk_col = table_info
        .column_info
        .iter()
        .find(|col| col.pk == 1)
        .unwrap()
        .to_owned();

    FieldFuture::new(async move {
        let result = sqlx::query_as::<_, (i64,)>(&format!(
            r#"
              select {} from {};
            "#,
            pk_col.name, table_name
        ))
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|(val,)| async_graphql::Value::Number(val.into()))
        .collect::<Vec<_>>();

        Ok(Some(Value::List(result)))
    })
}

pub fn column_resolver<'a>(
    table_name: String,
    col: &ColumnInfo,
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
            col.name, col.name, table_name
        ))
        .bind(id)
        .fetch_one(&db)
        .await
        .map(|(map_val,)| map_val.as_object().unwrap().clone())
        .map(|val| val.get(&col.name).unwrap().clone())?;

        Ok(Some(Value::from_json(value).unwrap()))
    })
}
