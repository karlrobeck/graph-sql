use anyhow::anyhow;
use clap::{Parser, Subcommand};
use graph_sql::{GraphSQL, config::GraphSQLConfig};
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(version, about = "A GraphQL server for SQL databases", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "graph_sql.toml")]
    pub config: String,
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    async fn serve(config: GraphSQLConfig) -> async_graphql::Result<()> {
        let pool = config.database.create_connection().await?;

        if let Some(path) = &config.database.migration_path {
            sqlx::migrate::Migrator::new(path.clone())
                .await?
                .run(&pool)
                .await?;
        }

        let graph_sql = GraphSQL::new(config);

        let (router, listener) = graph_sql.build(&pool).await?;

        if let Err(err) = axum::serve(listener, router.into_make_service()).await {
            error!("{}", err)
        }

        Ok(())
    }

    async fn introspect(
        config: GraphSQLConfig,
        output: Option<String>,
    ) -> async_graphql::Result<()> {
        let pool = config.database.create_connection().await?;

        if let Some(path) = &config.database.migration_path {
            sqlx::migrate::Migrator::new(path.clone())
                .await?
                .run(&pool)
                .await?;
        }

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

    pub async fn start(&self) -> async_graphql::Result<()> {
        let config = load_config(&self.config)?;

        match &self.command {
            Commands::Introspect { output } => Cli::introspect(config, output.to_owned()).await,
            Commands::Serve => Cli::serve(config).await,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the GraphQL server
    Serve,
    /// Introspect the database schema and output GraphQL schema
    Introspect {
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub fn load_config(config_path: &str) -> anyhow::Result<GraphSQLConfig> {
    debug!("Loading config from: {}", config_path);

    if std::path::Path::new(config_path).exists() {
        info!("Config file found, loading from: {}", config_path);

        let config_content = std::fs::read_to_string(config_path).map_err(|e| {
            debug!("Failed to read config file: {}", e);
            e
        })?;

        let config: GraphSQLConfig = toml::from_str(&config_content).map_err(|e| {
            debug!("Failed to parse config file: {}", e);
            e
        })?;

        debug!("Config loaded successfully");
        return Ok(config);
    }

    Err(anyhow!("Unable to load config"))
}
