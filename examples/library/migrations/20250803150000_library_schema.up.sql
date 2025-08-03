-- Library Management System Schema
-- Demonstrates various SQLite data types and their GraphQL mappings
-- Authors table
CREATE TABLE author(
  id integer PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL,
  birth_year integer,
  death_year integer,
  nationality text,
  biography text,
  photo BLOB, -- Demonstrates BLOB → String mapping
  rating numeric(3, 2), -- Demonstrates NUMERIC → String mapping
  is_active boolean DEFAULT 1,
  created_at text DEFAULT (datetime('now'))
);

-- Publishers table
CREATE TABLE publisher(
  id integer PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL UNIQUE,
  address text,
  website text,
  founded_year integer,
  logo BLOB,
  is_active boolean DEFAULT 1
);

-- Book genres
CREATE TABLE genre(
  id integer PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL UNIQUE,
  description text,
  parent_genre_id integer,
  FOREIGN KEY (parent_genre_id) REFERENCES genre(id)
);

-- Books table - demonstrates various data types
CREATE TABLE book(
  id integer PRIMARY KEY AUTOINCREMENT,
  isbn text UNIQUE NOT NULL,
  title text NOT NULL,
  subtitle text,
  author_id integer NOT NULL,
  publisher_id integer,
  genre_id integer,
  publication_year integer,
  page_count integer CHECK (page_count > 0),
  price real CHECK (price >= 0),
  weight real, -- in grams
  dimensions text, -- "height x width x depth cm"
  language TEXT
  DEFAULT 'English',
  description text,
  cover_image BLOB,
  rating numeric(2, 1) CHECK (rating >= 0 AND rating <= 5), -- 0.0 to 5.0
  total_copies integer DEFAULT 1 CHECK (total_copies > 0),
  available_copies integer DEFAULT 1 CHECK (available_copies >= 0),
  is_digital boolean DEFAULT 0,
  digital_file BLOB, -- For digital books
  created_at text DEFAULT (datetime('now')),
  updated_at text DEFAULT (datetime('now')),
  FOREIGN KEY (author_id) REFERENCES author(id),
  FOREIGN KEY (publisher_id) REFERENCES publisher(id),
  FOREIGN KEY (genre_id) REFERENCES genre(id),
  CHECK (available_copies <= total_copies)
);

-- Library members
CREATE TABLE member(
  id integer PRIMARY KEY AUTOINCREMENT,
  membership_number text UNIQUE NOT NULL,
  first_name text NOT NULL,
  last_name text NOT NULL,
  email text UNIQUE NOT NULL,
  phone text,
  address text,
  date_of_birth text,
  membership_type text DEFAULT 'standard' CHECK (membership_type IN ('standard', 'premium', 'student', 'senior')),
  registration_date text DEFAULT (date('now')),
  expiry_date text,
  is_active boolean DEFAULT 1,
  fine_amount real DEFAULT 0.0 CHECK (fine_amount >= 0),
  max_books integer DEFAULT 5 CHECK (max_books > 0),
  profile_photo BLOB
);

-- Book loans/borrowing
CREATE TABLE loan(
  id integer PRIMARY KEY AUTOINCREMENT,
  book_id integer NOT NULL,
  member_id integer NOT NULL,
  loan_date text DEFAULT (date('now')),
  due_date text NOT NULL,
  return_date text,
  is_returned boolean DEFAULT 0,
  fine_amount real DEFAULT 0.0 CHECK (fine_amount >= 0),
  renewal_count integer DEFAULT 0 CHECK (renewal_count >= 0),
  notes text,
  created_at text DEFAULT (datetime('now')),
  FOREIGN KEY (book_id) REFERENCES book(id),
  FOREIGN KEY (member_id) REFERENCES member(id)
);

-- Book reservations
CREATE TABLE reservation(
  id integer PRIMARY KEY AUTOINCREMENT,
  book_id integer NOT NULL,
  member_id integer NOT NULL,
  reservation_date text DEFAULT (date('now')),
  expiry_date text NOT NULL,
  is_fulfilled boolean DEFAULT 0,
  is_cancelled boolean DEFAULT 0,
  notification_sent boolean DEFAULT 0,
  FOREIGN KEY (book_id) REFERENCES book(id),
  FOREIGN KEY (member_id) REFERENCES member(id)
);

