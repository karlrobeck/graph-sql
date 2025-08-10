use graph_sql::{GraphSQL, config::load_config};

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    println!("🚀 Starting Blog Example Server...");

    // Load configuration from TOML file
    let config = load_config("examples/blog/config.toml")?;

    // Create database connection
    let db = config.database.create_connection().await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./examples/blog/migrations")
        .run(&db)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    println!("🔍 Creating GraphSQL instance and building server...");
    let graph_sql = GraphSQL::new(config);

    let (router, listener) = graph_sql.build(&db).await?;

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

    if let Err(e) = axum::serve(listener, router.into_make_service()).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
