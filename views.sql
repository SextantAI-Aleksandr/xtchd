

CREATE VIEW author_detail AS (
    -- this view yields the view.rs::AuthorDetail struct
    WITH authorship AS (
        SELECT auth_id, ARRAY_AGG(JSON_BUILD_OBJECT('id', art_id, 'name', title)) AS authored
        FROM articles GROUP BY auth_id
    ) SELECT au.prior_id, au.auth_id, au.name, au.prior_sha256, au.write_timestamp, au.new_sha256, authored
    FROM authorship
    INNER JOIN authors au ON authorship.auth_id = au.auth_id
);


CREATE VIEW article_text AS (
    -- this view yields the view.rs::ArticleText struct
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
        'content', JSON_BUILD_OBJECT('art_id', ar.art_id, 'auth_id', ar.auth_id, 'title', ar.title, 'image_file', am.image_file),
        'prior_sha256', ar.prior_sha256, 'write_timestamp', ar.write_timestamp, 'new_sha256', ar.new_sha256
    ) AS article, 
    p.art_paras
    FROM articles ar
    LEFT JOIN authors au ON ar.auth_id = au.auth_id
    LEFT JOIN paragraphs p ON ar.art_id = p.art_id
    LEFT JOIN article_mut am ON ar.art_id = am.art_id
);


CREATE VIEW article_refs AS (
    /* this view yields Vec<views::ArticleRef>, keyed by (art_id, apara_id)
    for the article from which the refence is being made. 
    NOTE: because apara_id can be NULL, NULL is replaced with -1 in this view */
    SELECT from_art AS art_id, COALESCE(from_para, -1) AS apara_id,
        ARRAY_AGG(JSON_BUILD_OBJECT(
            'aref_id', aref_id, 'art_id', refs_art, 'apara_id', refs_para,
            'title', title, 'comment',comment )) AS art_refs
    FROM article_ref_article ara
    INNER JOIN articles  
        ON ara.refs_art = articles.art_id
    GROUP BY (from_art, apara_id)
);


CREATE VIEW video_refs AS (
    /* this view yields Vec<views::VideoRef>, keyed by (art_id, apara_id)
    for the article from which the refence is being made. 
    NOTE: because apara_id can be NULL, NULL is replaced with -1 in this view */
    SELECT art_id, COALESCE(apara_id, -1) AS apara_id,
        ARRAY_AGG(JSON_BUILD_OBJECT(
            'vref_id', vref_id, 'vid_pk', arv.vid_pk, 'sec_req', sec_req,
            'title', title, 'comment',comment )) AS vid_refs
    FROM article_ref_video arv
    INNER JOIN youtube_videos yv
        ON arv.vid_pk = yv.vid_pk
    GROUP BY (art_id, apara_id)
);



