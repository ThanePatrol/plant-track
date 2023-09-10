CREATE TABLE IF NOT EXISTS plants (
	plant_id INTEGER PRIMARY KEY AUTOINCREMENT,
	botanical_name TEXT NOT NULL,
	last_fed TEXT -- ISO8601 strings, ie "YYYY-MM-DD HH:MM:SS.SSS"
	feed_interval INTEGER -- days until next feed
	last_potted TEXT -- iso timestamp again
	potting_interval INTEGER -- days until next potting
	notes TEXT -- comments/notes 
);

