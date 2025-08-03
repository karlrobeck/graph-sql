use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    println!("📚 Starting Library Management Example Server...");

    // Use in-memory database for demo purposes
    let db = SqlitePool::connect("sqlite::memory:").await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./examples/library/migrations")
        .run(&db)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    println!("🔍 Introspecting database schema...");
    let schema = graph_sql::introspect(&db).await?.finish()?;

    println!("🌐 Setting up GraphQL server...");
    let router = Router::new().route(
        "/",
        axum::routing::get(graphiql).post_service(GraphQL::new(schema)),
    );

    let listener = TcpListener::bind("0.0.0.0:8083").await?;

    println!("✅ Library Management GraphQL API is running!");
    println!("📚 GraphiQL interface: http://localhost:8083");
    println!();
    println!("Example operations to try:");
    println!(
        "  - Browse books: {{ book {{ list(input: {{page: 1, limit: 5}}) {{ title, author {{ name }}, genre {{ name }} }} }} }}"
    );
    println!(
        "  - Check loans: {{ loan {{ list(input: {{page: 1, limit: 5}}) {{ book {{ title }}, member {{ name }}, due_date }} }} }}"
    );
    println!(
        "  - Search authors: {{ author {{ list(input: {{page: 1, limit: 10}}) {{ name, birth_year, nationality }} }} }}"
    );
    println!();

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
