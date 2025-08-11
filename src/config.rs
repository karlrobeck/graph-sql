use std::path::PathBuf;

use async_graphql::dynamic::SchemaBuilder;
use serde::Deserialize;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing::{debug, info};

/// Load configuration from a TOML file.
///
/// This function reads a TOML configuration file from the specified path and
/// parses it into a [`GraphSQLConfig`] structure.
///
/// # Arguments
///
/// * `config_path` - A string slice that holds the path to the configuration file
///
/// # Returns
///
/// Returns `Ok(GraphSQLConfig)` if the file exists and can be parsed successfully,
/// otherwise returns an error.
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file does not exist
/// - The file cannot be read due to permission issues
/// - The TOML content is malformed or invalid
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

/// Main configuration structure for Graph-SQL.
///
/// This structure contains all the necessary configuration sections
/// for running a Graph-SQL server, including server settings, GraphQL
/// schema configuration, and database connection parameters.
///
/// # Example
///
/// ```toml
/// [server]
/// host = "127.0.0.1"
/// port = 3000
///
/// [graphql]
/// enable-playground = true
/// limit-complexity = 1000
///
/// [database]
/// use-env = false
///
/// [database.sqlite]
/// filename = "data.db"
/// foreign-keys = true
/// ```
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GraphSQLConfig {
    /// Server configuration settings
    pub server: ServerConfig,
    /// GraphQL schema and behavior configuration
    pub graphql: GraphQLConfig,
    /// Database connection and SQLite-specific settings
    pub database: DatabaseConfig,
}

impl GraphSQLConfig {
    /// Creates a new `GraphSQLConfig` from a configuration file path.
    ///
    /// This is a convenience method that wraps [`load_config`] and converts
    /// any errors to async-graphql compatible errors.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    ///
    /// Returns the parsed configuration or an async-graphql error.
    pub fn from_path(path: &str) -> async_graphql::Result<Self> {
        Ok(load_config(path)?)
    }
}

/// HTTP server configuration.
///
/// Contains settings for the HTTP server that will host the GraphQL endpoint.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    /// The host address to bind the server to (e.g., "127.0.0.1", "0.0.0.0")
    pub host: String,
    /// The port number to listen on (e.g., 3000, 8080)
    pub port: u16,
}

/// GraphQL schema configuration and security settings.
///
/// This structure configures various aspects of the GraphQL schema behavior,
/// including security limits, feature toggles, and development tools.
/// All fields are optional and will use sensible defaults if not specified.
///
/// See [`async_graphql::dynamic::SchemaBuilder`] for more details on these options.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GraphQLConfig {
    /// Enable file uploading capabilities (default: true)
    /// Registers the Upload scalar type for handling multipart file uploads
    pub enable_uploading: Option<bool>,

    /// Enable GraphQL Playground in development (default: false in production)
    /// Provides an interactive GraphQL IDE for testing queries
    pub enable_playground: Option<bool>,

    /// Maximum query depth allowed (default: unlimited)
    /// Prevents deeply nested queries that could cause performance issues
    pub limit_depth: Option<usize>,

    /// Maximum query complexity score allowed (default: unlimited)
    /// Uses a scoring system to prevent overly complex queries
    pub limit_complexity: Option<usize>,

    /// Maximum recursive depth in selections (default: 32)
    /// Prevents stack overflow from excessive recursion
    pub limit_recursive_depth: Option<usize>,

    /// Maximum number of directives per field (default: unlimited)
    /// Limits directive usage to prevent abuse
    pub limit_directives: Option<usize>,

    /// Disable GraphQL field name suggestions in error messages (default: false)
    /// When enabled, improves security by not revealing schema structure
    pub disable_suggestions: Option<bool>,

    /// Disable introspection queries completely (default: false)
    /// Prevents schema discovery in production environments
    pub disable_introspection: Option<bool>,

    /// Only allow introspection queries, block all others (default: false)
    /// Useful for schema analysis tools without exposing data
    pub introspection_only: Option<bool>,

    /// Enable Apollo Federation support (default: false)
    /// Allows this service to participate in a federated GraphQL architecture
    pub enable_federation: Option<bool>,
}

impl GraphQLConfig {
    /// Apply this configuration to a GraphQL schema builder.
    ///
    /// This method configures an [`async_graphql::dynamic::SchemaBuilder`] with all
    /// the settings specified in this configuration. Each setting is applied
    /// only if it has been explicitly configured (is `Some`).
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema builder to configure
    ///
    /// # Returns
    ///
    /// Returns the configured schema builder ready for finalization.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_graphql::dynamic::SchemaBuilder;
    /// # use graph_sql::config::GraphQLConfig;
    ///
    /// let config = GraphQLConfig {
    ///     limit_complexity: Some(1000),
    ///     disable_introspection: Some(true),
    ///     ..Default::default()
    /// };
    ///
    /// let schema_builder = SchemaBuilder::new();
    /// let configured_builder = config.apply(schema_builder);
    /// ```
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

/// Database connection configuration.
///
/// This structure manages database connection settings and supports multiple
/// connection methods including environment variables, direct URLs, and
/// detailed SQLite-specific configuration.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DatabaseConfig {
    /// Use DATABASE_URL environment variable (default: true)
    /// When true, attempts to connect using the DATABASE_URL environment variable
    pub use_env: Option<bool>,

