# Development

This guide covers how to set up a development environment for contributing to
graph-sql, understanding the codebase, and extending functionality.

## Development Setup

### Prerequisites

- **Rust 1.86.0+** with 2024 edition support
- **SQLite 3.35+** (usually bundled with sqlx)
- **Git** for version control

### Clone and Setup

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
depth = 10
complexity = 50
EOF

# Build the project
cargo build

# Run tests
cargo test

# Run the main CLI application
cargo run -- serve

# Open http://localhost:8000 for GraphiQL interface
```

## Project Structure

```
graph-sql/
├── src/                 # Core library code
│   ├── cli.rs          # CLI argument parsing and commands
│   ├── config.rs       # Configuration management
│   ├── lib.rs          # Main library entry point and introspection
│   ├── loader.rs       # Data loading utilities
│   ├── main.rs         # CLI binary entry point
│   ├── parser.rs       # SQL and schema parsing utilities
│   ├── resolvers.rs    # GraphQL resolver generation
│   ├── traits.rs       # Trait implementations for type generation
│   └── utils.rs        # Utility functions
├── examples/           # Example applications
│   ├── blog/          # Blog system example
│   ├── ecommerce/     # E-commerce example
│   ├── library/       # Library management example
│   └── tasks/         # Task management example
├── docs/              # Documentation (RSPress)
├── Cargo.toml         # Main project dependencies
├── config.toml        # Default configuration
└── README.md          # Project documentation
```

## Core Components

### 1. Introspection (`src/lib.rs`)

The main entry point that analyzes SQLite databases:

```rust
// Main introspection function
pub async fn introspect(pool: &SqlitePool) -> Result<SchemaBuilder, Error>

// Key data structures
pub struct SqliteTable {
    pub table_info: TableInfo,
    pub columns: Vec<ColumnDef>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}
```

**Development patterns**:

- Add `#[instrument(skip(db), level = "debug")]` to new functions
- Use `tracing` for structured logging
- Always use parameterized queries with `sqlx`

### 2. Type Generation (`src/traits.rs`)

Converts SQLite schema to GraphQL types:

```rust
// Trait for converting SQL tables to GraphQL
trait ToGraphQLType {
    fn to_object_type(&self, schema: &mut SchemaBuilder) -> ObjectType;
    fn to_input_type(&self, schema: &mut SchemaBuilder) -> InputObjectType;
}
```

**Adding new type mappings**:

1. Update the type mapping in `map_sqlite_type_to_graphql`
2. Handle nullability in `determine_field_nullability`
3. Add tests for the new type

### 3. Resolver Generation (`src/resolvers.rs`)

Creates CRUD operations for GraphQL:

```rust
// Resolver function pattern
fn list_resolver(table_info: SqliteTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    Box::pin(async move {
        // Use sea-query for SQL generation
        let query = Query::select()
            .from(table_name)
            .column(Asterisk)
            .to_string(SqliteQueryBuilder);
        
        // Execute with sqlx
        let results = sqlx::query_as::<_, Row>(&query)
            .fetch_all(db)
            .await?;
        
        // Transform and return
        Ok(Some(async_graphql::Value::List(values)))
    })
}
```

**Adding new resolvers**:

1. Create resolver function in `src/resolvers.rs`
2. Register in trait implementations
3. Use sea-query for SQL generation
4. Add comprehensive error handling

## Development Workflow

### Running Examples

Each example demonstrates different patterns:

```bash
# Run the blog example
cd examples/blog
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run specific example with custom config
cd examples/ecommerce
cargo run -- serve -c custom-config.toml
```

### Testing Changes

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_introspection

# Run integration tests
cargo test --test integration_tests
```

### Code Formatting and Linting

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Check for unused dependencies
cargo machete

# Run all checks
cargo fmt && cargo clippy && cargo test
```

### Debugging with Tracing

Set debug logging to see detailed introspection:

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run -- serve

# Or specific modules
export RUST_LOG="graph_sql=debug,sqlx=info"
cargo run -- serve
```

**Key debug information**:

- Database schema introspection details
- Generated SQL queries
- Parameter binding
- Resolver execution flow
- Error stack traces

## Understanding the Codebase

### Database Schema Introspection Flow

1. **Query `sqlite_master`** - Get table definitions
2. **Use `pragma_table_info`** - Get column details
3. **Analyze foreign keys** - Detect relationships
4. **Map types** - Convert SQLite to GraphQL types

```rust
// Example introspection query
let tables_query = "
    SELECT name FROM sqlite_master 
    WHERE type='table' AND name NOT LIKE 'sqlite_%'
";

// Column information query
let columns_query = "PRAGMA table_info(?)";

// Foreign key query
let fk_query = "PRAGMA foreign_key_list(?)";
```

### GraphQL Schema Generation

1. **Create base types** - Query and Mutation
2. **Generate object types** - One per table
3. **Add relationships** - Based on foreign keys
4. **Create resolvers** - CRUD operations
5. **Build schema** - Using async-graphql-dynamic

### Sea-Query SQL Generation

graph-sql uses sea-query for type-safe SQL generation:

```rust
// Select query example
let query = Query::select()
    .from(table_name)
    .columns(columns)
    .and_where(Expr::col("id").eq(id))
    .limit(limit)
    .offset(offset)
    .to_string(SqliteQueryBuilder);