-- Book reviews by members
CREATE TABLE review(
  id integer PRIMARY KEY AUTOINCREMENT,
  book_id integer NOT NULL,
  member_id integer NOT NULL,
  rating integer NOT NULL CHECK (rating >= 1 AND rating <= 5),
  title text,
  content text,
  is_spoiler boolean DEFAULT 0,
  helpful_votes integer DEFAULT 0,
  review_date text DEFAULT (date('now')),
  FOREIGN KEY (book_id) REFERENCES book(id) ON DELETE CASCADE,
  FOREIGN KEY (member_id) REFERENCES member(id) ON DELETE CASCADE,
  UNIQUE (book_id, member_id)
);

-- Insert sample data
-- Authors
INSERT INTO author(name, birth_year, death_year, nationality, biography, rating, is_active)
  VALUES ('J.K. Rowling', 1965, NULL, 'British', 'British author best known for the Harry Potter series.', 4.85, 1),
('George Orwell', 1903, 1950, 'British', 'English novelist and journalist, famous for dystopian fiction.', 4.72, 1),
('Agatha Christie', 1890, 1976, 'British', 'British writer known for detective novels, especially Hercule Poirot and Miss Marple.', 4.56, 1),
('Isaac Asimov', 1920, 1992, 'American', 'American science fiction writer and biochemistry professor.', 4.43, 1),
('Maya Angelou', 1928, 2014, 'American', 'American memoirist, poet, and civil rights activist.', 4.67, 1),
('Haruki Murakami', 1949, NULL, 'Japanese', 'Contemporary Japanese writer known for surreal fiction.', 4.34, 1);

-- Publishers
INSERT INTO publisher(name, address, website, founded_year, is_active)
  VALUES ('Penguin Random House', 'New York, NY', 'https://penguinrandomhouse.com', 1927, 1),
('HarperCollins', 'New York, NY', 'https://harpercollins.com', 1989, 1),
('Scholastic', 'New York, NY', 'https://scholastic.com', 1920, 1),
('Vintage Books', 'New York, NY', 'https://vintagebooks.com', 1954, 1),
('Knopf', 'New York, NY', 'https://knopf.com', 1915, 1);

-- Genres (with hierarchical structure)
INSERT INTO genre(name, description, parent_genre_id)
  VALUES ('Fiction', 'Literary works of imagination', NULL),
('Non-Fiction', 'Factual and informational works', NULL),
('Science Fiction', 'Speculative fiction with futuristic concepts', 1),
('Fantasy', 'Fiction involving magical or supernatural elements', 1),
('Mystery', 'Fiction dealing with puzzling crimes or strange events', 1),
('Romance', 'Fiction focused on romantic relationships', 1),
('Biography', 'Life stories of real people', 2),
('History', 'Records and interpretation of past events', 2),
('Science', 'Natural sciences and technology', 2),
('Self-Help', 'Guides for personal improvement', 2);

-- Books
INSERT INTO book(isbn, title, subtitle, author_id, publisher_id, genre_id, publication_year, page_count, price, weight, dimensions, language, description, rating, total_copies, available_copies)
  VALUES ('978-0439708180', 'Harry Potter and the Philosopher''s Stone', NULL, 1, 3, 4, 1997, 223, 12.99, 340, '19.7 x 12.9 x 1.4 cm', 'English', 'The first novel in the Harry Potter series and J.K. Rowling''s debut novel.', 4.8, 5, 3),
('978-0452284234', 'Nineteen Eighty-Four', NULL, 2, 4, 3, 1949, 328, 15.99, 280, '19.8 x 12.9 x 2.1 cm', 'English', 'A dystopian social science fiction novel and cautionary tale.', 4.7, 4, 2),
('978-0062073488', 'Murder on the Orient Express', NULL, 3, 2, 5, 1934, 256, 14.99, 260, '19.4 x 12.8 x 1.6 cm', 'English', 'A detective novel featuring Hercule Poirot.', 4.5, 3, 1),
('978-0553293357', 'Foundation', NULL, 4, 1, 3, 1951, 244, 16.99, 220, '17.8 x 10.7 x 1.6 cm', 'English', 'The first novel in Asimov''s Foundation series.', 4.4, 2, 2),
('978-0345806567', 'I Know Why the Caged Bird Sings', NULL, 5, 1, 7, 1969, 281, 13.99, 254, '20.3 x 13.3 x 1.8 cm', 'English', 'The first in a seven-volume autobiographical series.', 4.6, 3, 2),
('978-0679775430', 'Norwegian Wood', NULL, 6, 5, 1, 1987, 296, 17.99, 290, '20.1 x 13.2 x 2.0 cm', 'English', 'A coming-of-age novel set in 1960s Tokyo.', 4.3, 2, 1);

