use crate::cli::{Cli, Commands, load_config};
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
};
use clap::Parser;
use graph_sql::{GraphSQL, config::GraphSQLConfig};
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
async fn serve_command(config: GraphSQLConfig) -> async_graphql::Result<()> {
    let pool = config.database.create_connection().await?;

    let graph_sql = GraphSQL::new(config);

    let (router, listener) = graph_sql.build(&pool).await?;

    if let Err(err) = axum::serve(listener, router.into_make_service()).await {
        error!("{}", err)
    }

    Ok(())
}

#[instrument(skip(config), level = "info")]
async fn introspect_command(
    config: GraphSQLConfig,
    output: Option<String>,
) -> async_graphql::Result<()> {
    let pool = config.database.create_connection().await?;

    let graph_sql = GraphSQL::new(config);

    let tables = graph_sql.introspect(&pool).await?;

    let schema = graph_sql.build_schema(tables)?.finish()?;
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
