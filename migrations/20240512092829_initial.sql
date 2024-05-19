CREATE TABLE IF NOT EXISTS "users" (
    id VARCHAR(36) NOT NULL,
    fullname TEXT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    PRIMARY KEY (id)
);

-- INVENTORY

CREATE TABLE IF NOT EXISTS "inventory_products" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    comment TEXT,
    PRIMARY KEY (id),
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS "inventory_items_categories" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    user_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE TABLE IF NOT EXISTS "inventory_items" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    weight INTEGER NOT NULL,
    category_id VARCHAR(36) NOT NULL,
    product_id VARCHAR(36),
    user_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (category_id) REFERENCES inventory_items_categories(id),
    FOREIGN KEY (product_id) REFERENCES inventory_products(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- TRIPS

CREATE TABLE IF NOT EXISTS "trips" (
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

CREATE TABLE IF NOT EXISTS "trips_types" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    user_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS "trips_to_trips_types" (
    trip_id VARCHAR(36) NOT NULL,
    trip_type_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (trip_id, trip_type_id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id),
    FOREIGN KEY(trip_type_id) REFERENCES "trips_types" (id)
);

CREATE TABLE IF NOT EXISTS "trip_todos" (
    id VARCHAR(36) NOT NULL,
    trip_id VARCHAR(36) NOT NULL,
    description TEXT NOT NULL,
    done BOOLEAN NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id)
);

CREATE TABLE IF NOT EXISTS "trips_items" (
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