CREATE VIEW image_refs AS (
    /* this view yields Vec<views::ImageRef> , keyed by (art_id, apara_id)
    for the article from which the refence is being made. 
    NOTE: because apara_id can be NULL, NULL is replaced with -1 in this view */
    SELECT art_id, COALESCE(apara_id, -1) AS apara_id,
        ARRAY_AGG(JSON_BUILD_OBJECT(
            'iref_id', iref_id, 'img_id', ari.img_id, 'src_thmb', src_thmb,
            'alt', alt, 'url', url, 'comment', comment )) AS img_refs
    FROM article_ref_image ari
    INNER JOIN images
        ON ari.img_id = images.img_id
    GROUP BY (art_id, apara_id)
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



CREATE VIEW combined_refs AS (
    /* this view yields Vec<views::References> keyed by (art_id, apara_id)*/
    WITH all_article_paragraphs AS (
        SELECT art_id, -1 AS apara_id FROM article_para -- for the article as a whole
        UNION
        SELECT art_id, apara_id FROM article_para
    )
    SELECT ap.art_id, ap.apara_id, 
        JSON_BUILD_OBJECT(
            'articles', CASE WHEN art_refs IS NULL THEN ARRAY[]::JSON[] else art_refs END,
            'videos',   CASE WHEN vid_refs IS NULL THEN ARRAY[]::JSON[] else vid_refs END,
            'images',   CASE WHEN img_refs IS NULL THEN ARRAY[]::JSON[] else img_refs END
        ) AS refs 
    FROM all_article_paragraphs ap 
    FULL OUTER JOIN  article_refs a 
        ON ap.art_id = a.art_id AND COALESCE(ap.apara_id, -1) = a.apara_id
    FULL OUTER JOIN video_refs v
        ON ap.art_id = v.art_id AND COALESCE(ap.apara_id, -1) = v.apara_id 
    FULL OUTER JOIN image_refs i 
        ON ap.art_id = i.art_id AND COALESCE(ap.apara_id, -1) = i.apara_id 
);


CREATE VIEW enriched_paragraphs AS (
    /*This view yields the views::EnrichedPara struct, keyed by (art_id, apara_id)*/
    SELECT ap.art_id, ap.apara_id, 
        JSON_BUILD_OBJECT(
            'para', JSON_BUILD_OBJECT('prior_id', ap.prior_id,
                'content', JSON_BUILD_OBJECT('art_id', ap.art_id, 'apara_id', ap.apara_id, 'md', md),
                'prior_sha256', ap.prior_sha256, 'write_timestamp', ap.write_timestamp, 'new_sha256', ap.new_sha256
                ), -- this is the integrity::XtchdSQL<views::
            'refs', cr.refs,
            'topics', CASE WHEN topics IS NULL THEN ARRAY[]::JSON[] else topics END
        ) as epara
    FROM article_para ap 
    LEFT JOIN combined_refs cr 
        ON ap.art_id = cr.art_id AND ap.apara_id = cr.apara_id 
    LEFT JOIN topic_refs tr 
        ON ap.art_id = tr.art_id AND ap.apara_id = tr.apara_id 
);


CREATE VIEW enriched_article_fields AS (
    -- this view yields the fields needed for views.rs::EnrichedArticle struct, keyed by art_id 
    WITH epara_agg AS (
        -- aggregated enriched paragraphs by article
        SELECT art_id, ARRAY_AGG(epara) AS paragraphs
        FROM enriched_paragraphs 
        GROUP BY art_id
    )
    SELECT art.art_id, epara_agg.paragraphs, cr.refs, -- cr.refs can be null due to LEFT JOIN if there are no -1 rows for that article
        JSON_BUILD_OBJECT('prior_id', au.prior_id,
            'content', JSON_BUILD_OBJECT('auth_id', au.auth_id, 'name', au.name),
            'prior_sha256', au.prior_sha256, 'write_timestamp', au.write_timestamp, 'new_sha256', au.new_sha256
        ) AS author, -- This JSON is for XtchdSQL<Author> which is converted to XtchdContent<Author> by the impl tokio_postgres::types::FromSql 
        JSON_BUILD_OBJECT('prior_id', art.prior_id,
            'content', JSON_BUILD_OBJECT('art_id', art.art_id, 'auth_id', art.auth_id, 'title', art.title, 'image_file', am.image_file),
            'prior_sha256', art.prior_sha256, 'write_timestamp', art.write_timestamp, 'new_sha256', art.new_sha256
        ) AS article -- This JSON is for XtchdSQL<Article> which is converted to XtchdContent<Article> by the impl tokio_postgres::types::FromSql 
    FROM articles art
    LEFT JOIN epara_agg ON art.art_id = epara_agg.art_id
    LEFT JOIN authors au ON art.auth_id = au.auth_id
    LEFT JOIN combined_refs cr ON art.art_id = cr.art_id AND cr.apara_id = -1
    LEFT JOIN article_mut am ON art.art_id = am.art_id
);