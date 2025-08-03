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
    println!("🛒 Starting E-commerce Example Server...");

    // Use in-memory database for demo purposes
    let db = SqlitePool::connect("sqlite::memory:").await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./examples/ecommerce/migrations")
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

    let listener = TcpListener::bind("0.0.0.0:8081").await?;

    println!("✅ E-commerce GraphQL API is running!");
    println!("🛒 GraphiQL interface: http://localhost:8081");
    println!();
    println!("Example queries to try:");
    println!(
        "  - Browse products: {{ product {{ list(input: {{page: 1, limit: 5}}) {{ name, price, category {{ name }} }} }} }}"
    );
    println!(
        "  - View orders: {{ order {{ list(input: {{page: 1, limit: 3}}) {{ total_amount, customer {{ name }}, order_item {{ list(input: {{page: 1, limit: 10}}) {{ quantity, product {{ name }} }} }} }} }} }}"
    );
    println!(
        "  - Customer details: {{ customer {{ view(input: {{id: 1}}) {{ name, email, address {{ street, city }} }} }} }}"
    );
    println!();

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
