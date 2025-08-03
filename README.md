# graph-sql

A high-performance Rust CLI tool and library that automatically introspects SQLite databases and generates complete GraphQL APIs with zero configuration. Built as a memory-safe alternative to traditional GraphQL servers, graph-sql acts as a lightweight gateway that pushes business logic to the database layer for maximum performance and simplicity.

🚀 **NEW**: graph-sql is now available as a standalone CLI application! Install it globally and instantly serve any SQLite database as a GraphQL API.

🔒 **Memory Safety**: Leverages Rust's zero-cost abstractions and memory safety guarantees to eliminate entire classes of bugs common in traditional API servers.

🏗️ **Database-First Architecture**: Acts as a stateless gateway/middleman, letting SQLite handle business logic, authorization, and data processing for optimal performance.

For detailed documentation and more queries, see the [examples directory](./examples/).

> **⚠️ Development Status**  
> **This project is in active development.** Breaking changes may occur without notice as we rapidly iterate and improve the library. While the core functionality is stable, the API may evolve significantly. For production use, please pin to a specific commit and thoroughly test any updates.

## 📋 Table of Contents

- [Installation](#-installation)
  - [CLI Application (Recommended)](#cli-application-recommended)
  - [CLI Usage](#cli-usage)
  - [As a Library](#as-a-library)
- [Library API](#-library-api)
  - [Core Functions](#core-functions)
  - [Integration Patterns](#integration-patterns)
  - [Framework Integration](#framework-integration)
  - [Use Cases](#use-cases)
- [How It Works](#-how-it-works)
- [Features](#-features)
- [Prerequisites](#-prerequisites)
- [Quick Start](#-quick-start)
  - [CLI Tool](#cli-tool)
  - [Library Integration](#library-integration)
- [Examples](#-examples)
  - [Running Examples](#-running-examples)
  - [Example Overview](#-example-overview)
  - [Quick Example - Blog System](#-quick-example---blog-system)
- [GraphQL Schema Structure](#️-graphql-schema-structure)
- [Database Schema Mapping](#️-database-schema-mapping)
- [Example Usage](#-example-usage)
- [Configuration](#️-configuration)
- [Architecture](#️-architecture)
- [Development](#-development)
- [Current Limitations](#-current-limitations)
- [Roadmap](#️-roadmap)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)
- [Support](#-support)

## 📦 Installation

### **CLI Application (Recommended)**

Install graph-sql globally using cargo:

```bash
cargo install --git https://github.com/karlrobeck/graph-sql.git
```

**Quick Start**:
```bash
# Serve any SQLite database as GraphQL API
graph-sql serve -d "sqlite://my_database.db" -p 8000

# Introspect and view schema
graph-sql introspect -d "sqlite://my_database.db"

# With migrations
graph-sql serve -d "sqlite://my_database.db" -m "./migrations"
```

### **CLI Usage**

```bash
# Start GraphQL server (creates local.db if not exists)
graph-sql serve

# Custom database and port
graph-sql serve -d "sqlite://data/app.db" -p 3000 --host 127.0.0.1

# Run with migrations
graph-sql serve -d "sqlite://app.db" -m "./migrations"

# Introspect schema to stdout
graph-sql introspect -d "sqlite://app.db"

# Save schema to file
graph-sql introspect -d "sqlite://app.db" -o schema.graphql

# Use environment variable
export DATABASE_URL="sqlite://production.db"
graph-sql serve  # Uses DATABASE_URL
```

**CLI Features**:
- 🗄️ **Automatic database creation** - Creates SQLite files if they don't exist
- 🔧 **Migration support** - Optional migration directory with `-m` flag
- 🌍 **Environment variables** - Uses `DATABASE_URL` environment variable
- 📄 **Schema export** - Export GraphQL schema to files with `introspect` command
- ⚙️ **Flexible configuration** - Customize host, port, and database URL

### **As a Library**

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

## 🔧 Library API

When using graph-sql as a library, it provides a simple, elegant API for integrating GraphQL into your Rust applications:

### **Core Functions**

```rust
// Main introspection function
pub async fn introspect(pool: &SqlitePool) -> Result<SchemaBuilder, Error>

// Schema builder for customization
impl SchemaBuilder {
    pub fn finish(self) -> Result<Schema<Query, Mutation, EmptySubscription>, Error>
    // Additional customization methods available
}
```

### **Integration Patterns**

**🔥 Minimal Setup** (3 lines):
```rust
let db = SqlitePool::connect("sqlite://app.db").await?;
let schema = graph_sql::introspect(&db).await?.finish()?;
let app = Router::new().route("/graphql", post_service(GraphQL::new(schema)));
```

**🛠️ With Custom Configuration**:
```rust
let schema = graph_sql::introspect(&db)
    .await?
    // Add custom resolvers, middleware, etc.
    .finish()?;
```

**🔄 With Hot Reloading** (Development):
```rust
// Reintrospect when schema changes
let schema = graph_sql::introspect(&db).await?.finish()?;
```

### **Framework Integration**

graph-sql works seamlessly with popular Rust web frameworks:

- **Axum** ✅ (shown in examples)
- **Actix-web** ✅ (via async-graphql-actix-web)
- **Warp** ✅ (via async-graphql-warp)
- **Tide** ✅ (via async-graphql-tide)

### **Use Cases**

Perfect for:
- ⚡ **High-performance APIs** - Memory-safe GraphQL gateway for heavy-load scenarios
- � **Secure data services** - Rust's memory safety eliminates common vulnerabilities
- 🏗️ **Microservices architecture** - Stateless gateway enabling horizontal scaling
- �🛠️ **Admin panels** - Auto-generated CRUD interfaces for content management
- 📊 **Data exploration** - Interactive GraphiQL interface for database exploration
- 🔄 **Legacy modernization** - Add secure GraphQL layer to existing SQLite applications
- 🏭 **Production workloads** - Single binary deployment for enterprise environments
- 📱 **Mobile backends** - High-performance API generation for mobile applications

## 📖 How It Works

graph-sql automatically transforms your SQLite databases into modern GraphQL services.

## 🚀 Features

- **Memory Safety**: Rust's zero-cost abstractions eliminate buffer overflows, memory leaks, and other common API server vulnerabilities
- **High Performance**: Designed for heavy-load scenarios with minimal resource overhead and efficient concurrency
- **Zero Configuration**: Automatically introspects your SQLite database structure with no setup required
- **Database-First Architecture**: Business logic lives in SQLite, not the application layer, for better performance and consistency
- **Stateless Gateway**: Pure middleman design enabling horizontal scaling and simple deployment
- **Full CRUD Operations**: Complete Create, Read, Update, Delete support through GraphQL mutations and queries
- **Foreign Key Relationships**: Automatic detection and mapping of foreign key relationships to GraphQL object relationships
- **Type-Safe Schema**: Generates GraphQL types that match your database schema with proper nullability
- **Dynamic Schema Generation**: Creates resolvers and types at runtime based on database introspection
- **Built-in GraphiQL**: Interactive GraphQL playground included for development and testing
- **Single Binary Deployment**: No runtime dependencies or complex installation requirements
- **SQLite Extensions**: Future support for sqlean and other SQLite extensions for advanced functionality

## 📋 Prerequisites

- Rust 1.86.0+ (2024 edition)
- SQLite database (or let graph-sql create one for you)

## 🚀 Quick Start

### **CLI Tool**

```bash
# Install globally
cargo install --git https://github.com/karlrobeck/graph-sql.git

# Serve any database instantly
graph-sql serve -d "sqlite://my_app.db"
# Open http://localhost:8000 for GraphiQL interface
```

### **Library Integration**

**Basic setup** in your `main.rs`:

```rust
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{Router, response::{Html, IntoResponse}};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Connect to your SQLite database
    let db = SqlitePool::connect("sqlite://your_database.db").await?;
    
    // Let graph-sql introspect and generate the schema
    let schema = graph_sql::introspect(&db).await?.finish()?;
    
    // Set up your GraphQL server
    let router = Router::new().route(
        "/",
        axum::routing::get(graphiql).post_service(GraphQL::new(schema)),
    );
    
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, router).await?;
    
    Ok(())
}
```

That's it! graph-sql automatically:
- 🔍 **Introspects your database** schema
- 🏗️ **Generates GraphQL types** for all tables  
- 🔗 **Maps foreign keys** to GraphQL relationships
- ⚡ **Creates CRUD resolvers** for all operations

## 📚 Examples

Perfect for high-performance APIs, secure data gateways, and transforming existing databases into scalable GraphQL services with memory-safe guarantees.

🚀 **NEW**: graph-sql is now available as a standalone CLI application! Install it globally and instantly serve any SQLite database as a GraphQL API.

For detailed documentation and more queries, see the [examples directory](./examples/).

## 📦 Installation

### **CLI Application (Recommended)**

Install graph-sql globally using cargo:

```bash
cargo install --git https://github.com/karlrobeck/graph-sql.git
```

**Quick Start**:
```bash
# Serve any SQLite database as GraphQL API
graph-sql serve -d "sqlite://my_database.db" -p 8000

# Introspect and view schema
graph-sql introspect -d "sqlite://my_database.db"

# With migrations
graph-sql serve -d "sqlite://my_database.db" -m "./migrations"
```

### **CLI Usage**

```bash
# Start GraphQL server (creates local.db if not exists)
graph-sql serve

# Custom database and port
graph-sql serve -d "sqlite://data/app.db" -p 3000 --host 127.0.0.1

# Run with migrations
graph-sql serve -d "sqlite://app.db" -m "./migrations"

# Introspect schema to stdout
graph-sql introspect -d "sqlite://app.db"

# Save schema to file
graph-sql introspect -d "sqlite://app.db" -o schema.graphql

# Use environment variable
export DATABASE_URL="sqlite://production.db"
graph-sql serve  # Uses DATABASE_URL
```

**CLI Features**:
- 🗄️ **Automatic database creation** - Creates SQLite files if they don't exist
- 🔧 **Migration support** - Optional migration directory with `-m` flag
- 🌍 **Environment variables** - Uses `DATABASE_URL` environment variable
- 📄 **Schema export** - Export GraphQL schema to files with `introspect` command
- ⚙️ **Flexible configuration** - Customize host, port, and database URL

### **As a Library**

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

## 🔧 Library API

When using graph-sql as a library, it provides a simple, elegant API for integrating GraphQL into your Rust applications:

### **Core Functions**

```rust
// Main introspection function
pub async fn introspect(pool: &SqlitePool) -> Result<SchemaBuilder, Error>

// Schema builder for customization
impl SchemaBuilder {
    pub fn finish(self) -> Result<Schema<Query, Mutation, EmptySubscription>, Error>
    // Additional customization methods available
}
```

### **Integration Patterns**

**🔥 Minimal Setup** (3 lines):
```rust
let db = SqlitePool::connect("sqlite://app.db").await?;
let schema = graph_sql::introspect(&db).await?.finish()?;
let app = Router::new().route("/graphql", post_service(GraphQL::new(schema)));
```

**🛠️ With Custom Configuration**:
```rust
let schema = graph_sql::introspect(&db)
    .await?
    // Add custom resolvers, middleware, etc.
    .finish()?;
```


**🔄 With Hot Reloading** (Development):
```rust
// Reintrospect when schema changes
let schema = graph_sql::introspect(&db).await?.finish()?;
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT OR Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [async-graphql](https://github.com/async-graphql/async-graphql)
- Powered by [SQLx](https://github.com/launchbadge/sqlx)
- Web framework by [Axum](https://github.com/tokio-rs/axum)
- Query building with [SeaQuery](https://github.com/SeaQL/sea-query)

## 📞 Support

- Create an [issue](https://github.com/karlrobeck/graph-sql/issues) for bug reports
- Start a [discussion](https://github.com/karlrobeck/graph-sql/discussions) for questions

---

**graph-sql** - Turning your SQLite database into a full-featured GraphQL API, effortlessly.
