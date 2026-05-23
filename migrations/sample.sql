DROP TABLE IF EXISTS posts;

CREATE TABLE posts (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL
) STRICT;

INSERT INTO posts (id, title, content) VALUES
    (1, 'Velvet Lantern', 'Quiet wind over stone.'),
    (2, 'Crimson Orbit', 'Coffee spills at dawn.'),
    (3, 'Mossy Circuit', 'Small gears hum softly.'),
    (4, 'Neon Thistle', 'Rain taps the window.'),
    (5, 'Copper Sparrow', 'Pages rustle in shade.'),
    (6, 'Amber Quarry', 'Distant trains fade out.'),
    (7, 'Silver Cactus', 'Warm dust on boots.'),
    (8, 'Indigo Harbor', 'Lantern light drifts home.'),
    (9, 'Obsidian Kite', 'Night birds cross clouds.'),
    (10, 'Golden Pebble', 'Footsteps echo, then stop.');
