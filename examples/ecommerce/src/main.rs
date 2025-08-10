use graph_sql::{GraphSQL, config::load_config};

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    println!("🛒 Starting E-commerce Example Server...");

    // Load configuration from TOML file
    let config = load_config("examples/ecommerce/config.toml")?;

    // Create database connection
    let db = config.database.create_connection().await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./examples/ecommerce/migrations")
        .run(&db)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    println!("🔍 Creating GraphSQL instance and building server...");
    let graph_sql = GraphSQL::new(config);

    let (router, listener) = graph_sql.build(&db).await?;

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

    if let Err(e) = axum::serve(listener, router.into_make_service()).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
