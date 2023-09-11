CREATE TABLE "trips_types_tmp" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    user_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name)
);

INSERT INTO trips_types_tmp SELECT *, (SELECT id FROM users LIMIT 1) as user_id FROM trips_types;

DROP TABLE trips_types;

ALTER TABLE trips_types_tmp RENAME TO trips_types;
