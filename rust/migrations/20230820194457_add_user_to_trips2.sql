CREATE TABLE "trips_tmp" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    date_start DATE NOT NULL,
    date_end DATE NOT NULL,
    location TEXT,
    state VARCHAR(8) NOT NULL,
    comment TEXT,
    temp_min INTEGER,
    temp_max INTEGER,
    user_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

INSERT INTO trips_tmp SELECT *, (SELECT id FROM users LIMIT 1) as user_id FROM trips;

DROP TABLE trips;

ALTER TABLE trips_tmp RENAME TO trips;
