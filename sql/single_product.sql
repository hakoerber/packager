WITH one AS (
    SELECT
        product.id AS id,
        product.user_id AS user_id,
        product.name AS name,
        product.description AS description,
        product.price AS price,
        product.bought_at AS bought_at,
        product.bought_from AS bought_from,
        array_agg(link.id) AS link_ids,
        array_agg(link.name) AS link_names,
        array_agg(link.url) AS link_urls
    FROM
        products AS product
        LEFT JOIN product_links AS link ON link.product_id = product.id
    GROUP BY
        product.id
),
two AS (
    SELECT
            product.id AS id,
        array_agg(comment.id) AS comment_ids,
        array_agg(comment.content) AS comment_contents,
        array_agg(comment.date) AS comment_dates
    FROM
        products AS product
        LEFT JOIN product_comments AS comment ON comment.product_id = product.id
    GROUP BY
        product.id
),
product AS (
    SELECT
        one.id AS id,
        one.user_id AS user_id,
        one.name AS name,
        one.description AS description,
        one.price AS price,
        one.bought_at AS bought_at,
        one.bought_from AS bought_from,
        one.link_ids AS link_ids,
        one.link_names AS link_names,
        one.link_urls AS link_urls,
        two.comment_ids AS comment_ids,
        two.comment_contents AS comment_contents,
        two.comment_dates AS comment_dates
    FROM
        one
        INNER JOIN two ON one.id = two.id
)
SELECT
    product.id AS id,
    product.name AS name,
    product.description AS description,
    product.price AS price,
    product.bought_at AS purchase_date,
    product.bought_from AS purchase_from,
    product.link_ids AS "link_ids!",
    product.link_names AS "link_names!",
    product.link_urls AS "link_urls!",
    product.comment_ids AS "comment_ids!",
    product.comment_contents AS "comment_contents!",
    product.comment_dates AS "comment_dates!"
FROM
    product
WHERE
    product.id = $1
    AND product.user_id = $2
