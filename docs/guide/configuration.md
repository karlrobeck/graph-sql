# Configuration

graph-sql uses TOML configuration files for all settings. The CLI looks for
`graph_sql.toml` in the current directory by default, or you can specify a
custom path with `-c`.

## Configuration Structure

```toml
[server]
host = "0.0.0.0"        # Server bind address
port = 8080             # Server port

[database]
database-url = "sqlite://local.db"  # Database connection string (optional if using sqlite section)
use-env = true                      # Use DATABASE_URL env var if available
migration-path = "./migrations"     # Optional path to migration files (future implementation)

[graphql]
enable-uploading = true             # Enable file uploads
enable-playground = true            # Enable GraphiQL interface
limit-depth = 5                     # Query depth limit
limit-complexity = 100              # Query complexity limit
limit-recursive-depth = 10          # Recursive depth limit
limit-directives = 50               # Directive limit
disable-suggestions = false         # Disable query suggestions
disable-introspection = false       # Disable schema introspection
introspection-only = false          # Allow only introspection queries
enable-federation = false           # Enable Apollo Federation

# Advanced SQLite configuration (optional)
[database.sqlite]
filename = "app.db"                 # Database filename
foreign-keys = true                 # Enable foreign key constraints
create-if-missing = true            # Create database if it doesn't exist
journal-mode = "wal"                # Journal mode (delete, truncate, persist, memory, wal, off)
synchronous = "normal"              # Synchronous mode (normal, off, full, extra)
# ... many more SQLite options available
```

## Configuration Options

### Server Section

- **`host`** - Server bind address (default: "0.0.0.0")
- **`port`** - Server port number (default: 8080)

```toml
[server]
host = "127.0.0.1"  # Only accept local connections
port = 3000         # Custom port
```

### Database Section

- **`database-url`** - SQLite database connection string (optional if using
  sqlite section)
- **`use-env`** - If true, uses `DATABASE_URL` environment variable when
  available (default: true)
- **`migration-path`** - Optional directory containing SQL migration files
  (future implementation)
- **`sqlite`** - Advanced SQLite configuration options (see below)

```toml
[database]
database-url = "sqlite://production.db"  # Simple connection string
use-env = true
migration-path = "./migrations"          # Future implementation

# OR use advanced SQLite configuration
[database.sqlite]
filename = "production.db"
foreign-keys = true
journal-mode = "wal"
```

### GraphQL Section (Optional)

All GraphQL options are optional with sensible defaults:

- **`enable-uploading`** - Enable file uploads (default: true)
- **`enable-playground`** - Enable GraphiQL interactive interface (default:
  true)
- **`limit-depth`** - Maximum query depth allowed
- **`limit-complexity`** - Maximum query complexity allowed
- **`limit-recursive-depth`** - Maximum recursive depth allowed
- **`limit-directives`** - Maximum number of directives allowed
- **`disable-suggestions`** - Disable query suggestions (default: false)
- **`disable-introspection`** - Disable schema introspection (default: false)
- **`introspection-only`** - Allow only introspection queries (default: false)
- **`enable-federation`** - Enable Apollo Federation support (default: false)

```toml
[graphql]
enable-playground = false  # Disable in production
limit-depth = 10          # Allow deeper queries
limit-complexity = 200    # Allow more complex queries
disable-introspection = true  # Disable introspection in production
```

### Advanced SQLite Configuration

The `[database.sqlite]` section provides fine-grained control over SQLite
behavior:

#### Basic SQLite Options

```toml
[database.sqlite]
filename = "app.db"                 # Database file path
foreign-keys = true                 # Enable foreign key constraints (default: true)
in-memory = false                   # Use in-memory database (default: false)
shared-cache = false                # Enable shared cache mode (default: false)
read-only = false                   # Open database in read-only mode (default: false)
create-if-missing = true            # Create database if it doesn't exist (default: true)
```

#### Performance Options

```toml
[database.sqlite]
statement-cache-capacity = 100      # Number of prepared statements to cache (default: 100)
busy-timeout = 5                    # Busy timeout in seconds (default: 5)
page-size = 4096                    # Database page size in bytes (default: 4096)
command-buffer-size = 8192          # Command buffer size (-1 for default)
row-buffer-size = 8192              # Row buffer size (-1 for default)
```

#### Journal and Synchronization

```toml
[database.sqlite]
journal-mode = "wal"                # Journal mode: delete, truncate, persist, memory, wal, off
locking-mode = "normal"             # Locking mode: normal, exclusive
synchronous = "normal"              # Synchronous mode: normal, off, full, extra
auto-vacuum = "none"                # Auto vacuum: none, full, incremental
```

#### Advanced Options

