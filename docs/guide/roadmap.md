# Roadmap

This roadmap outlines the planned development for graph-sql, organized by
timeframe and priority. The project follows a database-first philosophy,
focusing on features that enhance this approach.

## Short-term (Next 3-6 months)

These features are high priority and actively being developed or planned for
immediate implementation.

### JWT Authentication and Authorization

**Priority**: High **Status**: Planning

**Features**:

- JWT token validation and parsing
- User context extraction from tokens
- Database-side authorization checks
- Row-level security integration

**Implementation approach**:

```rust
// JWT claims passed to database for authorization
let user_context = extract_jwt_claims(request)?;

// Database-side authorization in queries
let query = Query::select()
    .from("posts")
    .and_where(Expr::col("author_id").eq(user_context.user_id))
    .to_string(SqliteQueryBuilder);
```

**Benefits**:

- Secure API access control
- User-specific data filtering
- Role-based permissions
- Stateless authentication

### SQLite Extension Support

**Priority**: High **Status**: Research

**Target Extensions**:

- **sqlean** - Math, string, and utility functions
- **sqlite-crypto** - Cryptographic functions
- **sqlite-uuid** - UUID generation
- **sqlite-json** - Advanced JSON operations

**Example usage**:

```sql
-- With sqlean crypto extension
CREATE TRIGGER hash_password
BEFORE INSERT ON users
FOR EACH ROW
WHEN NEW.password_hash IS NULL
BEGIN
    UPDATE users SET password_hash = crypto_hash(NEW.password_plain, 'sha256')
    WHERE rowid = NEW.rowid;
END;

-- With UUID extension
CREATE TABLE users (
    id TEXT PRIMARY KEY DEFAULT (uuid4()),
    name TEXT NOT NULL
);
```

**Benefits**:

- Enhanced database-side business logic
- Advanced data processing capabilities
- Better security features
- Reduced application complexity

### Advanced Filtering (WHERE clauses)

**Priority**: High **Status**: Design

**Planned Filtering Operations**:

```graphql
# String operations
query {
  posts(where: { 
    title: { contains: "rust" }
    content: { startsWith: "Welcome" }
  }) {
    id
    title
  }
}

# Numeric comparisons
query {
  users(where: { 
    age: { gt: 18, lt: 65 }
    score: { gte: 100 }
  }) {
    name
    age
  }
}

# Date filtering
query {
  posts(where: { 
    createdAt: { after: "2024-01-01" }
  }) {
    title
    createdAt
  }
}

# Logical operators
query {
  posts(where: { 
    OR: [
      { title: { contains: "rust" } }
      { tags: { contains: "programming" } }
    ]
  }) {
    title
  }
}
```

**Implementation strategy**:

- Generate WHERE clauses from GraphQL input
- Use sea-query for safe SQL generation
- Support for complex nested conditions
- Proper parameter binding for security

### Performance Optimizations

**Priority**: Medium-High **Status**: Ongoing

**Focus Areas**:

1. **N+1 Query Prevention**:
   ```rust
   // Implement data loader pattern
   struct DataLoader {
       batch_fn: BatchFn,
       cache: Cache,
   }

   // Batch related queries
   async fn load_authors(post_ids: Vec<i64>) -> Vec<User> {
       // Single query for multiple authors
   }
   ```

2. **Connection Pooling Optimization**:
   - Smarter connection reuse
   - Query-specific pool sizing
   - Connection health monitoring

3. **Query Result Caching**:
   - In-memory result caching
   - Cache invalidation strategies
   - Configurable TTL per query type

4. **Schema Caching**:
   - Cache introspection results
   - Hot reload on schema changes
   - Persistent schema cache

**Performance Goals**:

- 50% reduction in query latency for complex relationships
- 10x improvement in concurrent request handling
- Minimal memory overhead increase

### Data Loaders

**Priority**: Medium **Status**: Design

**Implementation**:

```rust
// Automatic batching for relationships
async fn resolve_posts_authors(posts: Vec<Post>) -> Vec<User> {
    let author_ids: Vec<i64> = posts.iter().map(|p| p.author_id).collect();
    
    // Single batch query instead of N individual queries
    let authors = Query::select()
        .from("users")
        .and_where(Expr::col("id").is_in(author_ids))
        .to_string(SqliteQueryBuilder);
    
    // Return authors in correct order
    batch_load_authors(author_ids).await
}
```

**Benefits**:

- Eliminate N+1 query problems
- Automatic query batching
- Improved performance for nested queries
- Transparent to end users

## Medium-term (6-12 months)

Features that build on the foundation and expand capabilities significantly.

### Row-Level Security in SQLite

**Priority**: High **Status**: Research

**Implementation Strategy**:

```sql
-- Policy-based access control
CREATE TABLE posts (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    is_public BOOLEAN DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Views with built-in security
CREATE VIEW user_posts AS
SELECT * FROM posts 
WHERE author_id = current_user_id() OR is_public = 1;

-- Custom function for user context
CREATE FUNCTION current_user_id() RETURNS INTEGER AS $$
    -- Implementation depends on JWT integration
$$;
```

