

CREATE VIEW author_detail AS (
    WITH authorship AS (
        SELECT auth_id, ARRAY_AGG(JSON_BUILD_OBJECT('id', art_id, 'name', title)) AS authored
        FROM articles GROUP BY auth_id
    ) SELECT au.prior_id, au.auth_id, au.name, au.prior_sha256, au.write_timestamp, au.new_sha256, authored
    FROM authorship
    INNER JOIN authors au ON authorship.auth_id = au.auth_id
);