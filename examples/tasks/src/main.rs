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
    println!("📝 Starting Task Manager Example Server...");

    // Use in-memory database for demo purposes
    let db = SqlitePool::connect("sqlite::memory:").await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./examples/tasks/migrations")
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

    let listener = TcpListener::bind("0.0.0.0:8082").await?;

    println!("✅ Task Manager GraphQL API is running!");
    println!("📝 GraphiQL interface: http://localhost:8082");
    println!();
    println!("Example operations to try:");
    println!(
        "  - List tasks: {{ task {{ list(input: {{page: 1, limit: 10}}) {{ title, is_completed, priority }} }} }}"
    );
    println!(
        "  - Create task: mutation {{ insert_task(input: {{title: \"New Task\", description: \"Task description\", priority: \"high\"}}) {{ id, title }} }}"
    );
    println!(
        "  - Complete task: mutation {{ update_task(id: 1, input: {{is_completed: true}}) {{ title, is_completed }} }}"
    );
    println!();

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
