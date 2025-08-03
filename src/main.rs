use std::path::Path;

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
    traits::{ToGraphqlMutations, ToGraphqlNode, ToGraphqlObject, ToGraphqlQueries},
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
        let name = table.table_name();

        let (query, mutations, mutation_inputs) = table.to_object()?;

        // add query
        query_object = query_object.field(Field::new(
            name.to_string(),
            TypeRef::named_nn(query.type_name()),
            |_| FieldFuture::new(async move { Ok(Some(Value::from(""))) }),
        ));

        // add mutations
        for mutation in mutations.into_iter() {
            mutation_object = mutation_object.field(mutation);
        }

        // register types
        table_objects.push(query);
        inputs.extend(mutation_inputs);
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
