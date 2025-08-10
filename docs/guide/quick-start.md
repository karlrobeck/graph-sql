# Quick Start

Get up and running with graph-sql in minutes. This guide will show you how to
create your first GraphQL API from a SQLite database.

## CLI Tool Setup

### 1. Install graph-sql

```bash
cargo install graph-sql --git https://github.com/karlrobeck/graph-sql.git
```

### 2. Create a Configuration File

Create a `graph_sql.toml` file in your project directory:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
database-url = "sqlite://my_app.db"
use-env = false

[graphql]
enable-playground = true
```

### 3. Start the Server

```bash
# Serve with default config (graph_sql.toml)
graph-sql serve

# Or specify a custom config file
graph-sql serve -c my-config.toml

# View available commands and options
graph-sql --help
```

Open `http://localhost:8080` for the GraphiQL interface!

### 4. Introspect Your Database Schema

You can also introspect your database to see the generated GraphQL schema:

```bash
# Print schema to stdout
graph-sql introspect

# Save schema to a file
graph-sql introspect -o schema.graphql

# Use custom config for introspection
graph-sql introspect -c my-config.toml -o my-schema.graphql
```

## Library Integration

For Rust projects, you can integrate graph-sql as a library. Here's how to use
the same approach as the CLI:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Load configuration (same format as CLI)
    let config = GraphSQLConfig::from_path("graph_sql.toml")?;
    
    // Create database connection
    let pool = config.database.create_connection().await?;
    
    // Create GraphSQL instance
    let graph_sql = GraphSQL::new(config);
    
    // Build the router and listener
    let (router, listener) = graph_sql.build(&pool).await?;
    
    // Start the server
    axum::serve(listener, router.into_make_service()).await?;
    
    Ok(())
}
```

### Alternative: Manual Setup

If you prefer more control over the setup:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Create config programmatically
    let config = GraphSQLConfig {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
        },
        database: DatabaseConfig {
            database_url: "sqlite://app.db".to_string(),
            use_env: false,
        },
        graphql: GraphQLConfig {
            enable_playground: true,
        },
    };
    
    // Rest of setup same as above
    let pool = config.database.create_connection().await?;
    let graph_sql = GraphSQL::new(config);
    let (router, listener) = graph_sql.build(&pool).await?;
    
    axum::serve(listener, router.into_make_service()).await?;
    
    Ok(())
}
```

## CLI Commands Reference

### Available Commands

```bash
# Show help and available commands
graph-sql --help

# Start the GraphQL server
graph-sql serve

# Start with custom config
graph-sql serve -c path/to/config.toml

# Introspect database schema
graph-sql introspect

# Save schema to file
graph-sql introspect -o schema.graphql

# Introspect with custom config
graph-sql introspect -c my-config.toml -o my-schema.graphql
```

### Configuration File

The CLI uses `graph_sql.toml` by default, but you can specify any file with
`-c`:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
database-url = "sqlite://app.db"
use-env = false

[graphql]
enable-playground = true
```

## What Happens Automatically

graph-sql automatically:

- 🔍 **Introspects your database** schema
- 🏗️ **Generates GraphQL types** for all tables
- 🔗 **Maps foreign keys** to GraphQL relationships
- ⚡ **Creates CRUD resolvers** for all operations
- 🎮 **Provides GraphiQL** interface for testing

## Quick Example with Blog System

Let's run the blog example to see graph-sql in action:

```bash
# Clone the repository
git clone https://github.com/karlrobeck/graph-sql.git
cd graph-sql

# Run the blog example
cd examples/blog
cargo run

# The server will start on http://localhost:8080
# Open the GraphiQL interface to explore the schema
```

The blog example includes:

- Users table with posts relationship
- Posts table with comments relationship
- Comments table with nested comment support
- Automatic GraphQL schema generation from SQLite schema

## Basic Usage Examples

Once your server is running, you can execute GraphQL queries:

### Query Data

```graphql
# Query all posts with author information
query {
  posts {
    id
    title
    content
    author {
      name
      email
    }
  }
}
```

### Create Data

```graphql
# Create a new post
mutation {
  createPost(
    title: "My New Post"
    content: "This is the content"
    authorId: 1
  ) {
    id
    title
  }
}
```

### Update Data

```graphql
# Update an existing post
mutation {
  updatePost(
    id: 1
    title: "Updated Title"
    content: "Updated content"
  ) {
    id
    title
    content
  }
}
```

## Next Steps

- Explore the [Configuration](/guide/configuration) options
- Check out the [Examples](/guide/examples) for real-world usage
- Learn about the [Library API](/guide/library-api) for custom integrations
