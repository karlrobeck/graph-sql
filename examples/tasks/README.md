# Task Manager Example

This example demonstrates a simple but complete task management system with projects, labels, time tracking, and task dependencies. It's perfect for understanding basic CRUD operations and simple relationships in graph-sql.

## Features Demonstrated

- **Basic CRUD Operations**: Create, read, update, and delete tasks
- **Simple Relationships**: Tasks belong to projects and have labels
- **Self-Referencing FKs**: Task dependencies (tasks that depend on other tasks)
- **Time Tracking**: Log hours spent on tasks
- **Status Management**: Task workflow from todo to done
- **Data Validation**: Priority levels, status constraints, and check constraints
- **Computed Fields**: Completion tracking and time calculations

## Schema Overview

```sql
project (task organization)
├── task (main task entities)
│   ├── task_dependency (task prerequisites)
│   └── time_entry (time tracking)
└── label (task categorization)
    └── task (labeled tasks)
```

## Running the Example

```bash
cd examples/tasks
cargo run --bin tasks
```

The server will start on `http://localhost:8082` with GraphiQL available for testing.

## Example Queries

### List all tasks with project and label info
```graphql
{
  task {
    list(input: {page: 1, limit: 10}) {
      id
      title
      description
      priority
      status
      is_completed
      estimated_hours
      due_date
      project {
        name
        color
      }
      label {
        name
        color
      }
    }
  }
}
```

### View a specific task with all details
```graphql
{
  task {
    view(input: {id: 1}) {
      title
      description
      priority
      status
      is_completed
      estimated_hours
      actual_hours
      due_date
      completed_at
      project {
        name
        description
      }
      label {
        name
        color
      }
    }
  }
}
```

### Get task dependencies
```graphql
{
  task_dependency {
    list(input: {page: 1, limit: 20}) {
      task {
        title
        status
      }
      depends_on_task {
        title
        status
        is_completed
      }
    }
  }
}
```

### View time tracking entries
```graphql
{
  time_entry {
    list(input: {page: 1, limit: 15}) {
      description
      hours
      entry_date
      task {
        title
        project {
          name
        }
      }
    }
  }
}
```

### Project overview with tasks
```graphql
{
  project {
    list(input: {page: 1, limit: 5}) {
      name
      description
      color
      is_active
    }
  }
  task {
    list(input: {page: 1, limit: 20}) {
      title
      status
      priority
      project {
        name
      }
    }
  }
}
```

## Example Mutations

### Create a new task
```graphql
mutation {
  insert_task(input: {
    title: "Implement search functionality"
    description: "Add full-text search capability to the application"
    priority: "high"
    status: "todo"
    project_id: 2
    label_id: 2
    estimated_hours: 6.0
    due_date: "2025-08-10"
  }) {
    id
    title
    priority
    project {
      name
    }
    label {
      name
    }
  }
}
```

### Update task status and mark as completed
```graphql
mutation {
  update_task(id: 3, input: {
    status: "done"
    is_completed: true
    actual_hours: 3.5
    completed_at: "2025-08-03T14:30:00"
  }) {
    id
    title
    status
    is_completed
    actual_hours
    completed_at
  }
}
```

### Create a new project
```graphql
mutation {
  insert_project(input: {
    name: "Marketing Campaign"
    description: "Q4 marketing initiatives and content creation"
    color: "#f97316"
  }) {
    id
    name
    description
    color
  }
}
```

### Log time on a task
```graphql
mutation {
  insert_time_entry(input: {
    task_id: 1
    description: "Debugging responsive layout issues"
    hours: 2.5
    entry_date: "2025-08-03"
  }) {
    id
    description
    hours
    task {
      title
      project {
        name
      }
    }
  }
}
```

### Create task dependency
```graphql
mutation {
  insert_task_dependency(input: {
    task_id: 11
    depends_on_task_id: 2
  }) {
    id
    task {
      title
    }
    depends_on_task {
      title
    }
  }
}
```

### Delete a completed task
```graphql
mutation {
  delete_task(input: {id: 8}) {
    rows_affected
  }
}
```

## Key Features

- **Priority Management**: Four priority levels (low, medium, high, urgent)
- **Status Workflow**: Five status states (todo, in_progress, review, done, cancelled)
- **Project Organization**: Group related tasks under projects
- **Label System**: Categorize tasks with colored labels
- **Time Tracking**: Log actual time spent vs. estimates
- **Task Dependencies**: Model task prerequisites and blockers
- **Due Date Management**: Track deadlines and completion dates
- **Data Integrity**: Constraints prevent invalid data (e.g., self-dependencies)

This example is ideal for learning graph-sql basics while building a practical task management system. It demonstrates how foreign key relationships automatically become navigable GraphQL fields, making it easy to traverse from tasks to projects, labels, dependencies, and time entries.
