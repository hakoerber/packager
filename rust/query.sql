SELECT
    i_item.id AS item_id
FROM inventory_items AS i_item
    LEFT JOIN (
        SELECT t_item.item_id as item_id
        FROM trips_items AS t_item
        WHERE t_item.trip_id = '2be5c6b9-9a46-4c90-b17a-87b2ede66163'
    ) AS t_item
    ON t_item.item_id = i_item.id
WHERE t_item.item_id IS NULL
