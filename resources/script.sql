
CREATE TABLE IF NOT EXISTS users (
	user_id SERIAL PRIMARY KEY,
	first_name TEXT NOT NULL,
	last_name TEXT NOT NULL,
	email TEXT NOT NULL,
	phone TEXT -- potentially for phone alerts? todo later
);

CREATE TABLE IF NOT EXISTS plants (
	plant_id SERIAL PRIMARY KEY,
	user_id INTEGER REFERENCES users(user_id) NOT NULL,
	botanical_name TEXT NOT NULL,
	common_name TEXT NOT NULL,
	last_fed DATE NOT NULL, 
	feed_interval INTEGER NOT NULL, -- days until next feed
	last_potted DATE NOT NULL,
	potting_interval INTEGER NOT NULL, -- days until next potting
	last_pruned DATE NOT NULL,
	pruning_interval INTEGER NOT NULL --days until next pruning
);
-- represents a single comment
CREATE TABLE IF NOT EXISTS comments (
	plant_id INTEGER REFERENCES plants(plant_id) NOT NULL,
	user_id INTEGER REFERENCES users(user_id) NOT NULL,
	time_made TIMESTAMP NOT NULL,
	comment TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS photos (
	plant_id INTEGER REFERENCES plants(plant_id) NOT NULL,
	user_id INTEGER REFERENCES users(user_id) NOT NULL,
	photo_uri TEXT NOT NULL
);

--chatGPT generated dummy data

-- Inserting users
INSERT INTO users (first_name, last_name, email, phone)
VALUES
  ('Alice', 'Johnson', 'alice.johnson@example.com', '123-456-7890'),
  ('Bob', 'Smith', 'bob.smith@example.com', '987-654-3210');

-- Inserting plants
INSERT INTO plants (user_id, botanical_name, common_name, last_fed, feed_interval, last_potted, potting_interval, last_pruned, pruning_interval)
VALUES
  (1, 'Ficus lyrata', 'Fiddle leaf', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (2, 'Monstera deliciosa', 'Swiss Cheese', '2023-09-01', 30, '2023-03-01', 180, '2023-08-01', 90),
  (1, 'Sanseveria trifasciata', 'Snake Plant', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Epiprenum aureum', 'Pothos', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Musa spp.', 'Banana', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Calathea spp.', 'Calathea', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Asplenium nidus', 'Birds Nest Fern', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60);

-- Inserting a comment
INSERT INTO comments (plant_id, user_id, time_made, comment)
VALUES
  (1, 2, '2023-09-11 10:30:00', 'This plant looks very healthy.');

-- Inserting a photo
INSERT INTO photos (plant_id, user_id, photo_uri)
VALUES
  (2, 1, 'https://example.com/photos/monstera1.jpg');


-- Inserting another user
INSERT INTO users (first_name, last_name, email, phone)
VALUES
  ('Charlie', 'Davis', 'charlie.davis@example.com', '555-444-3333');

-- Inserting another plant
INSERT INTO plants (user_id, botanical_name, common_name, last_fed, feed_interval, last_potted, potting_interval, last_pruned, pruning_interval)
VALUES
  (3, 'Kalanchoe tomentosa', 'Succulent', '2023-08-15', 40, '2023-01-10', 365, '2023-06-20', 120);

-- Inserting another comment
INSERT INTO comments (plant_id, user_id, time_made, comment)
VALUES
  (3, 3, '2023-09-11 11:00:00', 'Need to remember to water this less frequently.');

-- Inserting another photo
INSERT INTO photos (plant_id, user_id, photo_uri)
VALUES
  (3, 3, 'https://example.com/photos/succulent1.jpg');

-- Inserting another photo
INSERT INTO photos (plant_id, user_id, photo_uri)
VALUES
  (1, 3, 'https://example.com/photos/ficus2.jpg');


-- Inserting plants
INSERT INTO plants (user_id, botanical_name, common_name, last_fed, feed_interval, last_potted, potting_interval, last_pruned, pruning_interval)
VALUES
  (1, 'Sanseveria trifasciata', 'Snake Plant', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Epiprenum aureum', 'Pothos', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Musa spp.', 'Banana', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Calathea spp.', 'Calathea', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Asplenium nidus', 'Birds Nest Fern', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Hoya carnosa', 'Hoya', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Zamioculcas zamiifolia', 'Zanzibar Gem', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60),
  (1, 'Ctenanthe setosa "Grey Star"', 'Grey Star Ctenanthe', '2023-08-01', 30, '2023-01-01', 180, '2023-07-01', 60);

UPDATE plants
SET last_fed = '2022-01-01'
WHERE plant_id = 1;



