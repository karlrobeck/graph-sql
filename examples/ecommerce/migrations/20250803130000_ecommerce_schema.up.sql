-- E-commerce System Schema
-- Demonstrates complex business relationships and data integrity
-- Categories for products
CREATE TABLE category(
  id integer PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL UNIQUE,
  description text,
  parent_category_id integer, -- For hierarchical categories
  is_active boolean DEFAULT 1,
  created_at text DEFAULT (datetime('now')),
  FOREIGN KEY (parent_category_id) REFERENCES category(id)
);

-- Product catalog
CREATE TABLE product(
  id integer PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL,
  description text,
  price real NOT NULL CHECK (price >= 0),
  cost_price real CHECK (cost_price >= 0),
  sku text UNIQUE NOT NULL,
  category_id integer,
  brand text,
  weight real,
  dimensions text, -- JSON-like: "10x5x2 cm"
  is_active boolean DEFAULT 1,
  is_digital boolean DEFAULT 0,
  stock_quantity integer DEFAULT 0 CHECK (stock_quantity >= 0),
  low_stock_threshold integer DEFAULT 5,
  created_at text DEFAULT (datetime('now')),
  updated_at text DEFAULT (datetime('now')),
  FOREIGN KEY (category_id) REFERENCES category(id)
);

-- Customers
CREATE TABLE customer(
  id integer PRIMARY KEY AUTOINCREMENT,
  first_name text NOT NULL,
  last_name text NOT NULL,
  email text UNIQUE NOT NULL,
  phone text,
  date_of_birth text,
  is_active boolean DEFAULT 1,
  loyalty_points integer DEFAULT 0,
  created_at text DEFAULT (datetime('now')),
  updated_at text DEFAULT (datetime('now'))
);

-- Customer addresses (one customer can have multiple addresses)
CREATE TABLE address(
  id integer PRIMARY KEY AUTOINCREMENT,
  customer_id integer NOT NULL,
  type TEXT NOT NULL CHECK (type IN ('billing', 'shipping', 'both')),
  street text NOT NULL,
  city text NOT NULL,
  state text,
  postal_code text NOT NULL,
  country text NOT NULL DEFAULT 'US',
  is_default boolean DEFAULT 0,
  created_at text DEFAULT (datetime('now')),
  FOREIGN KEY (customer_id) REFERENCES customer(id) ON DELETE CASCADE
);

-- Orders
CREATE TABLE "order"(
  id integer PRIMARY KEY AUTOINCREMENT,
  customer_id integer NOT NULL,
  billing_address_id integer,
  shipping_address_id integer,
  order_number text UNIQUE NOT NULL,
  status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled', 'refunded')),
  subtotal real NOT NULL CHECK (subtotal >= 0),
  tax_amount real DEFAULT 0 CHECK (tax_amount >= 0),
  shipping_amount real DEFAULT 0 CHECK (shipping_amount >= 0),
  discount_amount real DEFAULT 0 CHECK (discount_amount >= 0),
  total_amount real NOT NULL CHECK (total_amount >= 0),
  currency text DEFAULT 'USD',
  payment_status text DEFAULT 'pending' CHECK (payment_status IN ('pending', 'paid', 'failed', 'refunded')),
  notes text,
  ordered_at text DEFAULT (datetime('now')),
  shipped_at text,
  delivered_at text,
  FOREIGN KEY (customer_id) REFERENCES customer(id),
  FOREIGN KEY (billing_address_id) REFERENCES address(id),
  FOREIGN KEY (shipping_address_id) REFERENCES address(id)
);

-- Order items (products in an order)
CREATE TABLE order_item(
  id integer PRIMARY KEY AUTOINCREMENT,
  order_id integer NOT NULL,
  product_id integer NOT NULL,
  quantity integer NOT NULL CHECK (quantity > 0),
  unit_price real NOT NULL CHECK (unit_price >= 0),
  total_price real NOT NULL CHECK (total_price >= 0),
  created_at text DEFAULT (datetime('now')),
  FOREIGN KEY (order_id) REFERENCES "order"(id) ON DELETE CASCADE,
  FOREIGN KEY (product_id) REFERENCES product(id)
);

