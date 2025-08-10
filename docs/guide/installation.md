# Installation

graph-sql can be used both as a CLI application and as a Rust library. Choose
the installation method that best fits your needs.

## CLI Application (Recommended)

Install graph-sql globally using cargo:

```bash
cargo install graph-sql --git https://github.com/karlrobeck/graph-sql.git
```

### Quick Start

Once installed, you can immediately start serving any SQLite database:

```bash
# Serve any SQLite database as GraphQL API (uses default config.toml)
graph-sql serve

# Use custom config file
graph-sql serve -c my-config.toml

# Introspect and view schema
graph-sql introspect

# Save schema to file
graph-sql introspect -o schema.graphql
```

## As a Library

Add graph-sql to your `Cargo.toml`:

```toml
[dependencies]
graph-sql = { git = "https://github.com/karlrobeck/graph-sql.git" }
async-graphql = "7.0.17"
async-graphql-axum = "7.0.17"
axum = "0.8.4"
sqlx = { version = "0.8.6", features = ["runtime-tokio-native-tls", "sqlite", "migrate"] }
tokio = { version = "1.47.0", features = ["full"] }
```

## Prerequisites

- **Rust 1.86.0+** (2024 edition)
- **SQLite database** (or let graph-sql create one for you)

## Verification

After installation, verify everything is working:

```bash
# Check version
graph-sql --version

# Test with a simple database
echo '[server]
host = "0.0.0.0"
port = 8000

[database]
database-url = "sqlite://test.db"

[graphql]
enable-playground = true' > config.toml

graph-sql serve
```

Open your browser to `http://localhost:8000` to see the GraphiQL interface.
