

CREATE VIEW author_detail AS (
    -- this view yields the view.rs::AuthorDetail struct
    WITH authorship AS (
        SELECT auth_id, ARRAY_AGG(JSON_BUILD_OBJECT('id', art_id, 'name', title)) AS authored
        FROM articles GROUP BY auth_id
    ) SELECT au.prior_id, au.auth_id, au.name, au.prior_sha256, au.write_timestamp, au.new_sha256, authored
    FROM authorship
    INNER JOIN authors au ON authorship.auth_id = au.auth_id
);



CREATE VIEW topic_refs AS (
    /* this view yields Vec<views::Topic> , keyed by (art_id, apara_id)
    for the paragraph where a topic is mentioned. */
    SELECT art_id, apara_id,
        ARRAY_AGG(JSON_BUILD_OBJECT(
            'tkey', amt.tkey, 'pos', pos,
            'name', name, 'count', count )) AS topics
    FROM apara_ment_topic amt
    INNER JOIN nlp_topics t
        ON amt.tkey = t.tkey
    GROUP BY (art_id, apara_id)
);

