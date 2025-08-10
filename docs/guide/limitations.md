# Current Limitations

While graph-sql provides powerful functionality for generating GraphQL APIs from
SQLite databases, there are some current limitations to be aware of. This page
documents these limitations and provides context for each.

## Database Support

### SQLite Only

**Current State**: graph-sql currently supports only SQLite databases.

**Impact**:

- Cannot connect to PostgreSQL, MySQL, SQL Server, or other databases
- Limited to SQLite's feature set and performance characteristics
- Cannot leverage database-specific features from other systems

**Workarounds**:

- Use SQLite for appropriate use cases (edge computing, embedded applications,
  small to medium applications)
- Consider SQLite's surprisingly high performance capabilities
- Leverage SQLite extensions for additional functionality

**Future Plans**: PostgreSQL and MySQL support are planned for future releases
with the same database-first philosophy.

### SQLite Dialect Support

**Current State**: Standard SQLite SQL dialect only.

**Limitations**:

- No advanced SQLite extensions (yet)
- Limited to built-in SQLite functions
- No custom collations or user-defined functions

**Future Plans**: Support for SQLite extensions like sqlean is planned.

## GraphQL Features

### No Subscriptions

**Current State**: GraphQL subscriptions are not implemented.

**Impact**:

- No real-time updates or live data
- Cannot push data changes to clients automatically
- No WebSocket-based real-time communication

**Workarounds**:

- Use polling with queries for near-real-time updates
- Implement client-side refresh mechanisms
- Consider Server-Sent Events (SSE) for simple real-time needs

**Future Plans**: Real-time subscriptions are planned for a future release.

### Limited Filtering

**Current State**: Queries support basic ID lookups and pagination only.

**Limitations**:

- No WHERE clause filtering beyond primary keys
- No complex query conditions (greater than, less than, etc.)
- No full-text search capabilities
- No sorting options beyond natural order

**Example of what's NOT supported**:

```graphql
# This is NOT currently supported
query {
  posts(where: { title: { contains: "rust" } }) {
    id
    title
  }
  
  users(where: { age: { gt: 18 } }, orderBy: { name: ASC }) {
    name
    age
  }
}
```

**Workarounds**:

- Use database views for pre-filtered data
- Implement filtering logic in SQLite triggers or stored procedures
- Create multiple specialized tables for different query patterns

**Future Plans**: Advanced filtering and sorting are high-priority features for
upcoming releases.

### No Aggregations

**Current State**: No support for aggregation operations.

**Limitations**:

- No COUNT, SUM, AVG, MIN, MAX operations
- No GROUP BY functionality
- No statistical queries

**Example of what's NOT supported**:

```graphql
# This is NOT currently supported
query {
  postStats {
    totalPosts
    averageLength
    postsByAuthor {
      authorName
      postCount
    }
  }
}
```

**Workarounds**:

- Create database views with aggregated data
- Use SQLite triggers to maintain summary tables
- Compute aggregations in the database layer

**Future Plans**: Aggregation support is planned for future releases.

## Authentication and Authorization

### No Built-in Authentication

**Current State**: No authentication or authorization mechanisms.

**Impact**:

- All GraphQL operations are publicly accessible
- No user context or permissions
- No row-level security

**Workarounds**:

- Use reverse proxy (nginx, Traefik) for basic authentication
- Implement API gateway with authentication upstream
- Use database-level constraints for data protection

**Future Plans**: JWT authentication with database-side authorization is
planned.

### No Authorization Framework

**Current State**: No field-level or operation-level permissions.

**Limitations**:

- Cannot restrict access to specific fields
- Cannot implement role-based access control (RBAC)
- No data ownership or multi-tenant support

**Workarounds**:

- Use database views to control data access
- Implement row-level security in SQLite (limited)
- Design schema with access control in mind

## Performance Limitations

### No Query Optimization

**Current State**: Limited query optimization beyond basic SQLite optimization.

**Limitations**:

- No query plan caching
- No intelligent batching of related queries
- No data loader pattern for N+1 query prevention

**Impact**:

- Potential N+1 query problems with nested relationships
- Suboptimal performance for complex queries
- No automatic query optimization

**Workarounds**:

- Design database schema to minimize joins
- Use SQLite indexes effectively
- Limit query depth and complexity

**Future Plans**: Data loaders and query optimization are planned improvements.

### No Caching Layer

**Current State**: No built-in caching mechanisms.

**Limitations**:

- Every query hits the database
- No response caching
- No intelligent cache invalidation

**Workarounds**:

