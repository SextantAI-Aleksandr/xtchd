

CREATE VIEW author_detail AS (
    WITH authorship AS (
        SELECT auth_id, ARRAY_AGG(JSON_BUILD_OBJECT('id', art_id, 'name', title)) AS authored
        FROM articles GROUP BY auth_id
    ) SELECT au.prior_id, au.auth_id, au.name, au.prior_sha256, au.write_timestamp, au.new_sha256, authored
    FROM authorship
    INNER JOIN authors au ON authorship.auth_id = au.auth_id
);


CREATE VIEW article_detail AS (
    WITH paragraphs AS (
        SELECT art_id, ARRAY_AGG(JSON_BUILD_OBJECT('prior_id', prior_id,
            'content', JSON_BUILD_OBJECT('art_id', art_id, 'apara_id', apara_id, 'md', md),
            'prior_sha256', prior_sha256, 'write_timestamp', write_timestamp, 'new_sha256', new_sha256
        )) AS art_paras
        FROM article_para GROUP BY art_id
    ) SELECT ar.art_id, 
    -- This JSON is for XtchdSQL<Author> which is converted to XtchdContent<Author> by the impl tokio_postgres::types::FromSql 
    JSON_BUILD_OBJECT('prior_id', au.prior_id,
        'content', JSON_BUILD_OBJECT('auth_id', au.auth_id, 'name', au.name),
        'prior_sha256', au.prior_sha256, 'write_timestamp', au.write_timestamp, 'new_sha256', au.new_sha256
    ) AS author, 
    -- This JSON is for XtchdSQL<Article> which is converted to XtchdContent<Article> by the impl tokio_postgres::types::FromSql 
    JSON_BUILD_OBJECT('prior_id', ar.prior_id,
        'content', JSON_BUILD_OBJECT('art_id', ar.art_id, 'auth_id', ar.auth_id, 'title', ar.title),
        'prior_sha256', ar.prior_sha256, 'write_timestamp', ar.write_timestamp, 'new_sha256', ar.new_sha256
    ) AS article, 
    p.art_paras
    FROM articles ar
    LEFT JOIN authors au ON ar.auth_id = au.auth_id
    LEFT JOIN paragraphs p ON ar.art_id = p.art_id
);