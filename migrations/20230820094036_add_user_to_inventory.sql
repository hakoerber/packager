CREATE TABLE "inventory_items_tmp" (
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

CREATE TABLE "inventory_items_categories_tmp" (
    id VARCHAR(36) NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    user_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

INSERT INTO inventory_items_tmp SELECT *, (SELECT id FROM users LIMIT 1) as user_id FROM inventory_items;
INSERT INTO inventory_items_categories_tmp SELECT *, (SELECT id FROM users LIMIT 1) as user_id FROM inventory_items_categories;

DROP TABLE inventory_items;
DROP TABLE inventory_items_categories;

ALTER TABLE inventory_items_tmp RENAME TO inventory_items;
ALTER TABLE inventory_items_categories_tmp RENAME TO inventory_items_categories;