-- Product reviews
CREATE TABLE review(
  id integer PRIMARY KEY AUTOINCREMENT,
  product_id integer NOT NULL,
  customer_id integer,
  rating integer NOT NULL CHECK (rating >= 1 AND rating <= 5),
  title text,
  content text,
  is_verified_purchase boolean DEFAULT 0,
  is_approved boolean DEFAULT 0,
  helpful_votes integer DEFAULT 0,
  created_at text DEFAULT (datetime('now')),
  FOREIGN KEY (product_id) REFERENCES product(id) ON DELETE CASCADE,
  FOREIGN KEY (customer_id) REFERENCES customer(id) ON DELETE SET NULL
);

-- Shopping cart (for current session)
CREATE TABLE cart_item(
  id integer PRIMARY KEY AUTOINCREMENT,
  customer_id integer NOT NULL,
  product_id integer NOT NULL,
  quantity integer NOT NULL CHECK (quantity > 0),
  added_at text DEFAULT (datetime('now')),
  updated_at text DEFAULT (datetime('now')),
  FOREIGN KEY (customer_id) REFERENCES customer(id) ON DELETE CASCADE,
  FOREIGN KEY (product_id) REFERENCES product(id) ON DELETE CASCADE,
  UNIQUE (customer_id, product_id)
);

-- Insert sample data
-- Categories
INSERT INTO category(name, description, parent_category_id)
  VALUES ('Electronics', 'Electronic devices and accessories', NULL),
('Computers', 'Desktop and laptop computers', 1),
('Smartphones', 'Mobile phones and accessories', 1),
('Accessories', 'Computer and phone accessories', 1),
('Clothing', 'Fashion and apparel', NULL),
('Men''s Clothing', 'Clothing for men', 5),
('Women''s Clothing', 'Clothing for women', 5),
('Books', 'Physical and digital books', NULL),
('Fiction', 'Fiction books and novels', 8),
('Technical', 'Programming and technical books', 8);

-- Products
INSERT INTO product(name, description, price, cost_price, sku, category_id, brand, stock_quantity)
  VALUES ('MacBook Pro 16"', 'Apple MacBook Pro with M2 chip, 16GB RAM, 512GB SSD', 2499.00, 2000.00, 'MBP-16-M2-512', 2, 'Apple', 15),
('iPhone 15 Pro', 'Latest iPhone with A17 Pro chip, 128GB storage', 999.00, 750.00, 'IPH-15-PRO-128', 3, 'Apple', 42),
('Dell XPS 13', 'Premium ultrabook with Intel i7, 16GB RAM, 512GB SSD', 1299.00, 980.00, 'DELL-XPS13-I7', 2, 'Dell', 8),
('Samsung Galaxy S24', 'Android flagship with 256GB storage', 899.00, 680.00, 'SAM-S24-256', 3, 'Samsung', 23),
('Wireless Mouse', 'Ergonomic wireless mouse with USB-C charging', 79.99, 35.00, 'MOUSE-WL-001', 4, 'Logitech', 156),
('USB-C Hub', '7-in-1 USB-C hub with HDMI, USB-A, and card readers', 49.99, 22.00, 'HUB-USBC-7IN1', 4, 'Anker', 89),
('Men''s T-Shirt', 'Premium cotton t-shirt in various colors', 29.99, 12.00, 'TSHIRT-M-COTTON', 6, 'Premium Basics', 200),
('Women''s Jeans', 'Slim fit denim jeans, premium quality', 89.99, 45.00, 'JEANS-W-SLIM', 7, 'Denim Co', 67),
('Programming Book', 'Learn Rust Programming - Complete Guide', 49.99, 20.00, 'BOOK-RUST-PROG', 10, 'Tech Publishers', 34),
('Mystery Novel', 'The Digital Detective - A cyberpunk thriller', 15.99, 6.00, 'BOOK-MYSTERY-DD', 9, 'Fiction House', 78);

-- Customers
INSERT INTO customer(first_name, last_name, email, phone, loyalty_points)
  VALUES ('John', 'Doe', 'john.doe@example.com', '+1-555-0101', 250),
