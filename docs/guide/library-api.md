# Library API

When using graph-sql as a library, it provides a simple, configurable API
through the `GraphSQL` struct for integrating GraphQL into your Rust
applications.

## GraphSQL API

### GraphSQL Struct

The `GraphSQL` struct provides a complete, configurable GraphQL server setup:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};

let config = GraphSQLConfig::default();
let graph_sql = GraphSQL::new(config);
```

### Main Methods

```rust
impl GraphSQL {
    pub fn new(config: GraphSQLConfig) -> Self
    pub async fn introspect(&self, db: &SqlitePool) -> async_graphql::Result<Vec<CreateTable>>
    pub fn build_schema(&self, tables: Vec<CreateTable>) -> async_graphql::Result<SchemaBuilder>
    pub async fn build(&self, db: &SqlitePool) -> async_graphql::Result<(Router, TcpListener)>
}
```

**Complete Example**:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    let db = SqlitePool::connect("sqlite://app.db").await?;
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    
    let (router, listener) = graph_sql.build(&db).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
```

## Integration Patterns

### Minimal Setup

The fastest way to get started with a complete GraphQL server:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    let db = SqlitePool::connect("sqlite://app.db").await?;
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    
    let (router, listener) = graph_sql.build(&db).await?;
    println!("GraphiQL: http://localhost:8080");
    axum::serve(listener, router).await?;
    Ok(())
}
```

### Complete Setup with GraphiQL

A more complete setup including the GraphiQL interface with custom routing:

```rust
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{Router, response::{Html, IntoResponse}};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use graph_sql::{GraphSQL, config::GraphSQLConfig};

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    let db = SqlitePool::connect("sqlite://app.db").await?;
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    
    // Get the schema for custom routing
    let tables = graph_sql.introspect(&db).await?;
    let schema = graph_sql.build_schema(tables)?.finish()?;
    
    let router = Router::new()
        .route("/", axum::routing::get(graphiql))
        .route("/graphql", axum::routing::post_service(GraphQL::new(schema)));
    
    println!("GraphiQL: http://localhost:8080");
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router).await?;
    
    Ok(())
}
```

### With Custom Configuration

For more control over your setup:

```rust
use graph_sql::{GraphSQL, config::{GraphSQLConfig, ServerConfig, DatabaseConfig, GraphQLConfig}};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // Configure database connection
    let db = SqlitePool::connect("sqlite://app.db").await?;
    
    // Run migrations if needed
    sqlx::migrate!("./migrations").run(&db).await?;
    
    // Custom configuration
    let config = GraphSQLConfig {
        server: ServerConfig {
            host: "localhost".to_string(),
            port: 3000,
        },
        database: DatabaseConfig {
            url: "sqlite://app.db".to_string(),
            ..Default::default()
        },
        graphql: GraphQLConfig {
            enable_playground: Some(true),
            ..Default::default()
        },
    };
    
    let graph_sql = GraphSQL::new(config);
    let (router, listener) = graph_sql.build(&db).await?;
    
    println!("GraphiQL: http://localhost:3000");
    axum::serve(listener, router).await?;
    Ok(())
}
```

### Step-by-Step Integration

For understanding the full process using the GraphSQL struct:

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    // 1. Connect to database
    let db = SqlitePool::connect("sqlite://app.db").await?;
    
    // 2. Create configuration
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    
    // 3. Introspect database (returns CREATE TABLE statements)
    let tables = graph_sql.introspect(&db).await?;
    println!("Found {} tables", tables.len());
    
    // 4. Build GraphQL schema
    let schema_builder = graph_sql.build_schema(tables)?;
    let schema = schema_builder.finish()?;
    
    // 5. Create router and listener manually if needed
    // Or use the convenience method:
    let (router, listener) = graph_sql.build(&db).await?;
    axum::serve(listener, router).await?;
    
    Ok(())
}
```

### Hot Reloading (Development)

For development environments where you want to reload the schema when the
database changes:

