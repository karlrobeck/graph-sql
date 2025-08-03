use async_graphql::{
    Value,
    dynamic::{Field, FieldFuture, Object, Schema, TypeRef},
    http::GraphiQLSource,
};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
};
use sea_query::Iden;
use sqlx::SqlitePool;
use tokio::net::TcpListener;

use crate::{
    traits::{ToGraphqlMutations, ToGraphqlNode, ToGraphqlQueries},
    types::SqliteTable,
};

mod resolvers;
mod traits;
mod types;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    let db = SqlitePool::connect("sqlite://:memory:").await?;

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    let tables = SqliteTable::introspect(&db).await?;

    let mut query_object = Object::new("Query");
    let mut mutation_object = Object::new("Mutation");

    let mut table_objects = vec![];
    let mut inputs = vec![];

    for table in tables {
        let table_obj = table.to_node()?;
        let name = table.table_name();

        let insert_mutation = table.to_insert_mutation()?;
        let update_mutation = table.to_update_mutation()?;
        let delete_mutation = table.to_delete_mutation()?;

        let list_query = table.to_list_query()?;
        let view_query = table.to_view_query()?;

        let table_obj = table_obj.field(list_query.1).field(view_query.1);

        mutation_object = mutation_object
            .field(insert_mutation.1)
            .field(update_mutation.1)
            .field(delete_mutation.1);

        inputs.push(insert_mutation.0);
        inputs.push(update_mutation.0);
        inputs.push(delete_mutation.0);
        inputs.push(list_query.0);
        inputs.push(view_query.0);

        query_object = query_object.field(Field::new(
            name.to_string(),
            TypeRef::named_nn(table_obj.type_name()),
            |_| FieldFuture::new(async move { Ok(Some(Value::from(""))) }),
        ));

        table_objects.push(table_obj);
    }

    let mut schema = Schema::build(
        query_object.type_name(),
        Some(mutation_object.type_name()),
        None,
    )
    .register(query_object)
    .register(mutation_object);

    for object in table_objects {
        schema = schema.register(object);
    }

    for input in inputs {
        schema = schema.register(input);
    }

    let schema = schema.data(db).finish()?;

    let router = Router::new().route(
        "/",
        axum::routing::get(graphiql).post_service(GraphQL::new(schema)),
    );

    let listener = TcpListener::bind("0.0.0.0:8000").await?;

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