    /// Direct database connection URL
    /// Used when use_env is false or DATABASE_URL is not set
    pub database_url: Option<String>,

    /// Path to database migration files
    /// Used for running database schema migrations
    pub migration_path: Option<PathBuf>,

    /// SQLite-specific connection configuration
    /// Provides fine-grained control over SQLite connection parameters
    pub sqlite: Option<SqliteConfig>,
}

impl DatabaseConfig {
    /// Create a SQLite connection pool using the configured settings.
    ///
    /// This method attempts to create a connection pool in the following order:
    /// 1. Use SQLite-specific configuration if provided
    /// 2. Use DATABASE_URL environment variable if `use_env` is true (default)
    /// 3. Use the `database_url` field if specified
    /// 4. Fall back to in-memory database as last resort
    ///
    /// # Returns
    ///
    /// Returns a `SqlitePool` on success or a SQLx error on failure.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The database file cannot be accessed or created
    /// - The connection parameters are invalid
    /// - The SQLite driver encounters an initialization error
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
            return SqlitePool::connect(db_url).await;
        }

        unimplemented!()
    }
}

/// Comprehensive SQLite connection configuration.
///
/// This structure provides fine-grained control over SQLite connection parameters,
/// corresponding to the options available in [`sqlx::sqlite::SqliteConnectOptions`].
/// All fields are optional and will use SQLx defaults if not specified.
///
/// # Examples
///
/// Basic configuration:
/// ```toml
/// [database.sqlite]
/// filename = "app.db"
/// foreign-keys = true
/// ```
///
/// Advanced configuration:
/// ```toml
/// [database.sqlite]
/// filename = "app.db"
/// foreign-keys = true
/// journal-mode = "wal"
/// synchronous = "normal"
/// busy-timeout = 30
/// statement-cache-capacity = 200
/// ```
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqliteConfig {
    /// Database file path (default: "local.db")
    /// Set to ":memory:" for in-memory database
    pub filename: Option<String>,

    /// Enable foreign key constraint enforcement (default: true)
    /// SQLx enables this by default unlike SQLite's default behavior
    pub foreign_keys: Option<bool>,

    /// Create database in memory instead of on disk (default: false)
    /// Sets the SQLITE_OPEN_MEMORY flag
    pub in_memory: Option<bool>,

    /// Enable shared cache mode (default: false)
    /// Sets the SQLITE_OPEN_SHAREDCACHE flag for sharing data between connections
    pub shared_cache: Option<bool>,

    /// Journal mode for crash recovery and concurrency
    /// Options: delete, truncate, persist, memory, wal, off (default: off)
    pub journal_mode: Option<SqliteJournalMode>,

    /// Locking mode for database access
    /// Options: normal, exclusive (default: normal)
    pub locking_mode: Option<SqliteLockingMode>,

    /// Open database in read-only mode (default: false)
    pub read_only: Option<bool>,

    /// Create database file if it doesn't exist (default: true)
    pub create_if_missing: Option<bool>,

    /// Maximum number of prepared statements to cache (default: 100)
    /// Uses LRU eviction when capacity is reached
    pub statement_cache_capacity: Option<u16>,

    /// Timeout in seconds when database is locked (default: 5)
    /// How long to wait before returning SQLITE_BUSY error
    pub busy_timeout: Option<u16>,

    /// Synchronization mode for durability vs performance
    /// Options: off, normal, full, extra (default: normal)
    pub synchronous: Option<SqliteSynchronousMode>,

    /// Automatic vacuum mode for database maintenance
    /// Options: none, full, incremental (default: none)
    pub auto_vacuum: Option<SqliteVacuumMode>,

    /// Page size in bytes (default: 4096)
    /// Must be a power of 2 between 512 and 65536
    pub page_size: Option<u32>,

    /// Custom PRAGMA statements to execute on connection
    /// Allows setting additional SQLite configuration options
    pub pragma: Option<Vec<SqlitePragma>>,

    /// Mark database as immutable/read-only media (default: false)
    /// Optimization for read-only databases on read-only storage
    pub immutable: Option<bool>,

    /// Use serialized threading mode (default: false)
    /// Enable only if experiencing concurrency issues, has performance cost
    pub serialized: Option<bool>,

    /// Command buffer size for worker thread backpressure
    /// Set to -1 to use SQLx default
    pub command_buffer_size: Option<isize>,

    /// Row buffer size for result streaming backpressure
    /// Set to -1 to use SQLx default  
    pub row_buffer_size: Option<isize>,

    /// Virtual File System name (default: empty, uses OS default)
    /// Allows using custom VFS implementations
    pub vfs: Option<String>,

    /// SQLite extensions to load at connection time
    /// Enables additional functionality like spatial operations
    pub extensions: Option<Vec<SqliteExtension>>,

    /// Execute PRAGMA optimize on connection close
    /// Recommended for long-lived databases to maintain query performance
    pub optimize_on_close: Option<SqliteOptimizeOnClose>,
}