```rust
use std::time::Duration;
use tokio::time::sleep;
use graph_sql::{GraphSQL, config::GraphSQLConfig};

async fn create_schema(graph_sql: &GraphSQL, db: &SqlitePool) -> async_graphql::Result<async_graphql::Schema<async_graphql::dynamic::Object, async_graphql::dynamic::Object, async_graphql::EmptySubscription>> {
    let tables = graph_sql.introspect(db).await?;
    graph_sql.build_schema(tables)?.finish()
}

#[tokio::main]
async fn main() -> async_graphql::Result<()> {
    let db = SqlitePool::connect("sqlite://app.db").await?;
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    
    // In development, you might want to periodically re-introspect
    // This is just an example - you'd typically use file watching instead
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;
            // Re-introspect and update schema...
        }
    });
    
    let schema = create_schema(&graph_sql, &db).await?;
    // Rest of setup...
    Ok(())
}
```

## Framework Integration

graph-sql works seamlessly with popular Rust web frameworks through the
async-graphql ecosystem:

### Axum ✅

Using the GraphSQL API (recommended):

```rust
use graph_sql::{GraphSQL, config::GraphSQLConfig};

let config = GraphSQLConfig::default();
let graph_sql = GraphSQL::new(config);
let (router, listener) = graph_sql.build(&db).await?;
```

For custom routing, you can build the schema manually:

```rust
use async_graphql_axum::GraphQL;
use axum::{Router, routing::post_service};

let config = GraphSQLConfig::default();
let graph_sql = GraphSQL::new(config);
let tables = graph_sql.introspect(&db).await?;
let schema = graph_sql.build_schema(tables)?.finish()?;
let app = Router::new()
    .route("/graphql", post_service(GraphQL::new(schema)));
```

### Actix-web ✅

```rust
use async_graphql_actix_web::{GraphQL, GraphQLSubscription};
use actix_web::{web, App, HttpServer};
use graph_sql::{GraphSQL, config::GraphSQLConfig};

let config = GraphSQLConfig::default();
let graph_sql = GraphSQL::new(config);
let tables = graph_sql.introspect(&db).await?;
let schema = graph_sql.build_schema(tables)?.finish()?;

HttpServer::new(move || {
    App::new()
        .service(web::resource("/graphql").to(GraphQL::new(schema.clone())))
})
```

### Warp ✅

```rust
use async_graphql_warp::GraphQL;
use warp::Filter;
use graph_sql::{GraphSQL, config::GraphSQLConfig};

let config = GraphSQLConfig::default();
let graph_sql = GraphSQL::new(config);
let tables = graph_sql.introspect(&db).await?;
let schema = graph_sql.build_schema(tables)?.finish()?;

let graphql_post = async_graphql_warp::graphql(schema)
    .and_then(|(schema, request): (MySchema, async_graphql::Request)| async move {
        Ok::<_, Infallible>(async_graphql_warp::Response::from(schema.execute(request).await))
    });
```

### Tide ✅

```rust
use async_graphql_tide::GraphQL;
use tide::Request;
use graph_sql::{GraphSQL, config::GraphSQLConfig};

let config = GraphSQLConfig::default();
let graph_sql = GraphSQL::new(config);
let tables = graph_sql.introspect(&db).await?;
let schema = graph_sql.build_schema(tables)?.finish()?;

let mut app = tide::new();
app.at("/graphql").post(GraphQL::new(schema));
```

## Use Cases

The library API is perfect for:

### High-Performance APIs

Using the GraphSQL API with custom configuration:

```rust
use graph_sql::{GraphSQL, config::{GraphSQLConfig, ServerConfig}};

// Optimized for heavy load scenarios
let config = GraphSQLConfig {
    server: ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
    },
    ..Default::default()
};

let graph_sql = GraphSQL::new(config);
let (router, listener) = graph_sql.build(&db).await?;

// The router includes built-in optimizations and DataLoader integration
axum::serve(listener, router).await?;
```

### Microservices Architecture

