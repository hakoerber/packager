CREATE TABLE "users" (
    id VARCHAR(36) NOT NULL,
    fullname TEXT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    PRIMARY KEY (id)
);
