use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
};
use clap::{Parser, Subcommand};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(version, about = "A GraphQL server for SQL databases", long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the GraphQL server
    Serve {
        /// Database connection URL
        #[arg(short, long, env = "DATABASE_URL", default_value = "sqlite://local.db")]
        database_url: String,
        /// Server host address
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Server port
        #[arg(short, long, default_value = "8000")]
        port: u16,
        /// Path to migrations directory (optional)
        #[arg(short, long)]
        migrations: Option<String>,
    },
    /// Introspect the database schema and output GraphQL schema
    Introspect {
        /// Database connection URL
        #[arg(short, long, env = "DATABASE_URL", default_value = "sqlite://local.db")]
        database_url: String,
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
        /// Path to migrations directory (optional)
        #[arg(short, long)]
        migrations: Option<String>,
    },
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

async fn ensure_database_exists(database_url: &str) -> anyhow::Result<()> {
    // Check if it's a SQLite file URL (not in-memory)
    if database_url.starts_with("sqlite://") && !database_url.contains(":memory:") {
        // Check if the database already exists
        if !Sqlite::database_exists(database_url).await? {
            println!("Creating database {}", database_url);
            Sqlite::create_database(database_url).await?;
            println!("Database created successfully.");
        } else {
            println!("Database already exists.");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Serve {
            database_url,
            host,
            port,
            migrations,
        } => serve_command(database_url, host, port, migrations).await,
        Commands::Introspect {
            database_url,
            output,
            migrations,
        } => introspect_command(database_url, output, migrations).await,
    }
}

async fn serve_command(
    database_url: String,
    host: String,
    port: u16,
    migrations: Option<String>,
) -> async_graphql::Result<()> {
    println!("Starting GraphQL server...");
    println!("Database URL: {}", database_url);
    println!("Server address: {}:{}", host, port);

    // Ensure database file exists if it's a file-based SQLite database
    ensure_database_exists(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to ensure database exists: {}", e))?;

    let db = SqlitePool::connect(&database_url).await?;

    // Run migrations if path is provided
    if let Some(migrations_path) = migrations {
        println!("Running migrations from: {}", migrations_path);
        sqlx::migrate::Migrator::new(std::path::Path::new(&migrations_path))
            .await?
            .run(&db)
            .await
            .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
    }

    let schema = graph_sql::introspect(&db).await?.finish()?;

    let router = Router::new().route(
        "/",
        axum::routing::get(graphiql).post_service(GraphQL::new(schema)),
    );

    let bind_address = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&bind_address).await?;

    println!("GraphQL server running at http://{}", bind_address);
    println!("GraphiQL interface available at http://{}", bind_address);

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}

async fn introspect_command(
    database_url: String,
    output: Option<String>,
    migrations: Option<String>,
) -> async_graphql::Result<()> {
    println!("Introspecting database schema...");
    println!("Database URL: {}", database_url);

    // Ensure database file exists if it's a file-based SQLite database
    ensure_database_exists(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to ensure database exists: {}", e))?;

    let db = SqlitePool::connect(&database_url).await?;

    // Run migrations if path is provided
    if let Some(migrations_path) = migrations {
        println!("Running migrations from: {}", migrations_path);
        sqlx::migrate::Migrator::new(std::path::Path::new(&migrations_path))
            .await?
            .run(&db)
            .await
            .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
    }

    let schema = graph_sql::introspect(&db).await?.finish()?;
    let sdl = schema.sdl();

    match output {
        Some(file_path) => {
            std::fs::write(&file_path, &sdl)
                .map_err(|e| anyhow::anyhow!("Failed to write to file {}: {}", file_path, e))?;
            println!("GraphQL schema written to: {}", file_path);
        }
        None => {
            println!("{}", sdl);
        }
    }

    Ok(())
}
