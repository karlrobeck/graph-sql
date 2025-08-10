# Examples

The graph-sql repository includes several comprehensive examples that
demonstrate real-world usage patterns. Each example shows how to use graph-sql
as a library with complete database setup and migrations.

## Running Examples

The examples are built as Rust binaries that can be run using cargo. To run any
of the included examples:

```bash
# Run examples from the project root directory
cargo run --example blog

# Run other examples
cargo run --example ecommerce
cargo run --example library
cargo run --example tasks

# Open http://localhost:8080 for GraphiQL interface
```

**Alternative method** (running from example directories):

```bash
# Navigate to an example directory
cd examples/blog

# Run the example (it will create the database and run migrations)
cargo run

# Open http://localhost:8080 for GraphiQL interface
```

Each example demonstrates library usage with:

- Automatic database creation
- Built-in migration handling
- Custom GraphQL server setup
- Interactive GraphiQL interface

## Available Examples

### Blog System

**Location**: `examples/blog/`

A complete blog content management system with relationships between posts,
authors, and comments.

**Features**:

- User management with authentication
- Post creation and editing
- Nested comment system
- Author-post relationships
- Category management

**Schema**:

```sql
-- Users table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Posts table with foreign key to users
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id)
);

-- Comments with self-referencing foreign key
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    post_id INTEGER NOT NULL,
    parent_comment_id INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id),
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (parent_comment_id) REFERENCES comments(id)
);
```

**Example Queries**:

```graphql
# Get all posts with authors and comments
query {
  posts {
    id
    title
    content
    author {
      name
      email
    }
    comments {
      content
      author {
        name
      }
    }
  }
}
```

### E-commerce System

**Location**: `examples/ecommerce/`

A comprehensive product catalog system with categories, variants, and order
management.

**Features**:

- Product catalog with categories
- Product variants (size, color, etc.)
- Order management system
- Customer management
- Inventory tracking

**Schema highlights**:

- Products with categories
- Product variants with different prices
- Orders with line items
- Customer relationships

### Library Management

**Location**: `examples/library/`

A book lending system with authors, books, and borrowing records.

**Features**:

- Book catalog with author information
- Member management
- Book borrowing and returns
- Due date tracking
- Author-book relationships

**Schema highlights**:

- Books with multiple authors (many-to-many)
- Borrowing records with due dates
- Member management
- Late fee calculations

### Task Management

**Location**: `examples/tasks/`

A todo application with users, projects, and task assignments.

**Features**:

- Project organization
- Task assignment to users
- Priority and status tracking
- Due date management
- Team collaboration

**Schema highlights**:

- Projects with team members
- Tasks with assignees
- Priority and status enums
- Deadline tracking

## Schema Patterns Demonstrated

### Foreign Key Relationships

All examples show how graph-sql automatically detects and maps foreign key
relationships:

```sql
-- One-to-many: author_id in posts table
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users(id)
);
```

Becomes:

```graphql
type Post {
  id: ID!
  title: String!
  author: User!  # Automatic relationship
}

type User {
  id: ID!
  name: String!
  posts: [Post!]!  # Reverse relationship
}
```

### Many-to-Many Relationships

Junction tables create array relationships:

```sql
-- Many-to-many via junction table
CREATE TABLE post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id),
    PRIMARY KEY (post_id, tag_id)
);
```

### Self-Referencing Relationships

Perfect for nested structures like comments:

```sql
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    parent_comment_id INTEGER,
    FOREIGN KEY (parent_comment_id) REFERENCES comments(id)
);
```

## Custom Example Setup

To create your own example:

1. **Create a new directory**:

```bash
mkdir examples/my-app
cd examples/my-app
```

**Alternative**: Create as a root-level example that can be run with
`cargo run --example my-app`:

```bash
# Create example file in examples directory
touch examples/my-app.rs
```