('Jane', 'Smith', 'jane.smith@example.com', '+1-555-0102', 180),
('Mike', 'Johnson', 'mike.j@example.com', '+1-555-0103', 420),
('Sarah', 'Wilson', 'sarah.wilson@example.com', '+1-555-0104', 95),
('David', 'Brown', 'david.brown@example.com', '+1-555-0105', 340);

-- Addresses
INSERT INTO address(customer_id, type, street, city, state, postal_code, country, is_default)
  VALUES (1, 'both', '123 Main St', 'San Francisco', 'CA', '94105', 'US', 1),
(1, 'shipping', '456 Work Ave', 'San Francisco', 'CA', '94107', 'US', 0),
(2, 'both', '789 Oak Dr', 'Los Angeles', 'CA', '90210', 'US', 1),
(3, 'both', '321 Pine St', 'Seattle', 'WA', '98101', 'US', 1),
(4, 'both', '654 Elm Ave', 'Portland', 'OR', '97201', 'US', 1),
(5, 'both', '987 Cedar Blvd', 'Austin', 'TX', '73301', 'US', 1);

-- Orders
INSERT INTO "order"(customer_id, billing_address_id, shipping_address_id, order_number, status, subtotal, tax_amount, shipping_amount, total_amount, payment_status)
  VALUES (1, 1, 1, 'ORD-2025-001', 'delivered', 2578.99, 206.32, 15.99, 2801.30, 'paid'),
(2, 3, 3, 'ORD-2025-002', 'shipped', 949.98, 75.98, 12.99, 1038.95, 'paid'),
(3, 4, 4, 'ORD-2025-003', 'processing', 179.97, 14.40, 8.99, 203.36, 'paid'),
(4, 5, 5, 'ORD-2025-004', 'confirmed', 89.99, 7.20, 5.99, 103.18, 'paid'),
(5, 6, 6, 'ORD-2025-005', 'pending', 129.98, 10.40, 7.99, 148.37, 'pending');

-- Order items
INSERT INTO order_item(order_id, product_id, quantity, unit_price, total_price)
  VALUES
    -- Order 1: MacBook Pro + Mouse
(1, 1, 1, 2499.00, 2499.00),
(1, 5, 1, 79.99, 79.99),
    -- Order 2: Samsung Galaxy S24 + USB-C Hub
(2, 4, 1, 899.00, 899.00),
(2, 6, 1, 49.99, 49.99),
    -- Order 3: Men's T-Shirt + Programming Book + Mouse
(3, 7, 2, 29.99, 59.98),
(3, 9, 1, 49.99, 49.99),
(3, 5, 1, 79.99, 79.99),
    -- Order 4: Women's Jeans
(4, 8, 1, 89.99, 89.99),
    -- Order 5: Dell XPS 13 + USB-C Hub
(5, 3, 1, 1299.00, 1299.00),
(5, 6, 1, 49.99, 49.99);

-- Product reviews
INSERT INTO review(product_id, customer_id, rating, title, content, is_verified_purchase, is_approved)
  VALUES (1, 1, 5, 'Excellent laptop!', 'The MacBook Pro M2 is incredibly fast and the battery life is amazing. Perfect for development work.', 1, 1),
(1, 3, 4, 'Great performance', 'Very fast machine, though a bit pricey. The screen quality is outstanding.', 0, 1),
(4, 2, 5, 'Best Android phone', 'Samsung Galaxy S24 has an incredible camera and the performance is top-notch.', 1, 1),
(5, 1, 4, 'Good wireless mouse', 'Comfortable to use and good battery life. Could be a bit more ergonomic.', 1, 1),
(5, 3, 5, 'Perfect for productivity', 'This mouse is great for long coding sessions. Highly recommended!', 1, 1),
(9, 5, 5, 'Great Rust book', 'Comprehensive guide to Rust programming. Well written and easy to follow.', 0, 1);

-- Shopping cart items (current sessions)
INSERT INTO cart_item(customer_id, product_id, quantity)
  VALUES (2, 1, 1), -- Jane considering MacBook Pro
(2, 5, 1), -- Jane considering wireless mouse
(4, 10, 2), -- Sarah has mystery novels in cart
(4, 7, 1), -- Sarah considering t-shirt
(5, 6, 1);

-- David has USB-C hub in cart
