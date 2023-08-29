CREATE TABLE trips_items_tmp (
    item_id VARCHAR(36) NOT NULL,
    trip_id VARCHAR(36) NOT NULL,
    pick BOOLEAN NOT NULL,
    pack BOOLEAN NOT NULL,
    ready BOOLEAN NOT NULL,
    new BOOLEAN NOT NULL,
    PRIMARY KEY (item_id, trip_id),
    FOREIGN KEY(item_id) REFERENCES "inventory_items" (id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id)
);

INSERT INTO trips_items_tmp SELECT *, true as ready FROM trips_items;

DROP TABLE trips_items;

ALTER TABLE "trips_items_tmp" RENAME TO trips_items;