2. **Create Cargo.toml** (if creating as subdirectory):

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
graph-sql = { path = "../.." }
async-graphql = "7.0.17"
tokio = { version = "1.47.0", features = ["full"] }
anyhow = "1.0"
sqlx = { version = "0.8.6", features = ["runtime-tokio-native-tls", "sqlite", "migrate"] }
```

3. **Create main.rs**:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Create configuration programmatically
    let config = GraphSQLConfig {
        server: graph_sql::config::ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
        },
        database: graph_sql::config::DatabaseConfig {
            use_env: Some(false),
            database_url: Some("sqlite://my_app.db".to_string()),
            migration_path: None,
            sqlite: None,
        },
        graphql: graph_sql::config::GraphQLConfig {
            enable_uploading: Some(true),
            enable_playground: Some(true),
            limit_depth: Some(10),
            limit_complexity: Some(100),
            limit_recursive_depth: None,
            limit_directives: None,
            disable_suggestions: None,
            disable_introspection: None,
            introspection_only: None,
            enable_federation: None,
        },
    };
    
    // Create database connection
    let pool = config.database.create_connection().await?;
    
    // Run migrations using sqlx (runs separately from graph-sql)
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create GraphSQL instance
    let graph_sql = GraphSQL::new(config);
    
    // Build the router and listener
    let (router, listener) = graph_sql.build(&pool).await?;
    
    println!("GraphiQL: http://localhost:8080");
    axum::serve(listener, router.into_make_service()).await?;
    
    Ok(())
}
```

**Alternative with config file**:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Load config from file
    let config = GraphSQLConfig::from_path("graph_sql.toml")?;
    
    // Create database connection
    let pool = config.database.create_connection().await?;
    
    // Run migrations using sqlx (independent of graph-sql config)
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    // Create GraphSQL instance and build server
    let graph_sql = GraphSQL::new(config);
    let (router, listener) = graph_sql.build(&pool).await?;
    
    println!("GraphiQL: http://localhost:8080");
    axum::serve(listener, router.into_make_service()).await?;
    
    Ok(())
}
```

4. **Create graph_sql.toml** (for config file approach):

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
use-env = false

[database.sqlite]
filename = "my_app.db"
foreign-keys = true
create-if-missing = true

[graphql]
enable-playground = true
limit-depth = 10
limit-complexity = 100
```

5. **Create migrations**:

```bash
mkdir migrations
# Add your SQL migration files (e.g., 001_initial.sql, 002_add_users.sql)
```

**Migration Support**:

- **graph-sql CLI**: Built-in migration support is not yet implemented
- **Library mode**: You can use `sqlx::migrate!` directly for migrations
- **Manual approach**: Apply SQL files directly to SQLite

**Using sqlx migrations in library mode**:

```sql
-- migrations/001_initial.sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL
);

CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users(id)
);
```

The `sqlx::migrate!("./migrations").run(&pool).await?;` line in your code will
automatically apply these migrations.

**Manual schema application** (alternative approach):

```bash
# Create and apply schema manually
sqlite3 my_app.db < schema.sql

# Or interactively
sqlite3 my_app.db
.read schema.sql
.exit
```

## Best Practices from Examples

1. **Use proper foreign key constraints** for automatic relationship generation
2. **Include timestamps** for audit trails
3. **Use meaningful table and column names** (they become GraphQL field names)
4. **Leverage SQLite features** like CHECK constraints and triggers
5. **Design for the database-first philosophy** - put business logic in SQLite
6. **Use configuration files** for different environments (dev, staging, prod)
7. **Take advantage of SQLite performance options** in the config
8. **Use the GraphSQL builder pattern** for consistent server setup
9. **Use sqlx migrations in library mode** for automated schema management
10. **Separate migration logic from graph-sql setup** - run migrations before
    creating GraphSQL instance

## Running Your Custom Example

Once you've created your custom example, you can run it:

```bash
# If created as subdirectory
cd examples/my-app
cargo run

# If created as single file in examples/
cargo run --example my-app

# With custom config
cargo run -- -c custom-config.toml  # (from subdirectory)
```

## Configuration-First Development

The new API encourages configuration-first development:

```rust
// Option 1: Programmatic configuration
let config = GraphSQLConfig {
    // ... configure all options
};

// Option 2: File-based configuration
let config = GraphSQLConfig::from_path("graph_sql.toml")?;

// Same setup for both approaches
let pool = config.database.create_connection().await?;

// Optional: Run migrations with sqlx (separate from graph-sql)
sqlx::migrate!("./migrations").run(&pool).await?;

let graph_sql = GraphSQL::new(config);
let (router, listener) = graph_sql.build(&pool).await?;
```

This approach provides:

- **Consistent configuration** across different environments
- **Type-safe configuration** with validation
- **Flexible migration options** - use sqlx migrations or manual schema
  management
- **Clear separation of concerns** - migrations run independently of GraphQL
  setup
- **Easy testing** with different config options
- **Production-ready setup** out of the box

4. **Leverage SQLite features** like CHECK constraints and triggers
5. **Design for the database-first philosophy** - put business logic in SQLite
