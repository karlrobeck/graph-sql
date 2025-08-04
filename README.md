# graph-sql

A high-performance Rust CLI tool and library that automatically introspects
SQLite databases and generates complete GraphQL APIs with TOML-based
configuration. Built as a memory-safe alternative to traditional GraphQL
servers, graph-sql acts as a lightweight gateway that pushes business logic to
the database layer for maximum performance and simplicity.

🚀 **NEW**: graph-sql is now available as a standalone CLI application with TOML
configuration! Create a simple config file and instantly serve any SQLite
database as a GraphQL API.

🔒 **Memory Safety**: Leverages Rust's zero-cost abstractions and memory safety
guarantees to eliminate entire classes of bugs common in traditional API
servers.

🏗️ **Database-First Architecture**: Acts as a stateless gateway/middleman,
letting SQLite handle business logic, authorization, and data processing for
optimal performance.

For detailed documentation and more queries, see the
[examples directory](./examples/).

> **⚠️ Development Status**\
> **This project is in active development.** Breaking changes may occur without
> notice as we rapidly iterate and improve the library. While the core
> functionality is stable, the API may evolve significantly. For production use,
> please pin to a specific commit and thoroughly test any updates.

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
  - [Configuration Structure](#configuration-structure)
  - [Configuration Options](#configuration-options)
  - [Environment Variables](#environment-variables)
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
# Serve any SQLite database as GraphQL API (uses default config.toml)
graph-sql serve

# Use custom config file
graph-sql serve -c my-config.toml

# Introspect and view schema
graph-sql introspect

# Save schema to file
graph-sql introspect -o schema.graphql
```

### **CLI Usage**

The CLI now uses TOML-based configuration for all settings. Create a
`config.toml` file in your project directory:

```toml
[server]
host = "0.0.0.0"
port = 8000

[database]
database-url = "sqlite://my_database.db"
use-env = true  # Use DATABASE_URL environment variable if available
migrations-path = "./migrations"

[graphql]
enable-playground = true
depth = 5
complexity = 5
```

**Basic Commands**:

```bash
# Start GraphQL server (uses config.toml in current directory)
graph-sql serve

# Use custom config file
graph-sql serve -c /path/to/my-config.toml

# Introspect schema to stdout
graph-sql introspect

# Save schema to file
graph-sql introspect -o schema.graphql

# Use environment variable (set use-env = true in config)
export DATABASE_URL="sqlite://production.db"
graph-sql serve
```

**CLI Features**:

- 📋 **TOML Configuration** - All settings defined in structured config files
- 🗄️ **Automatic database creation** - Creates SQLite files if they don't exist
- 🔧 **Migration support** - Optional migration directory path in config
- 🌍 **Environment variables** - Uses `DATABASE_URL` when `use-env = true`
- 📄 **Schema export** - Export GraphQL schema to files with `introspect`
  command
- ⚙️ **Flexible configuration** - Customize host, port, database URL, and
  GraphQL settings

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

When using graph-sql as a library, it provides a simple, elegant API for
integrating GraphQL into your Rust applications:

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

- ⚡ **High-performance APIs** - Memory-safe GraphQL gateway for heavy-load
  scenarios
- � **Secure data services** - Rust's memory safety eliminates common
  vulnerabilities
- 🏗️ **Microservices architecture** - Stateless gateway enabling horizontal
  scaling
- �🛠️ **Admin panels** - Auto-generated CRUD interfaces for content management
- 📊 **Data exploration** - Interactive GraphiQL interface for database
  exploration
- 🔄 **Legacy modernization** - Add secure GraphQL layer to existing SQLite
  applications
- 🏭 **Production workloads** - Single binary deployment for enterprise
  environments
- 📱 **Mobile backends** - High-performance API generation for mobile
  applications

## 📖 How It Works

graph-sql automatically transforms your SQLite databases into modern GraphQL
services.

## 🚀 Features

- **Memory Safety**: Rust's zero-cost abstractions eliminate buffer overflows,
  memory leaks, and other common API server vulnerabilities
- **High Performance**: Designed for heavy-load scenarios with minimal resource
  overhead and efficient concurrency
- **TOML Configuration**: Simple, structured configuration files for all server
  and database settings
- **Database-First Architecture**: Business logic lives in SQLite, not the
  application layer, for better performance and consistency
- **Stateless Gateway**: Pure middleman design enabling horizontal scaling and
  simple deployment
- **Full CRUD Operations**: Complete Create, Read, Update, Delete support
  through GraphQL mutations and queries
- **Foreign Key Relationships**: Automatic detection and mapping of foreign key
  relationships to GraphQL object relationships
- **Type-Safe Schema**: Generates GraphQL types that match your database schema
  with proper nullability
- **Dynamic Schema Generation**: Creates resolvers and types at runtime based on
  database introspection
- **Built-in GraphiQL**: Interactive GraphQL playground included for development
  and testing
- **Single Binary Deployment**: No runtime dependencies or complex installation
  requirements
- **SQLite Extensions**: Future support for sqlean and other SQLite extensions
  for advanced functionality

## 📋 Prerequisites

- Rust 1.86.0+ (2024 edition)
- SQLite database (or let graph-sql create one for you)

## 🚀 Quick Start

### **CLI Tool**

```bash
# Install globally
cargo install --git https://github.com/karlrobeck/graph-sql.git

# Create a config.toml file
cat > config.toml << EOF
[server]
host = "0.0.0.0"
port = 8000

[database]
database-url = "sqlite://my_app.db"
use-env = true

[graphql]
enable-playground = true
EOF

# Serve any database instantly
graph-sql serve
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

For detailed documentation and more queries, see the
[examples directory](./examples/).

### **Running Examples**

Each example includes a `config.toml` file with the appropriate database
configuration. To run the included examples:

```bash
# Navigate to an example directory
cd examples/blog

# The example already has a config.toml configured
# Run the example
cargo run

# Open http://localhost:8000 for GraphiQL interface
```

**Example config.toml structure**:

```toml
[server]
host = "0.0.0.0"
port = 8000

[database]
database-url = "sqlite://blog.db"
use-env = false
migrations-path = "./migrations"

[graphql]
enable-playground = true
```

### **Example Overview**

- **Blog System** - Complete blog with posts, authors, and comments
- **E-commerce** - Product catalog with categories and orders
- **Library** - Book management with authors and borrowing
- **Task Manager** - Todo application with users and assignments

### **Quick Example - Blog System**

The blog example demonstrates a complete content management system with
relationships between posts, authors, and comments.

## 🏗️ GraphQL Schema Structure

graph-sql automatically generates GraphQL types based on your SQLite schema:

- **Tables** → GraphQL Object Types
- **Columns** → GraphQL Fields with appropriate types
- **Foreign Keys** → GraphQL Object Relationships
- **Primary Keys** → ID fields
- **Nullable Columns** → Optional GraphQL fields

## 🗄️ Database Schema Mapping

| SQLite Type | GraphQL Type |
| ----------- | ------------ |
| INTEGER     | Int          |
| TEXT        | String       |
| REAL        | Float        |
| BLOB        | String       |
| BOOLEAN     | Boolean      |

## 📖 Example Usage

Once your server is running, you can execute GraphQL queries:

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

## ⚙️ Configuration

graph-sql uses TOML configuration files for all settings. The CLI looks for
`config.toml` in the current directory by default, or you can specify a custom
path with `-c`.

### **Configuration Structure**

```toml
[server]
host = "0.0.0.0"        # Server bind address
port = 8000             # Server port

[database]
database-url = "sqlite://local.db"  # Database connection string
use-env = true                      # Use DATABASE_URL env var if available
migrations-path = "./migrations"    # Optional path to migration files

[graphql]
enable-playground = true            # Enable GraphiQL interface
depth = 5                          # Query depth limit
complexity = 5                     # Query complexity limit
```

### **Configuration Options**

**Server Section**:

- `host` - Server bind address (default: "0.0.0.0")
- `port` - Server port number (default: 8000)

**Database Section**:

- `database-url` - SQLite database connection string
- `use-env` - If true, uses `DATABASE_URL` environment variable when available
- `migrations-path` - Optional directory containing SQL migration files

**GraphQL Section** (optional):

- `enable-playground` - Enable GraphiQL interactive interface (default: true)
- `depth` - Maximum query depth allowed (default: 5)
- `complexity` - Maximum query complexity allowed (default: 5)

### **Environment Variables**

Set `use-env = true` in your config to enable environment variable support:

```bash
export DATABASE_URL="sqlite://production.db"
graph-sql serve  # Uses DATABASE_URL instead of config database-url
```

## 🏛️ Architecture

graph-sql follows a database-first architecture:

1. **Introspection Layer** - Analyzes SQLite schema
2. **Type Generation** - Creates GraphQL types from database structure
3. **Resolver Generation** - Builds CRUD operations automatically
4. **Gateway Layer** - Stateless GraphQL server

## 🛠️ Development

To contribute to graph-sql:

```bash
# Clone the repository
git clone https://github.com/karlrobeck/graph-sql.git
cd graph-sql

# Create a development config file
cat > config.toml << EOF
[server]
host = "0.0.0.0"
port = 8000

[database]
database-url = "sqlite://local.db"
use-env = true

[graphql]
enable-playground = true
depth = 5
complexity = 5
EOF

# Run tests
cargo test

# Run the main CLI application
cargo run -- serve

# Run examples
cd examples/blog
cargo run
```

## ⚠️ Current Limitations

- SQLite only (PostgreSQL and MySQL support planned)
- Basic authentication (OAuth and JWT support planned)
- Limited custom scalar types
- No subscription support yet

## 🗺️ Roadmap

- [ ] PostgreSQL and MySQL support
- [ ] Advanced authentication and authorization
- [ ] Custom scalar types
- [ ] GraphQL subscriptions
- [ ] Performance optimizations
- [ ] Docker containerization
- [ ] Cloud deployment guides

## 🤝 Contributing

We welcome contributions! Please see our
[Contributing Guidelines](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT OR Apache-2.0 License - see the
[LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [async-graphql](https://github.com/async-graphql/async-graphql)
- Powered by [SQLx](https://github.com/launchbadge/sqlx)
- Web framework by [Axum](https://github.com/tokio-rs/axum)
- Query building with [SeaQuery](https://github.com/SeaQL/sea-query)

## 📞 Support

- Create an [issue](https://github.com/karlrobeck/graph-sql/issues) for bug
  reports
- Start a [discussion](https://github.com/karlrobeck/graph-sql/discussions) for
  questions

---

**graph-sql** - Turning your SQLite database into a full-featured GraphQL API,
effortlessly.