**Features**:

- User context in database functions
- Policy-based data access
- Automatic security enforcement
- Multi-tenant data isolation

### Real-time Subscriptions

**Priority**: Medium-High **Status**: Planning

**GraphQL Subscriptions**:

```graphql
# Real-time updates
subscription {
  postUpdated(authorId: 123) {
    id
    title
    content
    updatedAt
  }
}

# New data notifications
subscription {
  newComment(postId: 456) {
    id
    content
    author {
      name
    }
  }
}
```

**Implementation Approach**:

- SQLite WAL mode for change detection
- WebSocket connections for real-time communication
- Efficient change notification system
- Subscription filtering and authorization

### Database Function Support

**Priority**: Medium **Status**: Research

**Custom Business Logic**:

```sql
-- SQLite user-defined functions
CREATE FUNCTION calculate_user_score(user_id INTEGER) 
RETURNS INTEGER AS $$
    SELECT 
        (post_count * 10) + 
        (comment_count * 2) + 
        (like_count * 1)
    FROM user_stats 
    WHERE id = user_id;
$$;

-- Stored procedures (via extensions)
CREATE PROCEDURE publish_post(post_id INTEGER, user_id INTEGER)
AS $$
BEGIN
    UPDATE posts 
    SET is_published = 1, published_at = CURRENT_TIMESTAMP
    WHERE id = post_id AND author_id = user_id;
    
    INSERT INTO activity_log (user_id, action, resource_id)
    VALUES (user_id, 'publish_post', post_id);
END;
$$;
```

**GraphQL Integration**:

```graphql
type User {
  id: ID!
  name: String!
  score: Int!  # Computed via database function
}

type Mutation {
  publishPost(id: ID!): Post  # Calls database procedure
}
```

### Connection Pooling and Caching

**Priority**: Medium **Status**: Planning

**Advanced Connection Management**:

```rust
// Smart connection pooling
struct AdaptivePool {
    read_pool: SqlitePool,
    write_pool: SqlitePool,
    cache_layer: RedisPool,
}

// Query-specific optimization
async fn execute_query(query: Query, optimization: QueryHint) -> Result<Value> {
    match optimization {
        QueryHint::ReadOnly => use_read_replica(query).await,
        QueryHint::Cacheable(ttl) => check_cache_or_execute(query, ttl).await,
        QueryHint::WriteHeavy => use_write_optimized_pool(query).await,
    }
}
```

**Caching Strategies**:

- Query result caching with smart invalidation
- Schema introspection caching
- Connection state caching
- Configurable cache backends (Redis, in-memory)

### PostgreSQL Support

**Priority**: Medium **Status**: Design

**Database-First PostgreSQL**:

```sql
-- PostgreSQL row-level security
CREATE POLICY user_posts_policy ON posts
FOR ALL TO authenticated_users
USING (author_id = current_user_id());

-- PostgreSQL stored procedures
CREATE OR REPLACE FUNCTION calculate_metrics(user_id INTEGER)
RETURNS TABLE(post_count INTEGER, avg_score NUMERIC) AS $$
BEGIN
    RETURN QUERY
    SELECT COUNT(*)::INTEGER, AVG(score)
    FROM posts
    WHERE author_id = user_id;
END;
$$ LANGUAGE plpgsql;
```

**Features**:

- Full PostgreSQL type system support
- Row-level security integration
- Stored procedure support
- Advanced indexing strategies
- JSON/JSONB support

## Long-term (12+ months)

Advanced features that significantly expand the scope and capabilities.

### Multi-Database Support

**Priority**: Medium **Status**: Concept

**Supported Databases**:

- **PostgreSQL** - Full production support with RLS
- **MySQL** - Stored procedures and triggers
- **SQL Server** - T-SQL procedures and CLR integration
- **SQLite** - Enhanced extension ecosystem

**Unified Interface**:

```rust
// Database-agnostic introspection
trait DatabaseIntrospector {
    async fn introspect_schema(&self) -> Result<Schema>;
    async fn get_business_logic(&self) -> Result<Vec<Function>>;
    async fn supports_feature(&self, feature: DatabaseFeature) -> bool;
}

// Database-specific optimizations
impl DatabaseIntrospector for PostgresIntrospector {
    async fn introspect_schema(&self) -> Result<Schema> {
        // PostgreSQL-specific introspection
        // Support for custom types, enums, arrays, etc.
    }
}
```

### Advanced SQLite Extensions Ecosystem

**Priority**: Medium-Low **Status**: Concept

**Custom Extensions**:

- **graph-sql-auth** - Authentication functions
- **graph-sql-crypto** - Advanced cryptography
- **graph-sql-ml** - Machine learning functions
- **graph-sql-geo** - Geospatial operations

**Example Usage**:

