use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub graphql: Option<GraphQLConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(rename = "database-url")]
    pub database_url: String,
    #[serde(rename = "use-env")]
    pub use_env: Option<bool>,
    #[serde(rename = "migrations-path")]
    pub migrations_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphQLConfig {
    #[serde(rename = "enable-playground")]
    pub enable_playground: bool,
    pub depth: u32,
    pub complexity: u32,
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            enable_playground: true,
            depth: 5,
            complexity: 5,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                database_url: "sqlite://local.db".to_string(),
                use_env: Some(true),
                migrations_path: None,
            },
            graphql: Some(GraphQLConfig {
                enable_playground: true,
                depth: 5,
                complexity: 5,
            }),
        }
    }
}

pub fn load_config(config_path: &str) -> anyhow::Result<Config> {
    debug!("Loading config from: {}", config_path);

    if std::path::Path::new(config_path).exists() {
        info!("Config file found, loading from: {}", config_path);
        let config_content = std::fs::read_to_string(config_path)?;
        let mut config: Config = toml::from_str(&config_content)?;

        // If use_env is true, try to get DATABASE_URL from environment
        if config.database.use_env.unwrap_or(false) {
            if let Ok(env_url) = std::env::var("DATABASE_URL") {
                debug!("Using DATABASE_URL from environment variable");
                config.database.database_url = env_url;
            } else {
                debug!("DATABASE_URL environment variable not found, using config value");
            }
        }

        debug!("Config loaded successfully");
        Ok(config)
    } else {
        warn!("Config file not found at {}, using defaults", config_path);
        Ok(Config::default())
    }
}
