CREATE TABLE cake(
  id integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL,
  price real,
  is_vegan integer, -- BOOLEAN stored as INTEGER (0/1)
  created_at text, -- DATETIME stored as TEXT (ISO8601)
  description text,
  image BLOB,
  rating numeric
);

CREATE TABLE filling(
  id integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL,
  calories integer,
  fat real,
  notes text,
  nutrition_info BLOB
);

CREATE TABLE fruit(
  id integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  name text NOT NULL,
  cake_id integer NOT NULL REFERENCES cake(id) ON DELETE CASCADE,
  weight real,
  is_fresh integer, -- BOOLEAN stored as INTEGER (0/1)
  picked_at text, -- DATETIME stored as TEXT (ISO8601)
  color text
);

CREATE TABLE cake_filling(
  cake_id integer NOT NULL REFERENCES cake(id) ON DELETE CASCADE,
  filling_id integer NOT NULL REFERENCES filling(id) ON DELETE CASCADE,
  amount integer,
  notes text,
  PRIMARY KEY (cake_id, filling_id)
);

-- Insert sample data
-- Insert cakes
INSERT INTO cake(name, price, is_vegan, created_at, description, rating)
  VALUES ('Chocolate Fudge Cake', 25.99, 0, '2025-07-30T10:00:00Z', 'Rich chocolate cake with fudge frosting', 4.8),
('Vanilla Bean Delight', 22.50, 0, '2025-07-30T11:00:00Z', 'Classic vanilla cake with cream frosting', 4.5),
('Vegan Carrot Cake', 28.00, 1, '2025-07-30T12:00:00Z', 'Moist carrot cake made with plant-based ingredients', 4.7),
('Red Velvet Supreme', 30.00, 0, '2025-07-30T13:00:00Z', 'Traditional red velvet with cream cheese frosting', 4.9),
('Lemon Zest Cake', 24.75, 1, '2025-07-30T14:00:00Z', 'Fresh lemon cake with citrus glaze', 4.6);

-- Insert fillings
INSERT INTO filling(name, calories, fat, notes)
  VALUES ('Chocolate Ganache', 180, 12.5, 'Rich dark chocolate filling'),
('Vanilla Custard', 120, 8.0, 'Smooth vanilla cream filling'),
('Strawberry Jam', 80, 0.2, 'Fresh strawberry preserve'),
('Cream Cheese', 140, 11.0, 'Tangy cream cheese filling'),
('Caramel Sauce', 160, 6.5, 'Sweet buttery caramel'),
('Raspberry Compote', 70, 0.1, 'Fresh raspberry fruit filling');

-- Insert fruits
INSERT INTO fruit(name, cake_id, weight, is_fresh, picked_at, color)
  VALUES ('Strawberries', 1, 150.5, 1, '2025-07-29T08:00:00Z', 'Red'),
('Blueberries', 2, 100.0, 1, '2025-07-29T09:00:00Z', 'Blue'),
('Raspberries', 3, 80.3, 1, '2025-07-29T10:00:00Z', 'Red'),
('Blackberries', 4, 120.7, 1, '2025-07-29T11:00:00Z', 'Purple'),
('Cherries', 5, 200.2, 1, '2025-07-29T12:00:00Z', 'Dark Red'),
('Peaches', 1, 250.0, 1, '2025-07-29T13:00:00Z', 'Orange'),
('Apples', 2, 180.5, 0, '2025-07-28T15:00:00Z', 'Green');

-- Insert cake-filling relationships
INSERT INTO cake_filling(cake_id, filling_id, amount, notes)
  VALUES (1, 1, 200, 'Primary filling for chocolate cake'),
(1, 5, 50, 'Drizzle of caramel on top'),
(2, 2, 180, 'Main vanilla custard layer'),
(2, 3, 100, 'Strawberry jam between layers'),
(3, 4, 150, 'Cream cheese frosting'),
(3, 6, 80, 'Raspberry compote swirl'),
(4, 4, 200, 'Classic cream cheese for red velvet'),
(5, 2, 120, 'Light vanilla custard'),
(5, 3, 60, 'Strawberry accent');

