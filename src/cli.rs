use anyhow::anyhow;
use clap::{Parser, Subcommand};
use graph_sql::config::GraphSQLConfig;
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[command(version, about = "A GraphQL server for SQL databases", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,
    #[command(subcommand)]
    pub command: Commands,
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