```rust
// Stateless gateway enabling horizontal scaling
async fn create_graphql_service(database_url: &str) -> Result<(axum::Router, tokio::net::TcpListener), Box<dyn std::error::Error>> {
    let db = SqlitePool::connect(database_url).await?;
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    
    Ok(graph_sql.build(&db).await?)
}
```

### Admin Panels

```rust
// Auto-generated CRUD interfaces for content management
let config = GraphSQLConfig {
    graphql: GraphQLConfig {
        enable_playground: Some(true), // Enable GraphiQL for admin interface
        ..Default::default()
    },
    ..Default::default()
};

let graph_sql = GraphSQL::new(config);
let (router, listener) = graph_sql.build(&db).await?;
        .subscription_endpoint("/graphql/ws")
        .finish())
}
```

### Data Exploration

```rust
// Interactive GraphiQL interface for database exploration
let config = GraphSQLConfig {
    graphql: GraphQLConfig {
        enable_playground: Some(true),
        ..Default::default()
    },
    ..Default::default()
};

let graph_sql = GraphSQL::new(config);
let (router, listener) = graph_sql.build(&db).await?;
// Router automatically includes GraphiQL at "/" when playground is enabled
```

## Error Handling

The library uses standard Rust error handling patterns:

```rust
use async_graphql::Result as GraphQLResult;
use graph_sql::{GraphSQL, config::GraphSQLConfig};

#[tokio::main]
async fn main() -> GraphQLResult<()> {
    let db = SqlitePool::connect("sqlite://app.db").await
        .map_err(|e| async_graphql::Error::new(format!("Database connection failed: {}", e)))?;
    
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    
    let (router, listener) = graph_sql.build(&db).await?;
    
    axum::serve(listener, router).await
        .map_err(|e| async_graphql::Error::new(format!("Server failed: {}", e)))?;
    
    Ok(())
}
```

## Performance Considerations

### Connection Pooling

```rust
use sqlx::sqlite::SqlitePoolOptions;

// Configure connection pool for optimal performance
let db = SqlitePoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .connect("sqlite://app.db").await?;

let config = GraphSQLConfig::default();
let graph_sql = GraphSQL::new(config);
let (router, listener) = graph_sql.build(&db).await?;
```

### Built-in Optimizations

The `GraphSQL` struct includes several built-in optimizations:

```rust
// DataLoader is automatically configured for N+1 query prevention
// Database connection is shared across all resolvers
// Schema is built once and reused

let graph_sql = GraphSQL::new(config);
let (router, listener) = graph_sql.build(&db).await?;
// Router includes all optimizations out of the box
```

### Custom Schema Caching

For advanced use cases, you can cache schemas manually:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use graph_sql::{GraphSQL, config::GraphSQLConfig};

// Cache the schema to avoid re-introspection on every request
static SCHEMA_CACHE: std::sync::OnceLock<Arc<RwLock<Option<async_graphql::Schema<async_graphql::dynamic::Object, async_graphql::dynamic::Object, async_graphql::EmptySubscription>>>>> = std::sync::OnceLock::new();

async fn get_or_create_schema(db: &SqlitePool) -> async_graphql::Result<async_graphql::Schema<async_graphql::dynamic::Object, async_graphql::dynamic::Object, async_graphql::EmptySubscription>> {
    let cache = SCHEMA_CACHE.get_or_init(|| Arc::new(RwLock::new(None)));
    
    {
        let read_guard = cache.read().await;
        if let Some(schema) = read_guard.as_ref() {
            return Ok(schema.clone());
        }
    }
    
    let config = GraphSQLConfig::default();
    let graph_sql = GraphSQL::new(config);
    let tables = graph_sql.introspect(db).await?;
    let schema = graph_sql.build_schema(tables)?.finish()?;
    
    {
        let mut write_guard = cache.write().await;
        *write_guard = Some(schema.clone());
    }
    
    Ok(schema)
}
```

## Next Steps

- Learn about [Configuration](/guide/configuration) options for different
  environments
- Check out [Examples](/guide/examples) for complete working applications
- Explore [Deployment](/guide/deployment) strategies for production use