- Use reverse proxy caching (nginx, Varnish)
- Implement application-level caching
- Use SQLite's built-in caching mechanisms

**Future Plans**: Query result caching is planned for future releases.

## Schema Limitations

### No Custom Scalars

**Current State**: Limited to basic GraphQL scalar types.

**Limitations**:

- No Date/DateTime types (returned as strings)
- No JSON scalar type
- No custom business-specific scalars

**Example limitations**:

```graphql
# Current: dates are strings
type Post {
  createdAt: String  # Would prefer DateTime scalar
}

# Would prefer: custom scalars
type Post {
  createdAt: DateTime
  metadata: JSON
  price: Money
}
```

**Workarounds**:

- Use string types for dates (ISO 8601 format)
- Store JSON as TEXT in SQLite
- Handle type conversion in client applications

### No Custom Resolvers

**Current State**: Only auto-generated CRUD resolvers are supported.

**Limitations**:

- Cannot add custom business logic resolvers
- No computed fields beyond database columns
- No custom mutations beyond basic CRUD

**Example of what's NOT supported**:

```graphql
# This is NOT currently supported
type User {
  id: ID!
  name: String!
  fullName: String!  # Computed field
  postCount: Int!    # Aggregated field
}

type Mutation {
  publishPost(id: ID!): Post  # Custom business logic
  sendEmail(to: String!, subject: String!): Boolean
}
```

**Workarounds**:

- Use database views for computed fields
- Implement business logic in SQLite triggers
- Use separate services for complex operations

**Future Plans**: Plugin system for custom resolvers is planned.

## Operational Limitations

### Limited Monitoring

**Current State**: Basic logging only, no metrics or monitoring.

**Limitations**:

- No performance metrics collection
- No query performance insights
- No alerting capabilities
- Limited observability

**Workarounds**:

- Use external monitoring tools
- Parse logs for insights
- Monitor at the infrastructure level

**Future Plans**: Built-in metrics and monitoring are planned.

### No Migration Management

**Current State**: Basic migration support only.

**Limitations**:

- No migration versioning
- No rollback capabilities
- No migration validation
- No schema evolution tracking

**Workarounds**:

- Use external migration tools
- Manage migrations manually
- Version control your schema changes

## Development Status Considerations

### Breaking Changes

**Current State**: Active development with potential breaking changes.

**Impact**:

- API may change between versions
- Configuration format may evolve
- Database introspection behavior may change

**Recommendations**:

- Pin to specific commits for production
- Test thoroughly when upgrading
- Follow release notes carefully

### Limited Documentation

**Current State**: Documentation is growing but not complete.

**Limitations**:

- Some advanced features undocumented
- Limited troubleshooting guides
- Few real-world usage examples

**Resources**:

- Check examples directory for patterns
- Review source code for advanced usage
- Join community discussions for help

## Workaround Strategies

### Database-First Approach

Many limitations can be addressed by embracing the database-first philosophy:

```sql
-- Use views for filtering
CREATE VIEW recent_posts AS
SELECT * FROM posts 
WHERE created_at > date('now', '-30 days');

-- Use triggers for business logic
CREATE TRIGGER update_post_count
AFTER INSERT ON posts
FOR EACH ROW
BEGIN
    UPDATE users 
    SET post_count = post_count + 1 
    WHERE id = NEW.author_id;
END;

-- Use computed columns
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    full_name TEXT GENERATED ALWAYS AS 
        (first_name || ' ' || last_name) STORED
);
```

### External Tool Integration

Combine graph-sql with other tools:

```yaml
# docker-compose.yml - Full stack
services:
  graph-sql:
    build: .
    environment:
      - DATABASE_URL=sqlite:///data/app.db

  nginx:
    image: nginx
    # Handle authentication, caching, rate limiting

  redis:
    image: redis
    # External caching layer

  prometheus:
    image: prom/prometheus
    # Monitoring and metrics
```

## Migration Strategies

### When Limitations Become Blockers

If current limitations prevent your use case:

1. **Evaluate alternatives** - Consider if graph-sql is the right fit
2. **Implement workarounds** - Use database-first approaches
3. **Contribute to development** - Help implement missing features
4. **Use hybrid approach** - Combine graph-sql with other tools
5. **Wait for features** - Check roadmap for planned improvements

### Future-Proofing

To minimize impact of current limitations:

- Design your schema with filtering in mind
- Use database views extensively
- Keep business logic in the database
- Plan for authentication integration
- Monitor performance carefully

Most of these limitations are temporary and will be addressed in future
releases. The core architecture of graph-sql is designed to support these
features as they are implemented.
