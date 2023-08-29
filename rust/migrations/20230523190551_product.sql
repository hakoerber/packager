CREATE TABLE "inventory_products" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    comment TEXT,
    PRIMARY KEY (id),
    UNIQUE (name)
);

CREATE TABLE "inventory_items_tmp" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    weight INTEGER NOT NULL,
    category_id VARCHAR(36) NOT NULL,
    product_id VARCHAR(36),
    PRIMARY KEY (id),
    FOREIGN KEY (category_id) REFERENCES inventory_items_categories(id)
    FOREIGN KEY (product_id) REFERENCES inventory_products(id)
);

INSERT INTO inventory_items_tmp SELECT *, NULL as product_id FROM inventory_items;

/* DROP TABLE inventory_items; */

/* ALTER TABLE "inventory_items_tmp" RENAME TO inventory_items; */

