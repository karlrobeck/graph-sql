# Frequently Asked Questions (FAQ)

## 📋 Table of Contents

- [General Questions](#-general-questions)
- [Development & Motivation](#-development--motivation)
- [Technical Questions](#-technical-questions)
- [Comparisons](#-comparisons)
- [Future Development](#-future-development)
- [Usage & Deployment](#-usage--deployment)
- [Contributing](#-contributing)

## 🤔 General Questions

### What is graph-sql?

graph-sql is a Rust CLI tool and library that automatically introspects SQLite databases and generates complete GraphQL APIs with zero configuration. It's designed for rapid prototyping, admin panels, and turning existing databases into modern GraphQL services.

### Who is the target audience?

- **Developers** building prototypes or MVPs who need quick GraphQL APIs
- **Frontend teams** needing backend APIs for development without backend expertise
- **SQLite users** wanting to modernize existing databases with GraphQL
- **Rust developers** looking for a lightweight GraphQL solution
- **Teams** building admin panels or internal tools

### Is graph-sql production-ready?

Currently, graph-sql is in **active development**. While the core functionality is stable, breaking changes may occur. For production use, we recommend:
- Pinning to a specific commit
- Thorough testing of updates
- Using it for internal tools or non-critical applications first

## 🚀 Development & Motivation

### Why did you create graph-sql?

graph-sql was born from several key objectives and philosophy:

1. **Memory Safety Alternative**: Provide a memory-safe alternative to existing GraphQL APIs by leveraging Rust's zero-cost abstractions and memory safety guarantees
2. **Simplicity First**: Minimize installation, configuration, and deployment complexity - single binary, zero config
3. **Performance Focus**: Built for heavy load scenarios with Rust's native performance and efficient resource utilization
4. **Database-Centric Architecture**: Act as a lightweight gateway/middleman that lets the database handle business logic and authorization
5. **SQLite Maximization**: Unlock SQLite's full potential as a web API backend, often overlooked in favor of traditional databases

### What problems does graph-sql solve?

- **Memory Safety**: Eliminates entire classes of bugs (buffer overflows, memory leaks) common in traditional API servers
- **Performance at Scale**: Designed to handle heavy loads efficiently with minimal resource overhead
- **Zero Configuration**: Turn any SQLite database into a GraphQL API instantly - no setup, no config files
- **Database-First Logic**: Push business logic to the database layer where it belongs, reducing application complexity
- **Foreign Key Intelligence**: Automatically maps relationships without manual schema definition
- **Deployment Simplicity**: Single binary deployment with no runtime dependencies

### Why focus on SQLite specifically?

SQLite is often underestimated but offers unique advantages for modern applications:
- **Simplicity**: Single file, no server setup required
- **Performance**: Excellent for read-heavy workloads and can handle significant write workloads
- **Deployment**: Easy to deploy, backup, and replicate
- **Edge Computing**: Perfect for edge deployments and serverless functions
- **Extensibility**: Supports extensions (like sqlean) for advanced functionality
- **ACID Compliance**: Full ACID transactions with excellent reliability
- **Embedded Logic**: Can host business logic through stored procedures and extensions

The goal is to make SQLite a first-class citizen for web APIs, not just a development or embedded database.

## 🔧 Technical Questions

### How does the automatic schema generation work?

graph-sql uses SQLite's built-in introspection capabilities:

1. **Table Discovery**: `PRAGMA table_list` to find all tables
2. **Column Analysis**: `PRAGMA table_info` for column definitions and types
3. **Relationship Detection**: `PRAGMA foreign_key_list` for foreign keys
4. **Type Mapping**: Converts SQLite types to GraphQL scalars
5. **Resolver Generation**: Creates dynamic resolvers for CRUD operations

### What GraphQL features are supported?

Currently supported:
- ✅ **Queries**: List and view operations with pagination
- ✅ **Mutations**: Insert, update, delete operations
- ✅ **Foreign Key Relationships**: Automatic relationship field generation
- ✅ **Type Safety**: Proper nullable/non-nullable field mapping
- ✅ **GraphiQL**: Built-in interactive query interface

Not yet supported:
- ❌ **Subscriptions**: Real-time updates
- ❌ **Advanced Filtering**: WHERE clauses beyond basic ID lookup
- ❌ **Custom Resolvers**: Plugin system for business logic
- ❌ **Aggregations**: COUNT, SUM, AVG operations

### Can I customize the generated schema?

graph-sql follows a **database-first architecture** where business logic lives in the database, not the application layer. As a compiled binary, the CLI application is intentionally not extensible through plugins or custom code.

**Current approach:**
- Schema is automatically generated from database structure
- Business logic should be implemented in SQLite (stored procedures, triggers, constraints)
- Authorization handled by SQLite using JWT claims passed from the gateway

**Future extensibility through SQLite:**
- **SQLite Extensions**: Support for sqlean and other SQLite extensions
- **Database Functions**: Custom SQL functions for business logic
- **Row-Level Security**: Authorization logic implemented in SQLite
- **Triggers and Constraints**: Database-enforced business rules

**Why this approach:**
- **Performance**: Database logic executes faster than application logic
- **Consistency**: Business rules enforced at the data layer
- **Security**: Authorization happens closest to the data
- **Simplicity**: Single binary with no plugin complexity

### What about performance?

graph-sql is designed specifically for **high-performance, heavy-load scenarios**:

**Rust Performance Advantages:**
- **Memory Safety**: Zero-cost abstractions with no garbage collection overhead
- **Native Speed**: Compiled binary performance comparable to C/C++
- **Efficient Concurrency**: Tokio async runtime for handling thousands of concurrent connections
- **Minimal Resource Usage**: Low memory footprint and CPU overhead

**Database-First Performance:**
- **Reduced Network Overhead**: Business logic executes in SQLite, not over network calls
- **Query Optimization**: SQLite's query planner handles optimization
- **Connection Efficiency**: Single connection pool to database
- **Caching**: SQLite's built-in page cache and query optimization

**Heavy Load Design:**
- **Stateless Gateway**: Pure middleman with no application state
- **Horizontal Scaling**: Multiple instances can serve the same database
- **Resource Efficiency**: Designed to handle high throughput with minimal resources

**Future Performance Features:**
- Data loaders for N+1 query prevention
- Connection pooling optimizations
- Query result caching
- Performance monitoring and metrics

## ⚖️ Comparisons

### How does graph-sql compare to Hasura?

| Feature | graph-sql | Hasura |
|---------|-----------|---------|
| **Database Support** | SQLite + extensions | PostgreSQL, MySQL, SQL Server, BigQuery |
| **Setup Complexity** | Zero config, single binary | Moderate (Docker/Cloud) |
| **Target Use Case** | High-performance gateway | Production, enterprise |
| **Memory Safety** | Rust memory safety | Go (garbage collected) |
| **Business Logic** | Database-first (SQLite) | Application-first (resolvers) |
| **Authentication** | JWT → SQLite authorization | Built-in with multiple providers |
| **Performance** | Heavy load optimized | Very fast, enterprise scale |
| **Deployment** | Single binary | Docker/Kubernetes |
| **Extensibility** | SQLite extensions | Extensive plugin system |
| **Learning Curve** | Minimal | Moderate |
| **Architecture** | Stateless gateway | Full GraphQL platform |

**When to choose graph-sql over Hasura:**
- You want memory-safe, Rust-based performance
- You prefer database-first business logic
- You need simple, single-binary deployment
- You want to leverage SQLite's capabilities
- You need a lightweight, stateless gateway

**When to choose Hasura over graph-sql:**
- You need multiple database support
- You require extensive enterprise features
- You prefer application-layer business logic
- You need advanced real-time subscriptions
- You want a full-featured GraphQL platform

### How does it compare to PostgREST?

| Feature | graph-sql | PostgREST |
|---------|-----------|-----------|
| **API Style** | GraphQL | REST |
| **Database** | SQLite | PostgreSQL |
| **Query Flexibility** | High (GraphQL) | Moderate (REST) |
| **Learning Curve** | GraphQL knowledge needed | REST-familiar |
| **Introspection** | Automatic + relationships | Automatic |
| **Type Safety** | Strong (Rust + GraphQL) | Moderate |

### How does it compare to Prisma?

| Feature | graph-sql | Prisma |
|---------|-----------|---------|
| **Approach** | Database-first | Schema-first |
| **Language** | Rust | TypeScript/Node.js |
| **Database Support** | SQLite | Multiple databases |
| **Code Generation** | Runtime schema | Client + types |
| **Migration** | External | Built-in |
| **ORM Features** | No ORM | Full ORM |

### How does it compare to Supabase?

| Feature | graph-sql | Supabase |
|---------|-----------|---------|
| **Scope** | GraphQL gateway/middleman | Full backend platform |
| **Database** | SQLite + extensions | PostgreSQL |
| **Architecture** | Stateless gateway | Full backend platform |
| **Business Logic** | Database-first (SQLite) | Mixed (database + edge functions) |
| **Authentication** | JWT → database authorization | Built-in authentication service |
| **Storage** | Database-only | Built-in file storage |
| **Real-time** | Planned | Built-in subscriptions |
| **Deployment** | Single binary | Cloud + self-hosted |
| **Memory Safety** | Rust memory safety | TypeScript/Node.js |
| **Performance Focus** | Heavy load optimization | Platform completeness |
| **Complexity** | Minimal gateway | Platform learning curve |

**Philosophical Similarity to Supabase:**
Both graph-sql and Supabase share the philosophy of **pushing logic to the database layer** rather than building complex application servers. However:

- **Supabase**: Uses PostgreSQL with RLS (Row Level Security) and edge functions
- **graph-sql**: Uses SQLite with extensions and JWT-based authorization

**When to choose graph-sql over Supabase:**
- You want a lightweight, single-binary solution
- You prefer SQLite's simplicity and performance characteristics
- You need memory-safe, Rust-based performance
- You want a pure gateway approach without additional services
- You're building high-performance, focused applications

**When to choose Supabase over graph-sql:**
- You need a complete backend platform
- You require PostgreSQL's advanced features
- You want built-in authentication, storage, and edge functions
- You prefer managed cloud services
- You need extensive real-time features

## 🛣️ Future Development

### What's on the roadmap?

**Short-term (Next 3-6 months):**
- JWT authentication and authorization integration
- SQLite extension support (starting with sqlean)
- Advanced filtering (WHERE clauses)
- Performance optimizations for heavy load scenarios

**Medium-term (6-12 months):**
- Row-level security implementation in SQLite
- Real-time subscriptions
- Database function support for business logic
- Connection pooling and caching optimizations

**Long-term (12+ months):**
- Multi-database support (PostgreSQL, MySQL) with same philosophy
- Advanced SQLite extensions ecosystem
- Performance monitoring and metrics
- Horizontal scaling tools and best practices

### Will you support other databases?

Yes! The **database-first philosophy** will extend to other databases while maintaining the same core principles:

**Planned Database Support:**
1. **PostgreSQL** (high priority) - with Row Level Security and stored procedures
2. **MySQL** (medium priority) - leveraging stored procedures and user-defined functions
3. **SQL Server** (lower priority) - with T-SQL business logic support

**Consistent Philosophy Across Databases:**
- **Gateway Architecture**: graph-sql remains a stateless middleman
- **Database-First Logic**: Business rules implemented in the database layer
- **JWT Authorization**: Authentication handled by passing JWT claims to database
- **Memory Safety**: Rust performance and safety benefits for all database backends
- **Zero Configuration**: Same simple setup regardless of database

**Database-Specific Features:**
- **PostgreSQL**: Row Level Security, stored procedures, custom types
- **MySQL**: Stored procedures, user-defined functions, triggers
- **SQLite**: Extensions (sqlean), custom functions, triggers
- **SQL Server**: T-SQL procedures, user-defined functions, CLR integration

The goal is to unlock the full potential of each database's capabilities while maintaining graph-sql's core principles of simplicity, performance, and database-first architecture.

### Will there be a hosted version?

A hosted/cloud version is under consideration, which would offer:
- Managed deployments
- Automatic scaling
- Built-in authentication
- Database hosting
- Real-time collaboration features

### How can I influence the roadmap?

- **GitHub Issues**: Request features or report needs
- **Discussions**: Join conversations about direction
- **Contributing**: Submit PRs for features you need
- **Community**: Share your use cases and requirements

## 🚀 Usage & Deployment

### Can I use graph-sql in production?

With caveats:
- **Internal tools**: Great for admin panels and internal APIs
- **Prototypes**: Perfect for MVPs and proof of concepts
- **Low-risk applications**: Suitable for non-critical services
- **Development**: Excellent for development and testing environments

**Not recommended for:**
- High-traffic public APIs (yet)
- Mission-critical applications
- Applications requiring real-time features
- Complex authentication/authorization needs

### How do I deploy graph-sql?

**Binary Deployment:**
```bash
# Install and run
cargo install --git https://github.com/karlrobeck/graph-sql.git
graph-sql serve -d "sqlite://production.db" -p 8080
```

**Docker Deployment:**
```dockerfile
FROM rust:alpine as builder
RUN cargo install --git https://github.com/karlrobeck/graph-sql.git

FROM alpine:latest
COPY --from=builder /usr/local/cargo/bin/graph-sql /usr/local/bin/
COPY database.db /app/
WORKDIR /app
CMD ["graph-sql", "serve", "-d", "sqlite://database.db"]
```

**Systemd Service:**
```ini
[Unit]
Description=graph-sql GraphQL API
After=network.target

[Service]
Type=simple
User=graph-sql
WorkingDirectory=/opt/graph-sql
ExecStart=/usr/local/bin/graph-sql serve -d sqlite:///opt/graph-sql/data.db
Restart=always

[Install]
WantedBy=multi-user.target
```

### What about scaling?

Current scaling options:
- **Vertical**: Increase server resources
- **Load Balancing**: Multiple instances behind a load balancer
- **Read Replicas**: SQLite supports read-only replicas
- **Caching**: Add Redis/Memcached in front

Future scaling features:
- Connection pooling optimizations
- Query result caching
- Horizontal sharding support

### How do I backup and migrate data?

**SQLite Advantages:**
- Simple file-based backups
- Easy replication with `rsync` or similar
- Built-in `.backup` command
- Version control friendly (for small databases)

**Migration Strategies:**
- Use SQLx migrations for schema changes
- Export/import via SQL dumps
- Custom migration scripts

## 🤝 Contributing

### How can I contribute?

**Code Contributions:**
- Fix bugs and implement features
- Improve documentation
- Add tests and examples
- Optimize performance

**Non-Code Contributions:**
- Report bugs and issues
- Request features
- Improve documentation
- Share use cases and feedback
- Help other users in discussions

### What skills are needed to contribute?

**Core Development:**
- Rust programming
- SQLite knowledge
- GraphQL understanding
- Async programming with tokio

**Specific Areas:**
- Database introspection (SQL expertise)
- GraphQL schema design
- Web framework knowledge (Axum)
- CLI development (clap)

### How do I get started contributing?

1. **Read the code**: Start with `src/main.rs` and explore
2. **Run examples**: Try the blog, ecommerce, and other examples
3. **Find an issue**: Look for "good first issue" labels
4. **Ask questions**: Use GitHub discussions for help
5. **Start small**: Documentation fixes, tests, or small features

### What's the development philosophy?

- **Memory Safety First**: Leverage Rust's memory safety for reliable, crash-free APIs
- **Database-First Architecture**: Business logic belongs in the database, not the application
- **Gateway Simplicity**: Act as a lightweight middleman, not a complex application server
- **Performance for Scale**: Designed to handle heavy loads efficiently
- **Zero Configuration**: Minimal setup and deployment complexity
- **SQLite Maximization**: Unlock SQLite's full potential as a web API backend
- **Stateless Design**: No application state, enabling horizontal scaling
- **Extension over Application**: Extend capabilities through database extensions, not application plugins

---

## 📞 Still Have Questions?

- **GitHub Issues**: [Report bugs or request features](https://github.com/karlrobeck/graph-sql/issues)
- **GitHub Discussions**: [Ask questions and share ideas](https://github.com/karlrobeck/graph-sql/discussions)
- **Documentation**: [Check the main README](./README.md)
- **Examples**: [Explore the examples directory](./examples/)

---

**graph-sql** - Turning your SQLite database into a full-featured GraphQL API, effortlessly.
