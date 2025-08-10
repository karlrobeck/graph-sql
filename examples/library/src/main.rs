use graph_sql::{GraphSQL, config::load_config};

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    println!("📚 Starting Library Management Example Server...");

    // Load configuration from TOML file
    let config = load_config("examples/library/config.toml")?;

    // Create database connection
    let db = config.database.create_connection().await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./examples/library/migrations")
        .run(&db)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    println!("🔍 Creating GraphSQL instance and building server...");
    let graph_sql = GraphSQL::new(config);

    let (router, listener) = graph_sql.build(&db).await?;

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

    if let Err(e) = axum::serve(listener, router.into_make_service()).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
