use std::path::Path;

use async_graphql::{
    dynamic::{Field, Object, Schema, TypeRef},
    http::GraphiQLSource,
};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

use crate::{resolvers::list_resolver, types::SqliteTable};

mod resolvers;
mod types;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        let table_obj = table.to_graphql_object();

        let insert_mutation = table.to_graphql_insert_mutation();

        mutation_object = mutation_object.field(insert_mutation.1);

        inputs.push(insert_mutation.0);

        query_object = query_object.field(Field::new(
            table.table_info.name.clone(),
            TypeRef::named_list(table_obj.type_name()),
            move |ctx| list_resolver(&table, &ctx),
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