```sql
-- ML-powered recommendations
CREATE FUNCTION recommend_posts(user_id INTEGER, limit INTEGER)
RETURNS TABLE(post_id INTEGER, score REAL) AS $$
    -- Machine learning model inference
$$;

-- Geospatial queries
CREATE FUNCTION nearby_users(lat REAL, lng REAL, radius_km REAL)
RETURNS TABLE(user_id INTEGER, distance_km REAL) AS $$
    -- Geospatial distance calculations
$$;
```

### Performance Monitoring and Metrics

**Priority**: Medium **Status**: Concept

**Comprehensive Observability**:

```rust
// Built-in metrics collection
struct Metrics {
    request_duration: Histogram,
    query_count: Counter,
    active_connections: Gauge,
    cache_hit_rate: Gauge,
}

// Performance insights
struct QueryStats {
    query_hash: String,
    execution_time: Duration,
    rows_returned: usize,
    cache_status: CacheStatus,
}
```

**Monitoring Features**:

- Request/response timing
- Query performance analytics
- Resource usage tracking
- Error rate monitoring
- Cache efficiency metrics
- Custom dashboards (Grafana integration)

### Horizontal Scaling Tools

**Priority**: Low **Status**: Concept

**Scaling Infrastructure**:

```yaml
# Kubernetes deployment with auto-scaling
apiVersion: apps/v1
kind: Deployment
metadata:
  name: graph-sql
spec:
  replicas: 5
  selector:
    matchLabels:
      app: graph-sql
  template:
    spec:
      containers:
        - name: graph-sql
          image: graph-sql:latest
          env:
            - name: DATABASE_URL
              value: "sqlite:///shared/app.db"
          resources:
            requests:
              memory: "64Mi"
              cpu: "50m"
            limits:
              memory: "128Mi"
              cpu: "100m"
```

**Scaling Features**:

- Auto-scaling based on metrics
- Load balancing optimization
- Distributed caching
- Read replica management
- Health check integration

### Custom Scalar Types

**Priority**: Low **Status**: Concept

**Rich Type System**:

```graphql
scalar DateTime
scalar JSON
scalar Money
scalar UUID

type Post {
  id: UUID!
  title: String!
  content: String!
  metadata: JSON
  price: Money
  createdAt: DateTime!
}
```

**Implementation**:

```rust
// Custom scalar implementations
impl ScalarType for DateTime {
    fn parse(value: Value) -> Result<Self> {
        // Parse ISO 8601 strings
    }
    
    fn to_value(&self) -> Value {
        // Serialize to ISO 8601
    }
}
```

## Plugin System Architecture

**Priority**: Medium-Low **Status**: Concept

**Extensible Plugin Framework**:

```rust
// Plugin trait for extending functionality
trait GraphQLPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn modify_schema(&self, builder: SchemaBuilder) -> Result<SchemaBuilder>;
    fn add_middleware(&self) -> Vec<Box<dyn Middleware>>;
    fn add_custom_scalars(&self) -> Vec<Box<dyn CustomScalar>>;
}

// Plugin registration
let schema = graph_sql::introspect(&db)
    .await?
    .add_plugin(AuthPlugin::new())
    .add_plugin(CachePlugin::new())
    .add_plugin(MetricsPlugin::new())
    .finish()?;
```

**Plugin Examples**:

- Authentication plugins (JWT, OAuth, API keys)
- Caching plugins (Redis, Memcached, in-memory)
- Monitoring plugins (Prometheus, DataDog, custom)
- Business logic plugins (custom resolvers, validators)

## Development Priorities

### Current Focus Areas

1. **Core Stability** - Ensure reliable SQLite introspection and schema
   generation
2. **Performance** - Optimize for high-load scenarios
3. **Security** - JWT authentication and authorization
4. **Developer Experience** - Better error messages, documentation, tooling

### Community Contributions

Areas where community contributions are especially welcome:

- **Documentation** - Examples, tutorials, best practices
- **Testing** - Edge cases, performance testing, integration tests
- **Extensions** - SQLite extension integration
- **Examples** - Real-world usage patterns
- **Tooling** - CLI improvements, development tools

### Feedback Integration

The roadmap is influenced by:

- **User feedback** - Feature requests and pain points
- **Performance benchmarks** - Real-world usage data
- **Security requirements** - Enterprise and production needs
- **Database evolution** - New SQLite features and extensions

## Getting Involved

### Contributing to Development

1. **Join discussions** on GitHub issues and discussions
2. **Report bugs** with detailed reproduction cases
3. **Submit feature requests** with clear use cases
4. **Contribute code** following the contribution guidelines
5. **Write documentation** and examples
6. **Test beta features** and provide feedback

### Staying Updated

- **GitHub Releases** - Major feature announcements
- **GitHub Discussions** - Design discussions and feedback
- **Documentation** - Updated guides and examples
- **Examples Repository** - New usage patterns and best practices

This roadmap is a living document that evolves based on user needs, technical
requirements, and community feedback. The database-first philosophy remains
central to all planned features.
