use graph_sql::{GraphSQL, config::load_config};

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    println!("📝 Starting Task Manager Example Server...");

    // Load configuration from TOML file
    let config = load_config("examples/tasks/config.toml")?;

    // Create database connection
    let db = config.database.create_connection().await?;

    println!("📊 Running database migrations...");
    sqlx::migrate!("./examples/tasks/migrations")
        .run(&db)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    println!("🔍 Creating GraphSQL instance and building server...");
    let graph_sql = GraphSQL::new(config);

    let (router, listener) = graph_sql.build(&db).await?;

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

    if let Err(e) = axum::serve(listener, router.into_make_service()).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
