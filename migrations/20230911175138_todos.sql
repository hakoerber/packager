-- Add migration script here
CREATE TABLE "trip_todos" (
    id VARCHAR(36) NOT NULL,
    trip_id VARCHAR(36) NOT NULL,
    description TEXT NOT NULL,
    done BOOLEAN NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY(trip_id) REFERENCES "trips" (id)
)
