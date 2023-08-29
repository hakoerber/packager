/* SELECT */
/*     t_item.item_id AS id, */
/*     t_item.pick AS picked, */
/*     t_item.pack AS packed, */
/*     t_item.new AS new, */
/*     i_item.name AS name, */
/*     i_item.description AS description, */
/*     i_item.weight AS weight, */
/*     i_item.category_id AS category_id */
/* FROM trips_items AS t_item */
/* INNER JOIN inventory_items AS i_item */
/*     ON i_item.id = t_item.item_id */
/* WHERE t_item.item_id = '7f492a29-5bc9-4e20-b4cf-445c5ac444fc' */
/* AND t_item.trip_id = '0535193c-7b47-4ba4-bca5-40e54c15c2d0'; */

/* SELECT */
/*     COALESCE(MAX(i_item.weight), 0) AS weight, */
/*     COUNT(i_item.weight) AS found, */
/*     IFNULL(i_item.weight, 'IT IS NULL') AS found2 */
/* FROM inventory_items_categories as category */
/* INNER JOIN inventory_items as i_item */
/*     ON i_item.category_id = category.id */
/* WHERE category_id = ( */
/*     SELECT category_id */
/*     FROM inventory_items */
/*     /1* WHERE inventory_items.id = '7f492a29-5bc9-4e20-b4cf-445c5ac444fc' *1/ */
/*     WHERE inventory_items.id = '69147a37-cc4e-416b-b8d5-d65017f12184' */
/* ) */

/* SELECT */
/*     category.id as category_id, */
/*     category.name as category_name, */
/*     category.description AS category_description, */
/*     inner.trip_id AS trip_id, */
/*     inner.item_id AS item_id, */
/*     inner.item_name AS item_name, */
/*     inner.item_description AS item_description, */
/*     inner.item_weight AS item_weight, */
/*     inner.item_is_picked AS item_is_picked, */
/*     inner.item_is_packed AS item_is_packed, */
/*     inner.item_is_new AS item_is_new */
/* FROM inventory_items_categories AS category */
/*     LEFT JOIN ( */
/*         SELECT */
/*             trip.trip_id AS trip_id, */
/*             category.id as category_id, */
/*             category.name as category_name, */
/*             category.description as category_description, */
/*             item.id as item_id, */
/*             item.name as item_name, */
/*             item.description as item_description, */
/*             item.weight as item_weight, */
/*             trip.pick as item_is_picked, */
/*             trip.pack as item_is_packed, */
/*             trip.new as item_is_new */
/*         FROM trips_items as trip */
/*         INNER JOIN inventory_items as item */
/*             ON item.id = trip.item_id */
/*         INNER JOIN inventory_items_categories as category */
/*             ON category.id = item.category_id */
/*         WHERE trip.trip_id = '0535193c-7b47-4ba4-bca5-40e54c15c2d0' */
/*     ) AS inner */
/*     ON inner.category_id = category.id */
/* WHERE category.id = '1293c6b6-eef5-4269-bf10-a1ac20549dac' */

SELECT
    trip.id AS id,
    trip.name AS name,
    CAST (date_start AS TEXT) date_start,
    CAST (date_end AS TEXT) date_end,
    state,
    location,
    temp_min,
    temp_max,
    comment,
    SUM(i_item.weight) AS total_weight
FROM trips AS trip
INNER JOIN trips_items AS t_item
    ON t_item.trip_id = trip.id
INNER JOIN inventory_items AS i_item
    ON t_item.item_id = i_item.id
WHERE
    trip.id = '0535193c-7b47-4ba4-bca5-40e54c15c2d0'
    AND t_item.pick = true
