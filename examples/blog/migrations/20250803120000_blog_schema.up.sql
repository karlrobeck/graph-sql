-- Blog System Schema
-- Demonstrates foreign key relationships and comprehensive CRUD operations
-- Users table
create table user (
  id integer primary key AUTOINCREMENT,
  name text not null,
  email text unique not null,
  bio text,
  avatar_url text,
  is_active boolean default 1,
  created_at text default (datetime('now')),
  updated_at text default (datetime('now'))
);

-- Categories table
create table category(
  id integer primary key AUTOINCREMENT,
  name text not null unique,
  description text,
  color text,
  created_at text default (datetime('now'))
);

-- Posts table with foreign key to users and categories
create table post(
  id integer primary key AUTOINCREMENT,
  title text not null,
  content text not null,
  excerpt text,
  author_id integer not null,
  category_id integer,
  is_published boolean default 0,
  view_count integer default 0,
  created_at text default (datetime('now')),
  updated_at text default (datetime('now')),
  published_at text,
  foreign key (author_id) references user (id) on delete cascade,
  foreign key (category_id) references category(id) on delete set null
);

-- Comments table with foreign keys to posts and users
create table comment(
  id integer primary key AUTOINCREMENT,
  content text not null,
  post_id integer not null,
  author_id integer,
  parent_comment_id integer, -- For nested comments
  is_approved boolean default 0,
  created_at text default (datetime('now')),
  updated_at text default (datetime('now')),
  foreign key (post_id) references post(id) on delete cascade,
  foreign key (author_id) references user (id) on delete set null,
  foreign key (parent_comment_id) references comment(id) on delete cascade
);

-- Tags table
create table tag(
  id integer primary key AUTOINCREMENT,
  name text not null unique,
  description text,
  created_at text default (datetime('now'))
);

-- Many-to-many relationship between posts and tags
create table post_tag(
  id integer primary key AUTOINCREMENT,
  post_id integer not null,
  tag_id integer not null,
  created_at text default (datetime('now')),
  foreign key (post_id) references post(id) on delete cascade,
  foreign key (tag_id) references tag(id) on delete cascade,
  unique (post_id, tag_id)
);

-- Insert sample data
insert into user (name, email, bio, avatar_url)
  values ('Alice Johnson', 'alice@example.com', 'Senior developer and tech blogger', 'https://example.com/avatar1.jpg'),
('Bob Smith', 'bob@example.com', 'Frontend specialist and UI/UX enthusiast', 'https://example.com/avatar2.jpg'),
('Carol Davis', 'carol@example.com', 'DevOps engineer and cloud architect', 'https://example.com/avatar3.jpg'),
('David Wilson', 'david@example.com', 'Full-stack developer and open source contributor', 'https://example.com/avatar4.jpg');

insert into category(name, description, color)
  values ('Technology', 'Posts about programming, software development, and tech trends', '#2563eb'),
('Tutorial', 'Step-by-step guides and educational content', '#059669'),
('Opinion', 'Personal thoughts and industry insights', '#dc2626'),
('News', 'Latest updates and announcements', '#7c3aed');

insert into tag(name, description)
  values ('rust', 'Posts about Rust programming language'),
('graphql', 'GraphQL related content'),
('database', 'Database design and optimization'),
('web-development', 'Web development techniques and frameworks'),
('performance', 'Performance optimization and best practices'),
('tutorial', 'Educational and how-to content');

insert into post(title, content, excerpt, author_id, category_id, is_published, view_count, published_at)
  values ('Getting Started with GraphQL and Rust', 'GraphQL has revolutionized how we think about APIs. In this comprehensive guide, we''ll explore how to build a GraphQL server using Rust and the async-graphql crate. We''ll cover schema design, resolvers, and best practices for building scalable APIs.', 'Learn how to build powerful GraphQL APIs with Rust and async-graphql', 1, 2, 1, 1247, datetime('now', '-5 days')),
('SQLite Performance Optimization Tips', 'SQLite is often underestimated, but with proper optimization, it can handle significant workloads. This post covers indexing strategies, query optimization, and configuration tweaks that can dramatically improve your SQLite performance.', 'Unlock the full potential of SQLite with these performance optimization techniques', 3, 1, 1, 892, datetime('now', '-3 days')),
('The Future of Web APIs: Why GraphQL Matters', 'REST has served us well, but GraphQL represents the next evolution in API design. In this opinion piece, I discuss why GraphQL''s type safety, introspection, and flexibility make it the ideal choice for modern applications.', 'Exploring why GraphQL is becoming the standard for modern API development', 2, 3, 1, 567, datetime('now', '-1 day')),
('Building Real-time Applications with Rust', 'Real-time applications require careful consideration of performance and concurrency. This tutorial demonstrates how to build a real-time chat application using Rust, WebSockets, and tokio for async programming.', 'Step-by-step guide to building real-time apps with Rust and WebSockets', 4, 2, 1, 234, datetime('now', '-6 hours')),
('Database Design Patterns for Modern Apps', 'Good database design is the foundation of any successful application. This post explores common patterns, normalization strategies, and how to design schemas that scale with your application.', 'Essential database design patterns every developer should know', 1, 1, 0, 0, null);

insert into post_tag(post_id, tag_id)
  values (1, 1),
(1, 2),
(1, 6), -- GraphQL + Rust post
(2, 3),
(2, 5), -- SQLite performance post
(3, 2), -- GraphQL opinion post
(4, 1),
(4, 4),
(4, 6), -- Real-time Rust post
(5, 3),
(5, 4);

-- Database design post
insert into comment(content, post_id, author_id, is_approved)
  values ('Great introduction to GraphQL with Rust! The examples are very clear.', 1, 2, 1),
('Thanks for this tutorial. I''ve been looking for a good Rust GraphQL guide.', 1, 3, 1),
('Could you add more examples about error handling in resolvers?', 1, 4, 1),
('These SQLite tips are gold! My queries are now 3x faster.', 2, 1, 1),
('The indexing section was particularly helpful. Thanks!', 2, 4, 1),
('Completely agree about GraphQL''s advantages. REST feels outdated now.', 3, 1, 1),
('While I like GraphQL, REST still has its place in simpler applications.', 3, 3, 1),
('The WebSocket implementation looks clean. Will definitely try this approach.', 4, 2, 1),
('Any plans for a follow-up post about scaling real-time apps?', 4, 1, 1);

-- Add a nested comment (reply to first comment)
insert into comment(content, post_id, author_id, parent_comment_id, is_approved)
  values ('I agree! The async-graphql crate makes it really straightforward.', 1, 1, 1, 1);

