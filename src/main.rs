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
use tracing::{debug, error, info};

mod cli;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

async fn ensure_database_exists(database_url: &str) -> anyhow::Result<()> {
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
    }
    Ok(())
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let config =
        load_config(&cli.config).map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    match cli.command {
        Commands::Serve => serve_command(config).await,
        Commands::Introspect { output } => introspect_command(config, output).await,
    }
}

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

    let graphql_config = config.graphql.unwrap_or_default();

    let schema = graph_sql::introspect(&db)
        .await?
        .limit_complexity(graphql_config.complexity as usize)
        .limit_depth(graphql_config.depth as usize)
        .finish()?;

    let mut router = Router::new();

    if graphql_config.enable_playground {
        router = router.route(
            "/",
            axum::routing::get(graphiql).post_service(GraphQL::new(schema)),
        );
    }

    let bind_address = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&bind_address).await?;

    info!("GraphQL server running at http://{}", bind_address);
    info!("GraphiQL interface available at http://{}", bind_address);

    if let Err(e) = axum::serve(listener, router).await {
        error!("Server error: {}", e);
    }

    Ok(())
}

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
