-- Simple Task Manager Schema
-- Demonstrates basic CRUD operations and simple relationships
-- Projects to organize tasks
CREATE TABLE project(
  id integer PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL,
  description text,
  color text DEFAULT '#3b82f6',
  is_active boolean DEFAULT 1,
  created_at text DEFAULT (datetime('now')),
  updated_at text DEFAULT (datetime('now'))
);

-- Task categories/labels
CREATE TABLE label(
  id integer PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL UNIQUE,
  color text DEFAULT '#6b7280',
  created_at text DEFAULT (datetime('now'))
);

-- Main tasks table
CREATE TABLE task(
  id integer PRIMARY KEY AUTOINCREMENT,
  title text NOT NULL,
  description text,
  priority text DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
  status text DEFAULT 'todo' CHECK (status IN ('todo', 'in_progress', 'review', 'done', 'cancelled')),
  is_completed boolean DEFAULT 0,
  project_id integer,
  label_id integer,
  estimated_hours real,
  actual_hours real,
  due_date text,
  completed_at text,
  created_at text DEFAULT (datetime('now')),
  updated_at text DEFAULT (datetime('now')),
  FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE SET NULL,
  FOREIGN KEY (label_id) REFERENCES label(id) ON DELETE SET NULL
);

-- Task dependencies (tasks that must be completed before others)
CREATE TABLE task_dependency(
  id integer PRIMARY KEY AUTOINCREMENT,
  task_id integer NOT NULL,
  depends_on_task_id integer NOT NULL,
  created_at text DEFAULT (datetime('now')),
  FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE CASCADE,
  FOREIGN KEY (depends_on_task_id) REFERENCES task(id) ON DELETE CASCADE,
  UNIQUE (task_id, depends_on_task_id),
  CHECK (task_id != depends_on_task_id)
);

-- Time tracking entries
CREATE TABLE time_entry(
  id integer PRIMARY KEY AUTOINCREMENT,
  task_id integer NOT NULL,
  description text,
  hours real NOT NULL CHECK (hours > 0),
  entry_date text DEFAULT (date('now')),
  created_at text DEFAULT (datetime('now')),
  FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE CASCADE
);

-- Insert sample data
-- Projects
INSERT INTO project(name, description, color)
  VALUES ('Website Redesign', 'Complete overhaul of company website with modern design', '#10b981'),
('Mobile App', 'Development of iOS and Android mobile application', '#8b5cf6'),
('API Integration', 'Integration with third-party APIs for data synchronization', '#f59e0b'),
('Documentation', 'Technical documentation and user guides', '#ef4444'),
('Personal', 'Personal tasks and reminders', '#6b7280');

-- Labels
INSERT INTO label(name, color)
  VALUES ('bug', '#ef4444'),
('feature', '#10b981'),
('enhancement', '#3b82f6'),
('documentation', '#f59e0b'),
('urgent', '#dc2626'),
('meeting', '#8b5cf6'),
('review', '#06b6d4'),
('testing', '#84cc16');

-- Tasks
INSERT INTO task(title, description, priority, status, project_id, label_id, estimated_hours, due_date)
  VALUES ('Design new homepage layout', 'Create wireframes and mockups for the new homepage design with improved user experience', 'high', 'in_progress', 1, 2, 8.0, date('now', '+3 days')),
('Implement user authentication', 'Set up secure user login and registration system with JWT tokens', 'high', 'todo', 2, 2, 12.0, date('now', '+5 days')),
('Fix mobile responsive issues', 'Address layout problems on mobile devices, particularly on screens smaller than 768px', 'medium', 'todo', 1, 1, 4.0, date('now', '+2 days')),
('API documentation update', 'Update REST API documentation with new endpoints and examples', 'medium', 'review', 4, 4, 6.0, date('now', '+1 day')),
('Set up CI/CD pipeline', 'Configure automated testing and deployment pipeline using GitHub Actions', 'high', 'todo', 3, 3, 10.0, date('now', '+7 days')),
('Database optimization', 'Optimize slow queries and add necessary indexes for better performance', 'medium', 'done', 3, 3, 8.0, date('now', '-2 days')),
('User testing session', 'Conduct usability testing with 5-10 users to gather feedback on new features', 'low', 'todo', 1, 8, 4.0, date('now', '+10 days')),
('Weekly team meeting', 'Regular team sync to discuss progress and blockers', 'medium', 'done', NULL, 6, 1.0, date('now', '-1 day')),
('Code review guidelines', 'Establish and document code review standards and best practices', 'low', 'todo', 4, 4, 3.0, date('now', '+14 days')),
('Buy groceries', 'Weekly grocery shopping - milk, bread, eggs, vegetables', 'low', 'todo', 5, NULL, 1.0, date('now', '+1 day'));

-- Mark some tasks as completed
UPDATE
  task
SET
  is_completed = 1,
  completed_at = datetime('now', '-1 day'),
  status = 'done'
WHERE
  id IN (6, 8);

-- Task dependencies
INSERT INTO task_dependency(task_id, depends_on_task_id)
  VALUES (2, 1), -- User auth depends on homepage design
(7, 1), -- User testing depends on homepage design
(7, 3), -- User testing depends on mobile fixes
(5, 4);

-- CI/CD depends on API documentation
-- Time entries
INSERT INTO time_entry(task_id, description, hours, entry_date)
  VALUES (1, 'Initial wireframe sketches', 2.5, date('now', '-2 days')),
(1, 'Detailed mockup creation', 3.0, date('now', '-1 day')),
(1, 'Client feedback incorporation', 1.5, date('now')),
(3, 'Responsive design analysis', 1.0, date('now', '-1 day')),
(4, 'API endpoint documentation', 2.0, date('now', '-3 days')),
(4, 'Example code writing', 1.5, date('now', '-2 days')),
(6, 'Query analysis and optimization', 4.0, date('now', '-3 days')),
(6, 'Index implementation', 2.5, date('now', '-2 days')),
(8, 'Team meeting preparation', 0.5, date('now', '-2 days')),
(8, 'Meeting facilitation', 1.0, date('now', '-1 day'));