-- Members
INSERT INTO member(membership_number, first_name, last_name, email, phone, address, date_of_birth, membership_type, expiry_date, max_books)
  VALUES ('LIB001', 'Alice', 'Johnson', 'alice.johnson@email.com', '+1-555-0101', '123 Oak Street, Springfield', '1985-03-15', 'premium', date('now', '+1 year'), 10),
('LIB002', 'Bob', 'Smith', 'bob.smith@email.com', '+1-555-0102', '456 Elm Avenue, Springfield', '1992-07-22', 'standard', date('now', '+1 year'), 5),
('LIB003', 'Carol', 'Davis', 'carol.davis@email.com', '+1-555-0103', '789 Pine Road, Springfield', '1978-11-08', 'standard', date('now', '+1 year'), 5),
('LIB004', 'David', 'Wilson', 'david.wilson@email.com', '+1-555-0104', '321 Maple Drive, Springfield', '2000-02-14', 'student', date('now', '+1 year'), 8),
('LIB005', 'Eva', 'Brown', 'eva.brown@email.com', '+1-555-0105', '654 Cedar Lane, Springfield', '1945-09-30', 'senior', date('now', '+1 year'), 7);

-- Loans (some current, some returned)
INSERT INTO loan(book_id, member_id, loan_date, due_date, return_date, is_returned, renewal_count)
  VALUES (1, 1, date('now', '-10 days'), date('now', '+4 days'), NULL, 0, 0), -- Alice has Harry Potter (due soon)
(2, 2, date('now', '-5 days'), date('now', '+9 days'), NULL, 0, 1), -- Bob has 1984 (renewed once)
(3, 3, date('now', '-20 days'), date('now', '-6 days'), date('now', '-3 days'), 1, 0), -- Carol returned Murder (was overdue)
(4, 4, date('now', '-7 days'), date('now', '+7 days'), NULL, 0, 0), -- David has Foundation
(5, 5, date('now', '-15 days'), date('now', '-1 day'), NULL, 0, 0), -- Eva has Caged Bird (overdue)
(6, 1, date('now', '-25 days'), date('now', '-11 days'), date('now', '-8 days'), 1, 0);

-- Alice returned Norwegian Wood
-- Update fines for overdue books
UPDATE
  loan
SET
  fine_amount = 2.50
WHERE
  id = 5;

-- Eva's overdue fine
-- Reservations
INSERT INTO reservation(book_id, member_id, reservation_date, expiry_date, is_fulfilled)
  VALUES (1, 3, date('now', '-2 days'), date('now', '+5 days'), 0), -- Carol waiting for Harry Potter
(3, 5, date('now', '-1 day'), date('now', '+6 days'), 0), -- Eva waiting for Murder on Orient Express
(2, 4, date('now', '-5 days'), date('now', '+2 days'), 1);

-- David's reservation fulfilled
-- Reviews
INSERT INTO review(book_id, member_id, rating, title, content, helpful_votes, review_date)
  VALUES (1, 1, 5, 'Magical and captivating', 'This book started my love for fantasy literature. Rowling created an incredible world.', 12, date('now', '-30 days')),
(1, 3, 5, 'A classic for all ages', 'Perfect introduction to the wizarding world. Great character development.', 8, date('now', '-15 days')),
(2, 2, 5, 'Chillingly prophetic', 'Orwell''s vision of the future feels more relevant than ever. A must-read.', 15, date('now', '-20 days')),
(3, 3, 4, 'Classic mystery', 'Christie''s plotting is masterful. The solution is both surprising and logical.', 6, date('now', '-10 days')),
(4, 4, 4, 'Brilliant sci-fi foundation', 'Asimov''s Foundation series starts strong with this book. Complex but rewarding.', 9, date('now', '-5 days')),
(5, 5, 5, 'Powerful and moving', 'Maya Angelou''s storytelling is both beautiful and heartbreaking. Life-changing read.', 11, date('now', '-25 days')),
(6, 1, 3, 'Melancholic but slow', 'Beautifully written but didn''t quite connect with me. Others might love it more.', 3, date('now', '-35 days'));

