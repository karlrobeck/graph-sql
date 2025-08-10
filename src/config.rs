use std::path::PathBuf;

use async_graphql::dynamic::SchemaBuilder;
use serde::Deserialize;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing::{debug, info};

/// Load configuration from a TOML file
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

    Err(anyhow::anyhow!("Unable to load config"))
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GraphSQLConfig {
    pub server: ServerConfig,
    pub graphql: GraphQLConfig,
    pub database: DatabaseConfig,
}

impl GraphSQLConfig {
    pub fn from_path(path: &str) -> async_graphql::Result<Self> {
        Ok(load_config(path)?)
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GraphQLConfig {
    pub enable_uploading: Option<bool>,
    pub enable_playground: Option<bool>,
    pub limit_depth: Option<usize>,
    pub limit_complexity: Option<usize>,
    pub limit_recursive_depth: Option<usize>,
    pub limit_directives: Option<usize>,
    pub disable_suggestions: Option<bool>,
    pub disable_introspection: Option<bool>,
    pub introspection_only: Option<bool>,
    pub enable_federation: Option<bool>,
}

impl GraphQLConfig {
    pub fn apply(&self, mut schema: SchemaBuilder) -> SchemaBuilder {
        if self.enable_uploading.unwrap_or(true) {
            schema = schema.enable_uploading();
        }

        if let Some(complexity) = self.limit_complexity {
            schema = schema.limit_complexity(complexity);
        }

        if let Some(depth) = self.limit_depth {
            schema = schema.limit_depth(depth);
        }

        if let Some(depth) = self.limit_recursive_depth {
            schema = schema.limit_recursive_depth(depth);
        }

        if let Some(directives) = self.limit_directives {
            schema = schema.limit_directives(directives);
        }

        if self.disable_suggestions.unwrap_or(false) {
            schema = schema.disable_suggestions();
        }

        if self.disable_introspection.unwrap_or(false) {
            schema = schema.disable_introspection();
        }

        if self.introspection_only.unwrap_or(false) {
            schema = schema.introspection_only();
        }

        if self.enable_federation.unwrap_or(false) {
            schema = schema.enable_federation();
        }

        schema
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DatabaseConfig {
    pub use_env: Option<bool>,
    pub database_url: Option<String>,
    pub migration_path: Option<PathBuf>,
    pub sqlite: Option<SqliteConfig>,
}

impl DatabaseConfig {
    pub async fn create_connection(&self) -> sqlx::Result<SqlitePool> {
        if let Some(sqlite) = &self.sqlite {
            let options = sqlite.apply();

            return SqlitePool::connect_with(options).await;
        }

        if self.use_env.unwrap_or(true) {
            return SqlitePool::connect(
                &std::env::var("DATABASE_URL").unwrap_or("sqlite://:memory:".into()),
            )
            .await;
        }

        if let Some(db_url) = &self.database_url {
            return SqlitePool::connect(&db_url).await;
        }

        unimplemented!()
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqliteConfig {
    pub filename: Option<String>,
    pub foreign_keys: Option<bool>,
    pub in_memory: Option<bool>,
    pub shared_cache: Option<bool>,
    pub journal_mode: Option<SqliteJournalMode>,
    pub locking_mode: Option<SqliteLockingMode>,
    pub read_only: Option<bool>,
    pub create_if_missing: Option<bool>,
    pub statement_cache_capacity: Option<u16>,
    pub busy_timeout: Option<u16>, // in seconds
    pub synchronous: Option<SqliteSynchronousMode>,
    pub auto_vacuum: Option<SqliteVacuumMode>,
    pub page_size: Option<u32>,
    pub pragma: Option<Vec<SqlitePragma>>,
    pub immutable: Option<bool>,
    pub serialized: Option<bool>,
    pub command_buffer_size: Option<isize>,
    pub row_buffer_size: Option<isize>,
    pub vfs: Option<String>,
    pub extensions: Option<Vec<SqliteExtension>>,
    pub optimize_on_close: Option<SqliteOptimizeOnClose>,
}

impl SqliteConfig {
    pub fn apply(&self) -> SqliteConnectOptions {
        let mut options = SqliteConnectOptions::new()
            .filename(self.filename.as_deref().unwrap_or("local.db"))
            .foreign_keys(self.foreign_keys.unwrap_or(true))
            .in_memory(self.in_memory.unwrap_or(false))
            .shared_cache(self.shared_cache.unwrap_or(false))
            .read_only(self.read_only.unwrap_or(false))
            .create_if_missing(self.create_if_missing.unwrap_or(true))
            .statement_cache_capacity(self.statement_cache_capacity.unwrap_or(100) as usize)
            .busy_timeout(std::time::Duration::from_secs(
                self.busy_timeout.unwrap_or(5) as u64,
            ))
            .journal_mode(
                self.journal_mode
                    .clone()
                    .unwrap_or(SqliteJournalMode::Off)
                    .into(),
            )
            .locking_mode(
                self.locking_mode
                    .clone()
                    .unwrap_or(SqliteLockingMode::Normal)
                    .into(),
            )
            .synchronous(
                self.synchronous
                    .clone()
                    .unwrap_or(SqliteSynchronousMode::Normal)
                    .into(),
            )
            .auto_vacuum(
                self.auto_vacuum
                    .clone()
                    .unwrap_or(SqliteVacuumMode::None)
                    .into(),
            )
            .page_size(self.page_size.unwrap_or(4096))
            .immutable(self.immutable.unwrap_or(false));

        if let Some(vfs) = &self.vfs {
            if vfs != "" {
                options = options.vfs(vfs.clone())
            }
        }

        if let Some(size) = self.command_buffer_size {
            if size != -1 {
                options = options.command_buffer_size(size as usize);
            }
        }

        if let Some(size) = self.row_buffer_size {
            if size != -1 {
                options = options.row_buffer_size(size as usize);
            }
        }

        if let Some(pragmas) = &self.pragma {
            for pragma in pragmas.iter() {
                options = options.pragma(pragma.key.clone(), pragma.value.clone());
            }
        }

        if let Some(extensions) = &self.extensions {
            for extension in extensions.iter() {
                if let Some(entry_point) = &extension.entry_point {
                    let path_str = entry_point.to_string_lossy().into_owned();
                    options = options.extension_with_entrypoint(extension.name.clone(), path_str);
                } else {
                    options = options.extension(extension.name.clone());
                }
            }
        }

        if let Some(optimize_on_close) = &self.optimize_on_close {
            options = options
                .optimize_on_close(optimize_on_close.enable, optimize_on_close.analysis_limit);
        }

        options
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqlitePragma {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqliteOptimizeOnClose {
    pub enable: bool,
    pub analysis_limit: Option<u32>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqliteExtension {
    pub name: String,
    pub entry_point: Option<PathBuf>,
}

// -- enums

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteJournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    Wal,
    Off,
}

impl From<SqliteJournalMode> for sqlx::sqlite::SqliteJournalMode {
    fn from(value: SqliteJournalMode) -> Self {
        match value {
            SqliteJournalMode::Delete => sqlx::sqlite::SqliteJournalMode::Delete,
            SqliteJournalMode::Truncate => sqlx::sqlite::SqliteJournalMode::Truncate,
            SqliteJournalMode::Persist => sqlx::sqlite::SqliteJournalMode::Persist,
            SqliteJournalMode::Memory => sqlx::sqlite::SqliteJournalMode::Memory,
            SqliteJournalMode::Wal => sqlx::sqlite::SqliteJournalMode::Wal,
            SqliteJournalMode::Off => sqlx::sqlite::SqliteJournalMode::Off,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteLockingMode {
    Normal,
    Exclusive,
}

impl From<SqliteLockingMode> for sqlx::sqlite::SqliteLockingMode {
    fn from(value: SqliteLockingMode) -> Self {
        match value {
            SqliteLockingMode::Normal => sqlx::sqlite::SqliteLockingMode::Normal,
            SqliteLockingMode::Exclusive => sqlx::sqlite::SqliteLockingMode::Exclusive,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteSynchronousMode {
    Normal,
    Off,
    Full,
    Extra,
}

impl From<SqliteSynchronousMode> for sqlx::sqlite::SqliteSynchronous {
    fn from(value: SqliteSynchronousMode) -> Self {
        match value {
            SqliteSynchronousMode::Normal => sqlx::sqlite::SqliteSynchronous::Normal,
            SqliteSynchronousMode::Off => sqlx::sqlite::SqliteSynchronous::Off,
            SqliteSynchronousMode::Full => sqlx::sqlite::SqliteSynchronous::Full,
            SqliteSynchronousMode::Extra => sqlx::sqlite::SqliteSynchronous::Extra,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteVacuumMode {
    None,
    Full,
    Incremental,
}

impl From<SqliteVacuumMode> for sqlx::sqlite::SqliteAutoVacuum {
    fn from(value: SqliteVacuumMode) -> Self {
        match value {
            SqliteVacuumMode::None => sqlx::sqlite::SqliteAutoVacuum::None,
            SqliteVacuumMode::Full => sqlx::sqlite::SqliteAutoVacuum::Full,
            SqliteVacuumMode::Incremental => sqlx::sqlite::SqliteAutoVacuum::Incremental,
        }
    }
}
