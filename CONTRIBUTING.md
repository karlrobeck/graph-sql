# Contributing to graph-sql

Thank you for your interest in contributing to graph-sql! We welcome contributions from the community and are excited to see what you'll build.

## 🤝 Ways to Contribute

### **Code Contributions**
- Fix bugs and implement features
- Improve performance optimizations
- Add tests and examples
- Enhance documentation
- Optimize database introspection

### **Non-Code Contributions**
- Report bugs and issues
- Request features and improvements
- Improve documentation
- Share use cases and feedback
- Help other users in discussions
- Write tutorials and guides

## 🛠️ Skills Needed

### **Core Development**
- **Rust Programming**: Intermediate to advanced Rust knowledge
- **SQLite**: Understanding of SQLite internals and SQL
- **GraphQL**: Knowledge of GraphQL schema design and resolvers
- **Async Programming**: Experience with tokio and async/await

### **Specific Areas**
- **Database Introspection**: SQL expertise for schema analysis
- **GraphQL Schema Design**: Type system and resolver patterns
- **Web Framework Knowledge**: Axum, async-graphql experience
- **CLI Development**: clap, TOML configuration
- **Performance Optimization**: Profiling and optimization techniques

## 🚀 Getting Started

### **1. Set Up Your Development Environment**

```bash
# Clone the repository
git clone https://github.com/karlrobeck/graph-sql.git
cd graph-sql

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

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
```

### **2. Understand the Codebase**

Start by exploring these key files:
- `src/main.rs` - CLI application entry point
- `src/lib.rs` - Library API and main introspection logic
- `src/types.rs` - GraphQL type generation
- `src/resolvers.rs` - CRUD resolver generation
- `src/cli.rs` - Command-line interface
- `examples/` - Example applications showing library usage

### **3. Run Tests and Examples**

```bash
# Run the test suite
cargo test

# Run the main CLI application
cargo run -- serve

# Try the examples
cd examples/blog
cargo run
```

### **4. Find an Issue**

Look for issues labeled:
- `good first issue` - Perfect for newcomers
- `help wanted` - Community contributions welcome
- `documentation` - Documentation improvements
- `performance` - Performance optimizations
- `feature` - New feature implementations

## 📋 Development Guidelines

### **Code Quality**
- Follow Rust best practices and idioms
- Write comprehensive tests for new features
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` and address warnings
- Document public APIs with rustdoc comments

### **Performance Focus**
graph-sql is designed for high-performance scenarios:
- Profile code changes for performance impact
- Minimize memory allocations in hot paths
- Use efficient data structures
- Consider async/await overhead
- Test with realistic database sizes

### **Database-First Philosophy**
When adding features, consider:
- How can this be implemented in the database layer?
- Can SQLite extensions handle this functionality?
- Does this maintain the stateless gateway design?
- How does this scale with multiple instances?

### **Memory Safety**
Leverage Rust's memory safety:
- Avoid `unsafe` code unless absolutely necessary
- Use proper error handling with `Result` types
- Prefer compile-time checks over runtime checks
- Document any `unsafe` usage thoroughly

## 🔄 Contribution Workflow

### **1. Fork and Branch**
```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/graph-sql.git
cd graph-sql

# Create a feature branch
git checkout -b feature/amazing-feature
```

### **2. Make Changes**
- Write clear, focused commits
- Add tests for new functionality
- Update documentation as needed
- Follow the coding style guide

### **3. Test Your Changes**
```bash
# Run all tests
cargo test

# Test with examples
cd examples/blog
cargo run

# Check formatting and lints
cargo fmt --check
cargo clippy -- -D warnings
```

### **4. Submit a Pull Request**
- Write a clear description of your changes
- Reference any related issues
- Include tests and documentation updates
- Ensure all CI checks pass

## 🏗️ Architecture Guidelines

### **Core Principles**
1. **Database-First**: Business logic belongs in the database
2. **Stateless Gateway**: No application state, pure middleman
3. **Memory Safety**: Leverage Rust's safety guarantees
4. **Performance**: Optimize for heavy-load scenarios
5. **Simplicity**: Minimal configuration and setup

### **Code Organization**
- `src/lib.rs` - Main library interface
- `src/types.rs` - GraphQL type generation from SQLite schema
- `src/resolvers.rs` - CRUD resolver implementations
- `src/traits.rs` - Shared traits and interfaces
- `src/cli.rs` - Command-line interface implementation

### **Adding New Features**
When adding features, consider:
- Does this maintain the database-first philosophy?
- Can this be extended to other databases in the future?
- How does this affect performance?
- Is the API simple and intuitive?

## 🧪 Testing

### **Test Categories**
- **Unit Tests**: Individual function testing
- **Integration Tests**: Full schema generation testing
- **Example Tests**: Ensure examples work correctly
- **Performance Tests**: Benchmark critical paths

### **Running Tests**
```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration
```

## 📝 Documentation

### **Documentation Types**
- **API Documentation**: Rustdoc comments for public APIs
- **User Guides**: README and usage examples
- **Architecture Docs**: Design decisions and patterns
- **Tutorial Content**: Step-by-step guides

### **Writing Guidelines**
- Use clear, concise language
- Include code examples
- Update examples when changing APIs
- Consider multiple skill levels

## 🐛 Reporting Issues

### **Bug Reports**
Include:
- Rust version and operating system
- graph-sql version or commit hash
- Minimal reproduction case
- Expected vs actual behavior
- Error messages and stack traces

### **Feature Requests**
Include:
- Clear description of the feature
- Use case and motivation
- Proposed API or interface
- Consideration of alternatives

## 💬 Community

### **Communication Channels**
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and general discussion
- **Pull Request Reviews**: Code review and collaboration

### **Code of Conduct**
We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and constructive in all interactions.

## 🎯 Priority Areas

We're especially looking for contributions in:

### **High Priority**
- JWT authentication integration
- SQLite extension support
- Performance optimizations
- Advanced filtering capabilities
- Documentation improvements

### **Medium Priority**
- PostgreSQL support
- Real-time subscriptions
- Connection pooling
- Deployment tooling
- Example applications

### **Future Focus**
- Multi-database architecture
- Horizontal scaling tools
- Performance monitoring
- Extension ecosystem

## 📚 Resources

### **Rust Resources**
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### **GraphQL Resources**
- [GraphQL Specification](https://spec.graphql.org/)
- [async-graphql Documentation](https://async-graphql.github.io/async-graphql/en/index.html)

### **SQLite Resources**
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [SQLx Documentation](https://docs.rs/sqlx/)

---

Thank you for contributing to graph-sql! Together, we're building a memory-safe, high-performance GraphQL gateway that maximizes SQLite's potential. 🚀