// Join query example
let query = Query::select()
    .from(table)
    .inner_join(
        related_table,
        Expr::col((table, "foreign_key"))
            .equals((related_table, "id"))
    )
    .to_string(SqliteQueryBuilder);
```

### Error Handling Patterns

```rust
// Use async_graphql::Result for GraphQL operations
use async_graphql::Result as GraphQLResult;

// Use anyhow::Result for general error handling
use anyhow::Result;

// Convert between error types
fn convert_error(e: sqlx::Error) -> async_graphql::Error {
    async_graphql::Error::new(format!("Database error: {}", e))
}
```

## Adding New Features

### Adding a New Resolver

1. **Define the resolver function**:

```rust
fn custom_resolver(table_info: SqliteTable, ctx: ResolverContext<'_>) -> FieldFuture<'_> {
    Box::pin(async move {
        // Implementation
        Ok(Some(async_graphql::Value::Null))
    })
}
```

2. **Register in the schema builder**:

```rust
// In src/traits.rs
impl ToGraphQLResolvers for SqliteTable {
    fn add_custom_resolver(&self, field: &mut Field) {
        field.argument(InputValue::new("custom_arg", TypeRef::named(TypeRef::STRING)));
        field.resolver(custom_resolver(self.clone()));
    }
}
```

3. **Add tests**:

```rust
#[tokio::test]
async fn test_custom_resolver() {
    let db = create_test_db().await;
    let schema = graph_sql::introspect(&db).await?.finish()?;
    
    let query = "{ customField(customArg: \"test\") }";
    let result = schema.execute(query).await;
    
    assert!(result.errors.is_empty());
}
```

### Adding Configuration Options

1. **Update the config struct** in `src/config.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct GraphQLConfig {
    pub enable_playground: bool,
    pub depth: u32,
    pub complexity: u32,
    pub new_option: String,  // Add new option
}
```

2. **Handle in the CLI** in `src/cli.rs`:

```rust
fn apply_config(config: &Config, schema_builder: SchemaBuilder) -> SchemaBuilder {
    // Apply new configuration
    if config.graphql.new_option == "enabled" {
        // Configure feature
    }
    schema_builder
}
```

3. **Update documentation**:

- Add to configuration guide
- Update example config files
- Add to CLI help text

### Adding Database Support

For future database support (PostgreSQL, MySQL):

1. **Abstract database operations**:

```rust
trait DatabaseIntrospector {
    async fn get_tables(&self) -> Result<Vec<String>>;
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnDef>>;
    async fn get_foreign_keys(&self, table: &str) -> Result<Vec<ForeignKeyInfo>>;
}
```

2. **Implement for each database**:

```rust
struct SqliteIntrospector<'a>(&'a SqlitePool);
struct PostgresIntrospector<'a>(&'a PgPool);
// etc.
```

3. **Update type mappings** for database-specific types

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_table_introspection() {
        let db = create_test_database().await;
        let tables = introspect_tables(&db).await.unwrap();
        assert_eq!(tables.len(), 3);
    }
    
    #[test]
    fn test_type_mapping() {
        let sqlite_type = "INTEGER";
        let graphql_type = map_sqlite_type_to_graphql(sqlite_type);
        assert_eq!(graphql_type, "Int");
    }
}
```

### Integration Tests

```rust
// tests/integration_tests.rs
#[tokio::test]
async fn test_full_schema_generation() {
    let db = setup_test_database().await;
    let schema = graph_sql::introspect(&db).await?.finish()?;
    
    // Test query execution
    let query = "{ posts { id title author { name } } }";
    let result = schema.execute(query).await;
    
    assert!(result.errors.is_empty());
    assert!(result.data.is_some());
}
```

### Example Tests

Each example includes its own tests:

```bash
# Run example-specific tests
cd examples/blog
cargo test

# Test example with different configurations
cd examples/ecommerce
RUST_LOG=debug cargo test
```

## Performance Development

### Profiling

```bash
# Install profiling tools
cargo install cargo-profiler

# Profile the application
cargo profiler callgrind --bin graph-sql -- serve

# Memory profiling
valgrind --tool=massif ./target/debug/graph-sql serve
```

### Benchmarking

```rust
// benches/schema_generation.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_introspection(c: &mut Criterion) {
    c.bench_function("introspect_large_db", |b| {
        b.iter(|| {
            // Benchmark introspection performance
            black_box(introspect_database())
        })
    });
}

criterion_group!(benches, benchmark_introspection);
criterion_main!(benches);
```

## Contributing Guidelines

### Code Style

- Use `rustfmt` for formatting
- Follow Rust naming conventions
- Add documentation for public APIs
- Include examples in doc comments

### Git Workflow

```bash
# Create feature branch
git checkout -b feature/new-resolver

# Make changes with clear commits
git commit -m "feat: add custom aggregation resolver"

# Push and create PR
git push origin feature/new-resolver
```

### Pull Request Process

1. **Fork and create branch** from `main`
2. **Add tests** for new functionality
3. **Update documentation** as needed
4. **Run full test suite** before submitting
5. **Write clear PR description** with examples

### Issue Guidelines

When reporting issues:

- Include minimum reproduction case
- Provide environment details (OS, Rust version, etc.)
- Include relevant logs with `RUST_LOG=debug`
- Describe expected vs actual behavior

This development guide provides comprehensive coverage for contributing to and
extending graph-sql.
