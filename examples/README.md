# Graph-SQL Examples

This directory contains comprehensive examples demonstrating different aspects and use cases of the graph-sql library. Each example is a complete, runnable application with its own database schema, sample data, and documentation.

## Available Examples

### 1. 📝 Blog System (`/blog`)
**Port**: 8080  
**Complexity**: Intermediate  
**Focus**: Content management and social features

A modern blogging platform demonstrating:
- **Content Hierarchy**: Users → Posts → Comments
- **Categorization**: Categories and Tags with many-to-many relationships
- **Social Features**: User profiles, comment threading
- **SEO Fields**: Slugs, meta descriptions, published status

**Key Relationships**:
- Many-to-many: Posts ↔ Tags
- One-to-many: Users → Posts, Posts → Comments
- Self-referencing: Categories with parent-child hierarchy

### 2. 🛒 E-commerce System (`/ecommerce`)
**Port**: 8081  
**Complexity**: Advanced  
**Focus**: Business transactions and inventory

A complete e-commerce platform featuring:
- **Product Catalog**: Products with variants, categories, images
- **Order Management**: Shopping carts, orders, line items
- **Customer System**: User accounts, addresses, order history
- **Reviews & Ratings**: Product reviews with helpful votes

**Key Relationships**:
- Complex hierarchies: Categories with nested subcategories
- Transaction flows: Cart → Order → Line Items
- Many-to-many: Products ↔ Categories

### 3. ✅ Task Manager (`/tasks`)
**Port**: 8082  
**Complexity**: Simple  
**Focus**: Basic CRUD and project management

A clean task management system showcasing:
- **Project Organization**: Projects containing tasks
- **Task Dependencies**: Self-referencing task relationships
- **Status Tracking**: Workflow states and priorities
- **Time Management**: Due dates, time tracking

**Key Relationships**:
- Self-referencing: Tasks can depend on other tasks
- Simple hierarchies: Projects → Tasks
- Status workflows with enum-like constraints

### 4. 📚 Library Management (`/library`)
**Port**: 8083  
**Complexity**: Comprehensive  
**Focus**: Data types and business logic

A full library management system demonstrating:
- **Rich Data Types**: All SQLite types including BLOB, NUMERIC
- **Complex Business Logic**: Loans, reservations, fines
- **Membership System**: Different member types with varying privileges
- **Review System**: Book ratings and member feedback

**Key Relationships**:
- Multiple foreign keys: Books → Author, Publisher, Genre
- Hierarchical data: Genres with parent-child relationships
- Complex constraints: Availability tracking, fine calculations

## Quick Start Guide

### Prerequisites
- Rust 1.70+
- SQLite 3.x

### Running Any Example

1. **Navigate to the example**:
   ```bash
   cd examples/[example-name]
   ```

2. **Install dependencies** (automatically handled by Cargo):
   ```bash
   cargo build
   ```

3. **Run the server**:
   ```bash
   cargo run
   ```

4. **Apply database migrations**:
   ```bash
   sqlx migrate run --database-url sqlite:[example-name].db
   ```

5. **Open GraphiQL interface**:
   - Blog: http://localhost:8080/graphiql
   - E-commerce: http://localhost:8081/graphiql
   - Tasks: http://localhost:8082/graphiql
   - Library: http://localhost:8083/graphiql

### Running Multiple Examples
Each example uses a different port, so you can run them simultaneously for comparison.

## Learning Path

### 🟢 **Beginner**: Start with Tasks
- Simple schema with clear relationships
- Basic CRUD operations
- Introduction to GraphQL queries and mutations
- Self-referencing relationships (task dependencies)

### 🟡 **Intermediate**: Move to Blog
- More complex content management
- Many-to-many relationships (posts and tags)
- User-generated content patterns
- SEO and publishing workflows

### 🟠 **Advanced**: Try E-commerce
- Business transaction patterns
- Complex product catalogs
- Order management workflows
- Customer relationship management

### 🔴 **Expert**: Master Library
- All SQLite data types in action
- Complex business rules and constraints
- Multi-table relationships
- Real-world domain modeling

## Common Patterns Demonstrated

### Foreign Key Mapping
All examples show how graph-sql automatically detects foreign key relationships:
```sql
-- Database schema
CREATE TABLE post (
    id INTEGER PRIMARY KEY,
    title TEXT,
    author_id INTEGER,
    FOREIGN KEY (author_id) REFERENCES user(id)
);
```

```graphql
# Automatic GraphQL schema
type Post {
    id: ID!
    title: String
    author: User    # Automatically generated relationship
}

type User {
    id: ID!
    posts: [Post!]! # Reverse relationship also generated
}
```

### Data Type Mappings
See how different SQLite types map to GraphQL:
- `INTEGER` → `Int`
- `TEXT` → `String`
- `REAL` → `Float`
- `BLOB` → `String` (base64 encoded)
- `BOOLEAN` → `Boolean`
- `NUMERIC` → `String` (for precision)

### Query Patterns
Each example includes comprehensive query examples:
- Basic CRUD operations
- Relationship traversal
- Filtering and sorting
- Aggregation patterns
- Complex business queries

## Development Tips

### Database Inspection
Use SQLite tools to explore the database:
```bash
sqlite3 examples/blog/blog.db
.tables
.schema post
SELECT * FROM user LIMIT 5;
```

### Schema Evolution
Each example includes both up and down migrations:
```bash
# Apply migrations
sqlx migrate run

# Rollback migrations  
sqlx migrate revert
```

### GraphQL Exploration
The GraphiQL interface includes:
- Schema explorer (click "Docs" in the top right)
- Query autocompletion
- Real-time validation
- Query history

### Custom Queries
Try building custom queries across relationships:
```graphql
# Find all posts by authors from a specific country
query PostsByCountry {
  users(where: "country = 'United States'") {
    name
    posts {
      title
      publishedAt
      tags {
        name
      }
    }
  }
}
```

## Architecture Insights

Each example demonstrates key graph-sql features:

- **Automatic Schema Generation**: SQLite introspection → GraphQL schema
- **Relationship Detection**: Foreign key columns → GraphQL relationships  
- **Type Mapping**: SQLite types → GraphQL scalars
- **CRUD Resolver Generation**: Automatic queries and mutations
- **Dynamic SQL**: Sea-query for flexible database operations

## Contributing Examples

To add a new example:

1. Create a new directory: `examples/[your-example]`
2. Add `Cargo.toml` with graph-sql dependency
3. Create `src/main.rs` with server setup
4. Add `migrations/` directory with schema files
5. Include comprehensive `README.md` with queries
6. Use a unique port number
7. Update this main README

## Getting Help

- Check individual example READMEs for specific guidance
- Explore the main graph-sql documentation
- Use GraphiQL's built-in documentation browser
- Examine the generated schema in the GraphiQL interface

Each example is self-contained and thoroughly documented to help you understand both graph-sql capabilities and real-world GraphQL API patterns.
