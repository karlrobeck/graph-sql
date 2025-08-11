# Configuration Schema

This document describes the JSON schema for Graph-SQL configuration files and
how to use it for autocompletion in various editors.

## Schema File

The JSON schema is defined in [`config.schema.json`](./config.schema.json) and
provides:

- **Autocompletion** for all configuration options
- **Validation** of configuration values
- **Documentation** with descriptions and examples
- **Type checking** for all fields

## Editor Setup

### VS Code

VS Code is automatically configured via
[`.vscode/settings.json`](./.vscode/settings.json) to use the schema for:

- `config.toml`
- `*.config.toml`
- `graph-sql.toml`

No additional setup required!

### Other Editors

For editors that support JSON Schema for TOML files, reference the schema:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["config.toml", "*.config.toml"],
      "url": "./config.schema.json"
    }
  ]
}
```

## Configuration Structure

The schema defines three main sections:

### Server Configuration

```toml
[server]
host = "127.0.0.1"  # Host address to bind to
port = 3000         # Port number to listen on
```

### GraphQL Configuration

```toml
[graphql]
enable-playground = true        # Enable GraphQL Playground
enable-uploading = true         # Enable file upload support
limit-complexity = 1000         # Query complexity limit
limit-depth = 15               # Query depth limit
limit-recursive-depth = 32     # Recursive depth limit
limit-directives = 50          # Directives per field limit
disable-suggestions = false    # Disable field suggestions
disable-introspection = false  # Disable introspection
introspection-only = false     # Only allow introspection
enable-federation = false      # Enable Apollo Federation
```

### Database Configuration

```toml
[database]
use-env = true                 # Use DATABASE_URL environment variable
database-url = "sqlite://..."  # Direct database URL
migration-path = "./migrations" # Path to migration files

[database.sqlite]
filename = "app.db"            # Database file path
foreign-keys = true            # Enable foreign key enforcement
journal-mode = "wal"           # Journal mode (delete, truncate, persist, memory, wal, off)
synchronous = "normal"         # Sync mode (off, normal, full, extra)
# ... many more SQLite options available
```

## Examples

The schema includes comprehensive examples:

### Basic Configuration

```toml
[server]
host = "127.0.0.1"
port = 3000

[graphql]
enable-playground = true
limit-complexity = 1000

[database]
use-env = false

[database.sqlite]
filename = "app.db"
foreign-keys = true
journal-mode = "wal"
synchronous = "normal"
```

### Production Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080

[graphql]
enable-playground = false
limit-complexity = 5000
limit-depth = 15
disable-introspection = true
disable-suggestions = true

[database]
use-env = false

[database.sqlite]
filename = "production.db"
foreign-keys = true
journal-mode = "wal"
synchronous = "normal"
busy-timeout = 30
statement-cache-capacity = 200

[[database.sqlite.pragma]]
key = "cache_size"
value = "10000"

[[database.sqlite.pragma]]
key = "temp_store"
value = "memory"

[database.sqlite.optimize-on-close]
enable = true
analysis-limit = 1000
```

## Schema Features

### Type Validation

- **Integers**: Validated ranges (e.g., port 1-65535)
- **Enums**: Predefined values with descriptions
- **Booleans**: True/false validation
- **Arrays**: Structured object arrays

### Documentation

- **Descriptions**: Detailed explanations for each field
- **Examples**: Real-world usage examples
- **Defaults**: Default values clearly marked
- **Links**: References to SQLite documentation

### Autocompletion

The schema provides intelligent autocompletion for:

- Configuration keys
- Enum values (e.g., journal modes)
- Example values
- Default suggestions

## Validation

The schema validates:

- Required fields are present
- Value types match expectations
- Enum values are valid
- Numeric ranges are respected
- Object structures are correct

## Contributing

When adding new configuration options:

1. Update the Rust structs in [`src/config.rs`](./src/config.rs)
2. Update the JSON schema in [`config.schema.json`](./config.schema.json)
3. Add examples and documentation
4. Test autocompletion works correctly

## References

- [JSON Schema Specification](https://json-schema.org/)
- [SQLite PRAGMA Documentation](https://www.sqlite.org/pragma.html)
- [SQLx SQLite Options](https://docs.rs/sqlx/latest/sqlx/sqlite/struct.SqliteConnectOptions.html)
- [async-graphql SchemaBuilder](https://docs.rs/async-graphql/latest/async_graphql/dynamic/struct.SchemaBuilder.html)
