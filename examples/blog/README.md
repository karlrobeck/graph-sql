# Blog System Example

This example demonstrates a complete blog system with users, posts, comments,
categories, and tags. It showcases graph-sql's ability to automatically handle
foreign key relationships and generate a comprehensive GraphQL API.

## Features Demonstrated

- **Foreign Key Relationships**: Automatic detection and mapping of
  relationships between tables
- **Many-to-One Relations**: Posts → Authors, Posts → Categories, Comments →
  Posts/Users
- **Many-to-Many Relations**: Posts ↔ Tags (via junction table)
- **Nested Comments**: Self-referencing foreign keys for comment replies
- **Comprehensive CRUD**: Full Create, Read, Update, Delete operations for all
  entities
- **Rich Sample Data**: Realistic blog content with multiple authors and
  interactions

## Schema Overview

```sql
user (authors and commenters)
├── post (blog posts by users)
│   ├── comment (comments on posts, with nested replies)
│   └── post_tag (many-to-many with tags)
├── category (post categories)
└── tag (post tags)
```

## Running the Example

```bash
cd examples/blog
cargo run --bin blog
```

The server will start on `http://localhost:8080` with GraphiQL available for
testing.

## Example Queries

### List all posts with authors

```graphql
{
  post {
    list(input: {page: 1, limit: 5}) {
      id
      title
      excerpt
      is_published
      view_count
      author {
        name
        email
      }
      category {
        name
        color
      }
    }
  }
}
```

### Get a specific post with comments and tags

```graphql
{
  post {
    view(input: {id: 1}) {
      title
      content
      author {
        name
        bio
      }
      category {
        name
        description
      }
    }
  }
  comment {
    list(input: {page: 1, limit: 10}) {
      content
      is_approved
      post {
        title
      }
      author {
        name
      }
      parent_comment {
        content
        author {
          name
        }
      }
    }
  }
  post_tag {
    list(input: {page: 1, limit: 20}) {
      post {
        title
      }
      tag {
        name
        description
      }
    }
  }
}
```

### Create a new blog post

```graphql
mutation {
  insert_post(input: {
    title: "My New Post"
    content: "This is the content of my new blog post..."
    excerpt: "A brief excerpt"
    author_id: 1
    category_id: 2
    is_published: true
  }) {
    id
    title
    author {
      name
    }
    category {
      name
    }
  }
}
```

### Add a comment to a post

```graphql
mutation {
  insert_comment(input: {
    content: "Great post! Very informative."
    post_id: 1
    author_id: 2
    is_approved: true
  }) {
    id
    content
    post {
      title
    }
    author {
      name
    }
  }
}
```

## Database Features Showcased

- **Foreign Key Constraints**: Proper referential integrity with CASCADE and SET
  NULL options
- **Default Values**: Timestamps, boolean flags, and computed defaults
- **Unique Constraints**: Email uniqueness, category names, tag names
- **Self-Referencing FKs**: Nested comments via `parent_comment_id`
- **Junction Tables**: Many-to-many relationships between posts and tags
- **Complex Relationships**: Multiple foreign keys per table (posts have both
  author and category)

This example demonstrates how graph-sql automatically handles complex database
schemas and provides a rich, type-safe GraphQL API with zero configuration.
