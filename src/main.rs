use crate::cli::{Cli, Commands, Config, load_config};
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
};
use clap::Parser;
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
use tokio::net::TcpListener;
use tracing::{debug, error, info, instrument};

mod cli;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

#[instrument(skip_all, level = "debug")]
async fn ensure_database_exists(database_url: &str) -> anyhow::Result<()> {
    debug!("Checking if database exists: {}", database_url);
    // Check if it's a SQLite file URL (not in-memory)
    if database_url.starts_with("sqlite://") && !database_url.contains(":memory:") {
        // Check if the database already exists
        if !Sqlite::database_exists(database_url).await? {
            info!("Creating database {}", database_url);
            Sqlite::create_database(database_url).await?;
            info!("Database created successfully");
        } else {
            debug!("Database already exists");
        }
    } else {
        debug!("Using in-memory database or non-file URL");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Initialize tracing with more detailed configuration
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,graph_sql=debug")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting graph-sql application");

    let cli = Cli::parse();
    debug!("Parsed CLI arguments: {:?}", cli);

    let config =
        load_config(&cli.config).map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    debug!("Configuration loaded successfully");

    match cli.command {
        Commands::Serve => serve_command(config).await,
        Commands::Introspect { output } => introspect_command(config, output).await,
    }
}

#[instrument(skip(config), level = "info")]
async fn serve_command(config: Config) -> async_graphql::Result<()> {
    info!("Starting GraphQL server...");
    debug!("Database URL: {}", config.database.database_url);
    info!(
        "Server address: {}:{}",
        config.server.host, config.server.port
    );

    // Ensure database file exists if it's a file-based SQLite database
    ensure_database_exists(&config.database.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to ensure database exists: {}", e))?;

    debug!("Connecting to database...");
    let db = SqlitePool::connect(&config.database.database_url).await?;
    info!("Successfully connected to database");

    // Run migrations if path is provided
    if let Some(migrations_path) = &config.database.migrations_path {
        if !migrations_path.is_empty() {
            info!("Running migrations from: {}", migrations_path);
            debug!("Migration path: {}", migrations_path);
            sqlx::migrate::Migrator::new(std::path::Path::new(migrations_path))
                .await?
                .run(&db)
                .await
                .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
            info!("Migrations completed successfully");
        }
    }

    let graphql_config = config.graphql.unwrap_or_default();
    debug!(
        "GraphQL config: complexity={:?}, depth={:?}, playground={}",
        graphql_config.complexity, graphql_config.depth, graphql_config.enable_playground
    );

    debug!("Starting database introspection for schema building...");
    let mut schema = graph_sql::introspect(&db).await?;

    if let Some(complexity) = graphql_config.complexity {
        debug!("Setting query complexity limit to: {}", complexity);
        schema = schema.limit_complexity(complexity as usize)
    }

    if let Some(depth) = graphql_config.depth {
        debug!("Setting query depth limit to: {}", depth);
        schema = schema.limit_depth(depth as usize);
    }

    debug!("Finalizing GraphQL schema...");
    let schema = schema.finish()?;
    info!("GraphQL schema built successfully");

    let mut router = Router::new();

    if graphql_config.enable_playground {
        debug!("GraphiQL playground enabled");
        router = router.route(
            "/",
            axum::routing::get(graphiql).post_service(GraphQL::new(schema)),
        );
    } else {
        debug!("GraphiQL playground disabled");
        router = router.route("/", axum::routing::post_service(GraphQL::new(schema)));
    }

    let bind_address = format!("{}:{}", config.server.host, config.server.port);
    debug!("Binding server to address: {}", bind_address);
    let listener = TcpListener::bind(&bind_address).await?;

    info!("GraphQL server running at http://{}", bind_address);
    info!("GraphiQL interface available at http://{}", bind_address);

    if let Err(e) = axum::serve(listener, router).await {
        error!("Server error: {}", e);
    }

    Ok(())
}

#[instrument(skip(config), level = "info")]
async fn introspect_command(config: Config, output: Option<String>) -> async_graphql::Result<()> {
    info!("Introspecting database schema...");
    debug!("Database URL: {}", config.database.database_url);

    // Ensure database file exists if it's a file-based SQLite database
    ensure_database_exists(&config.database.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to ensure database exists: {}", e))?;

    let db = SqlitePool::connect(&config.database.database_url).await?;

    // Run migrations if path is provided
    if let Some(migrations_path) = &config.database.migrations_path {
        if !migrations_path.is_empty() {
            info!("Running migrations from: {}", migrations_path);
            sqlx::migrate::Migrator::new(std::path::Path::new(migrations_path))
                .await?
                .run(&db)
                .await
                .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
        }
    }

    let schema = graph_sql::introspect(&db).await?.finish()?;
    let sdl = schema.sdl();

    match output {
        Some(file_path) => {
            std::fs::write(&file_path, &sdl)
                .map_err(|e| anyhow::anyhow!("Failed to write to file {}: {}", file_path, e))?;
            info!("GraphQL schema written to: {}", file_path);
        }
        None => {
            println!("{}", sdl);
        }
    }

    Ok(())
}
