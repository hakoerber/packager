CREATE TABLE "inventory_items" (
  id TEXT,
  name TEXT,
  description TEXT,
  weight INT,
category_id TEXT,
FOREIGN KEY (category_id) REFERENCES inventory_items_categories(id));
CREATE UNIQUE INDEX ux_unique ON inventory_items(name, category_id);

CREATE TABLE "inventory_items_categories" (
	id VARCHAR(36) NOT NULL,
	name TEXT NOT NULL,
	description TEXT,
	PRIMARY KEY (id),
	UNIQUE (name)
);

CREATE TABLE "trips" (
	id VARCHAR(36) NOT NULL,
	name TEXT NOT NULL,
	date_start DATE NOT NULL,
	date_end DATE NOT NULL,
    location TEXT,
    state VARCHAR(8) NOT NULL DEFAULT "Planning",
    comment TEXT,
    temp_min INTEGER,
    temp_max INTEGER,
	PRIMARY KEY (id),
	UNIQUE (name)
);


CREATE TABLE "trips_types" (
	id VARCHAR(36) NOT NULL,
	name TEXT NOT NULL,
	PRIMARY KEY (id),
	UNIQUE (name)
);

CREATE TABLE "trips_to_trips_types" (
	trip_id VARCHAR(36) NOT NULL,
	trip_type_id VARCHAR(36) NOT NULL,
	PRIMARY KEY (trip_id, trip_type_id),
	FOREIGN KEY(trip_id) REFERENCES "trips" (id),
	FOREIGN KEY(trip_type_id) REFERENCES "trips_types" (id)
);


CREATE TABLE trips_items (
	item_id VARCHAR(36) NOT NULL,
	trip_id VARCHAR(36) NOT NULL,
	pick BOOLEAN NOT NULL,
	pack BOOLEAN NOT NULL,
	PRIMARY KEY (item_id, trip_id),
	FOREIGN KEY(item_id) REFERENCES "inventory_items" (id),
	FOREIGN KEY(trip_id) REFERENCES "trips" (id)
);