impl SqliteConfig {
    /// Convert this configuration into SQLx `SqliteConnectOptions`.
    ///
    /// This method creates a [`sqlx::sqlite::SqliteConnectOptions`] instance
    /// configured with all the settings specified in this configuration.
    /// Any unspecified options will use sensible defaults.
    ///
    /// # Returns
    ///
    /// Returns a fully configured `SqliteConnectOptions` ready for creating connections.
    ///
    /// # Default Values
    ///
    /// - `filename`: "local.db"
    /// - `foreign_keys`: true
    /// - `statement_cache_capacity`: 100
    /// - `busy_timeout`: 5 seconds
    /// - `page_size`: 4096 bytes
    /// - All other options use SQLx/SQLite defaults
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
            if !vfs.is_empty() {
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

/// Custom PRAGMA statement configuration.
///
/// Represents a key-value pair for setting SQLite PRAGMA options
/// that are not directly supported by the configuration structure.
///
/// # Example
///
/// ```toml
/// [[database.sqlite.pragma]]
/// key = "cache_size"
/// value = "10000"
///
/// [[database.sqlite.pragma]]
/// key = "temp_store"
/// value = "memory"
/// ```
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqlitePragma {
    /// PRAGMA option name
    pub key: String,
    /// PRAGMA option value
    pub value: String,
}

/// PRAGMA optimize configuration for connection close.
///
/// Controls whether SQLite should run optimization analysis when
/// the connection is closed, which helps maintain query performance
/// for long-lived databases.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqliteOptimizeOnClose {
    /// Enable optimization on close
    pub enable: bool,
    /// Maximum number of rows to analyze per index (optional)
    /// Limits the scope of optimization analysis
    pub analysis_limit: Option<u32>,
}

/// SQLite extension configuration.
///
/// Defines a SQLite extension to be loaded when the connection is established.
/// Extensions can provide additional functionality like spatial operations,
/// full-text search, or custom functions.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SqliteExtension {
    /// Extension name or file path
    pub name: String,
    /// Custom entry point function name (optional)
    /// Used when the extension doesn't follow standard naming conventions
    pub entry_point: Option<PathBuf>,
}

// -- enums

/// SQLite journal mode configuration.
///
/// The journal mode controls how SQLite handles crash recovery and
/// concurrent access. Each mode has different trade-offs between
/// performance, durability, and concurrency.
///
/// See [SQLite documentation](https://www.sqlite.org/pragma.html#pragma_journal_mode) for details.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteJournalMode {
    /// DELETE mode (default for file databases)
    /// Journal file is deleted after each transaction
    Delete,
    /// TRUNCATE mode
    /// Journal file is truncated to zero length instead of deleted
    Truncate,
    /// PERSIST mode  
    /// Journal file is not deleted, header is overwritten with zeros
    Persist,
    /// MEMORY mode (default for in-memory databases)
    /// Journal is stored in memory instead of on disk
    Memory,
    /// WAL mode (Write-Ahead Logging)
    /// Best for concurrent read/write access, persists across connections
    Wal,
    /// OFF mode
    /// No journal, fastest but no crash recovery
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

/// SQLite database locking mode.
///
/// Controls how SQLite manages database file locking for concurrent access.
///
/// See [SQLite documentation](https://www.sqlite.org/pragma.html#pragma_locking_mode) for details.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteLockingMode {
    /// NORMAL mode (default)
    /// Database file is unlocked after each read or write transaction
    Normal,
    /// EXCLUSIVE mode
    /// Database file remains locked, preventing other processes from accessing it
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

/// SQLite synchronization mode.
///
/// Controls how much synchronization SQLite does with the file system
/// to ensure durability. Higher levels provide better crash protection
/// but with performance costs.
///
/// See [SQLite documentation](https://www.sqlite.org/pragma.html#pragma_synchronous) for details.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteSynchronousMode {
    /// NORMAL mode (recommended for WAL mode)
    /// Syncs at critical moments, good balance of safety and performance
    Normal,
    /// OFF mode
    /// No syncing, fastest but least safe
    Off,
    /// FULL mode (default)
    /// Syncs after every write, safest but slowest
    Full,
    /// EXTRA mode
    /// Even more syncing than FULL, maximum safety
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

/// SQLite automatic vacuum mode.
///
/// Controls how SQLite handles database file size management when
/// data is deleted. Vacuum operations reclaim space from deleted records.
///
/// See [SQLite documentation](https://www.sqlite.org/pragma.html#pragma_auto_vacuum) for details.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SqliteVacuumMode {
    /// No automatic vacuuming (default)
    /// Deleted space is not automatically reclaimed
    None,
    /// Full automatic vacuuming
    /// Database file shrinks automatically when data is deleted
    Full,
    /// Incremental vacuuming
    /// Space is marked for reclaim but manual PRAGMA incremental_vacuum is needed
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
