---
pageType: home

hero:
  name: graph-sql
  text: Memory-Safe GraphQL Gateway
  tagline: High-performance Rust CLI tool that automatically introspects SQLite databases and generates complete GraphQL APIs
  actions:
    - theme: brand
      text: Quick Start
      link: /guide/
    - theme: alt
      text: GitHub
      link: https://github.com/karlrobeck/graph-sql
  image:
    src: /rspress-icon.png
    alt: graph-sql Logo
features:
  - title: Memory Safety
    details: Leverages Rust's zero-cost abstractions and memory safety guarantees to eliminate entire classes of bugs common in traditional API servers.
    icon: 🔒
  - title: Database-First Architecture
    details: Acts as a stateless gateway/middleman, letting SQLite handle business logic for optimal performance and consistency.
    icon: 🏗️
  - title: High Performance
    details: Designed for heavy-load scenarios with minimal resource overhead and efficient concurrency using Tokio async runtime.
    icon: ⚡
  - title: Auto-Generated Schema
    details: Automatically introspects SQLite databases and generates complete GraphQL schemas with CRUD operations and relationships.
    icon: 🔍
  - title: Single Binary Deployment
    details: No runtime dependencies or complex installation requirements. Deploy anywhere with a single compiled binary.
    icon: 📦
  - title: TOML Configuration
    details: Simple, structured configuration files for all server and database settings. No complex setup required.
    icon: ⚙️
---
