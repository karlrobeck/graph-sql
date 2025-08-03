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
    println!("🚀 Starting Blog Example Server...");

    // Use in-memory database for demo purposes
    let db = SqlitePool::connect("sqlite::memory:").await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./migrations")
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

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    println!("✅ Blog GraphQL API is running!");
    println!("📝 GraphiQL interface: http://localhost:8080");
    println!();
    println!("Example queries to try:");
    println!(
        "  - List all posts: {{ post {{ list(input: {{page: 1, limit: 5}}) {{ title, author {{ name }} }} }} }}"
    );
    println!(
        "  - View post with comments: {{ post {{ view(input: {{id: 1}}) {{ title, content, comments {{ list(input: {{page: 1, limit: 10}}) {{ content, author {{ name }} }} }} }} }} }}"
    );
    println!();

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
