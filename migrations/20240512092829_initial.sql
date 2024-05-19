CREATE TABLE IF NOT EXISTS "users" (
    id uuid NOT NULL,
    fullname TEXT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    PRIMARY KEY (id)
);

-- INVENTORY

CREATE TABLE IF NOT EXISTS "inventory_products" (
    id uuid NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    comment TEXT,
    PRIMARY KEY (id),
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS "inventory_items_categories" (
    id uuid NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    user_id uuid NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE TABLE IF NOT EXISTS "inventory_items" (
    id uuid NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    weight INTEGER NOT NULL,
    category_id uuid NOT NULL,
    product_id uuid,
    user_id uuid NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (category_id) REFERENCES inventory_items_categories(id),
    FOREIGN KEY (product_id) REFERENCES inventory_products(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- TRIPS

CREATE TABLE IF NOT EXISTS "trips" (
    id uuid NOT NULL,
    name TEXT NOT NULL,
    date_start DATE NOT NULL,
    date_end DATE NOT NULL,
    location TEXT,
    state VARCHAR(8) NOT NULL,
    comment TEXT,
    temp_min INTEGER,
    temp_max INTEGER,
    user_id uuid NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS "trips_types" (
    id uuid NOT NULL,
    name TEXT NOT NULL,
    user_id uuid NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS "trips_to_trips_types" (
    trip_id uuid NOT NULL,
    trip_type_id uuid NOT NULL,
    PRIMARY KEY (trip_id, trip_type_id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id),
    FOREIGN KEY(trip_type_id) REFERENCES "trips_types" (id)
);

CREATE TABLE IF NOT EXISTS "trip_todos" (
    id uuid NOT NULL,
    trip_id uuid NOT NULL,
    description TEXT NOT NULL,
    done BOOLEAN NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id)
);

CREATE TABLE IF NOT EXISTS "trips_items" (
    item_id uuid NOT NULL,
    trip_id uuid NOT NULL,
    pick BOOLEAN NOT NULL,
    pack BOOLEAN NOT NULL,
    ready BOOLEAN NOT NULL,
    new BOOLEAN NOT NULL,
    user_id uuid NOT NULL,
    PRIMARY KEY (item_id, trip_id),
    FOREIGN KEY(item_id) REFERENCES "inventory_items" (id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

