-- CREATE TABLE "trips_tmp" (
--     id VARCHAR(36) NOT NULL,
--     name TEXT NOT NULL,
--     date_start DATE NOT NULL,
--     date_end DATE NOT NULL,
--     location TEXT,
--     state VARCHAR(8) NOT NULL,
--     user_id VARCHAR(36) NOT NULL,
--     comment TEXT,
--     temp_min INTEGER,
--     temp_max INTEGER,
--     PRIMARY KEY (id),
--     FOREIGN KEY (user_id) REFERENCES users(id),
--     UNIQUE (name)
-- );

CREATE TABLE "trips_items_tmp" (
    item_id VARCHAR(36) NOT NULL,
    trip_id VARCHAR(36) NOT NULL,
    pick BOOLEAN NOT NULL,
    pack BOOLEAN NOT NULL,
    ready BOOLEAN NOT NULL,
    new BOOLEAN NOT NULL,
    user_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (item_id, trip_id),
    FOREIGN KEY(item_id) REFERENCES "inventory_items" (id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- INSERT INTO trips_tmp SELECT *, (SELECT id FROM users LIMIT 1) as user_id FROM trips;
INSERT INTO trips_items_tmp SELECT *, (SELECT id FROM users LIMIT 1) as user_id FROM trips_items;

-- DROP TABLE trips;
DROP TABLE trips_items;

-- ALTER TABLE trips_tmp RENAME TO trips;
ALTER TABLE trips_items_tmp RENAME TO trips_items;