```toml
[database.sqlite]
immutable = false                   # Database is immutable (default: false)
serialized = false                  # Serialized threading mode (default: false)
vfs = ""                           # Virtual file system to use (empty for default)

# Custom pragma settings
[[database.sqlite.pragma]]
key = "cache_size"
value = "10000"

[[database.sqlite.pragma]]
key = "temp_store"
value = "memory"

# SQLite extensions
[[database.sqlite.extensions]]
name = "sqlean"
entry-point = "/path/to/sqlean.so"  # Optional entry point

# Optimization settings
[database.sqlite.optimize-on-close]
enable = true
analysis-limit = 1000               # Optional analysis limit
```

## Environment Variables

The database configuration supports environment variables:

```bash
export DATABASE_URL="sqlite://production.db"
```

With this configuration:

```toml
[database]
use-env = true  # Will use DATABASE_URL if available
# database-url is optional when use-env = true
```

If `use-env = true` (which is the default), the system will:

1. First check for `DATABASE_URL` environment variable
2. Fall back to `database-url` in config if env var not found
3. Fall back to `sqlite://:memory:` if neither is specified

## Configuration Examples

### Simple Development Configuration

```toml
[server]
host = "localhost"
port = 8080

[database]
database-url = "sqlite://dev.db"
use-env = false

[graphql]
enable-playground = true
```

### Advanced Development Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
use-env = false
# migration-path = "./migrations"  # Future implementation (commented out)

[database.sqlite]
filename = "dev.db"
foreign-keys = true
journal-mode = "wal"
synchronous = "normal"
create-if-missing = true

[graphql]
enable-playground = true
limit-depth = 20
limit-complexity = 1000
```

### Production Configuration

```toml
[server]
host = "0.0.0.0"
port = 80

[database]
use-env = true  # Use DATABASE_URL environment variable

[database.sqlite]
foreign-keys = true
journal-mode = "wal"
synchronous = "normal"
busy-timeout = 30

[graphql]
enable-playground = false
enable-uploading = false
limit-depth = 5
limit-complexity = 100
disable-introspection = true
```

### Docker Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
use-env = true

[database.sqlite]
filename = "/data/app.db"  # Use mounted volume
journal-mode = "wal"
busy-timeout = 30

[graphql]
enable-playground = true
limit-depth = 10
```

### High-Performance Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080

[database.sqlite]
filename = "app.db"
journal-mode = "wal"
synchronous = "normal"
page-size = 8192
statement-cache-capacity = 200
busy-timeout = 60

# Performance pragmas
[[database.sqlite.pragma]]
key = "cache_size"
value = "20000"

[[database.sqlite.pragma]]
key = "temp_store"
value = "memory"

[[database.sqlite.pragma]]
key = "mmap_size"
value = "268435456"  # 256MB

[graphql]
enable-playground = false
limit-depth = 8
limit-complexity = 500
limit-recursive-depth = 15
```

## Using Custom Configuration Files

You can specify a custom configuration file path:

```bash
# Use a specific config file
graph-sql serve -c /path/to/my-config.toml

# Different configs for different environments
graph-sql serve -c configs/development.toml
graph-sql serve -c configs/production.toml
graph-sql serve -c configs/testing.toml
```

## Configuration File Loading

graph-sql looks for configuration files in this order:

1. File specified with `-c` flag
2. `graph_sql.toml` in current directory
3. If no config file is found, an error is returned

## Configuration Validation

graph-sql validates your configuration on startup and provides helpful error
messages:

```bash
# Example error output
Error: Invalid configuration
  -> server.port must be between 1 and 65535
  -> database section is required
  -> graphql.limit-depth must be a positive number
```

## Migration Support (Future Implementation)

> **⚠️ Coming Soon**: Migration support is planned for a future release and is
> not yet implemented.

The `migration-path` configuration option is available in the configuration
schema but the migration functionality is not yet active:

```toml
[database]
migration-path = "./migrations"  # Reserved for future migration support
```

**Planned Features**:

- Automatic database migration execution on startup
- Sequential migration file processing
- Migration rollback capabilities
- Migration status tracking

**Planned Migration File Structure**:

```
migrations/
├── 001_initial.sql
├── 002_add_users.sql
└── 003_add_posts.sql
```

**Current Workaround**: For now, you can manually run your SQL schema files
against your SQLite database:

```bash
# Apply schema manually
sqlite3 app.db < schema.sql

# Or using the sqlite3 command line
sqlite3 app.db
.read schema.sql
.exit
```

## SQLite Journal Modes

Understanding journal modes for optimal performance:

- **`delete`** - Default mode, deletes journal file after each transaction
- **`truncate`** - Truncates journal file instead of deleting (faster on some
  systems)
- **`persist`** - Keeps journal file between transactions
- **`memory`** - Stores journal in memory (faster but less safe)
- **`wal`** - Write-Ahead Logging (recommended for most use cases)
- **`off`** - No journal (fastest but unsafe)

## Synchronous Modes

Control durability vs performance trade-offs:

- **`off`** - No synchronization (fastest, least safe)
- **`normal`** - Sync at critical moments (good balance)
- **`full`** - Sync after every write (safest, slowest)
- **`extra`** - Extra durability checks (very safe, very slow)
