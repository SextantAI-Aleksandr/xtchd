CREATE TABLE IF NOT EXISTS content_classes (
	content_class VARCHAR(15) NOT NULL PRIMARY KEY
);
INSERT INTO content_classes (content_class) VALUES
('init') ('dev_test'),
('Article'), ('ArticlePara'), ('ArticleAddendum'), ('Author'), 
('Video'), ('TranscriptPara'), ('YoutubeChannel'),
('Image');



CREATE TABLE IF NOT EXISTS hash_integrity (
	prior_id INTEGER UNIQUE , --(prior_id = id - 1), -- ensure the prior_id is one less than the item id
	id INTEGER NOT NULL PRIMARY KEY,
    name_or_text VARCHAR,	-- overloaded with the title of the article / author etc. OR with the text of the paragraph/transcript
	content_class VARCHAR(15) NOT NULL,
	content_json VARCHAR NOT NULL,		-- XtchdContent serialized to JSON
    to_parent INTEGER,                 -- overloaded to be the article_id (for a paragraph), video_id (for transcript paragraph), or destination for a reference
    from_child INTEGER,                -- source id for a reference 
	uploaded_at TIMESTAMP NOT NULL,     -- local machine timestamp
	prior_sha256 CHAR(64) NOT NULL, -- included for checking integrity
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(id, new_sha256), -- this allows the hiprior constraint below 
	ts tsvector GENERATED ALWAYS AS ( to_tsvector('english', COALESCE(name_or_text, '') )) STORED,
CONSTRAINT hi_con_class FOREIGN KEY (content_class) REFERENCES content_classes (content_class) ON UPDATE CASCADE,
CONSTRAINT hi_prior_inc_one CHECK ( (id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = id - 1)) ),
-- this constraint means you can't delete a row (apart from the last one) without violating a FOREIGN KEY constraint
CONSTRAINT hi_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES hash_integrity (id, new_sha256),
-- the clever CHECK on uploaded_at ensures you can't "rewrite everything" each day with a new yet consistent history
CONSTRAINT hi_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - uploaded_at)) <= 1),
CONSTRAINT hi_verify_sha CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'id=', id::VARCHAR,
				' content_class=', content_class,
				' content_json=', content_json,
				' name_or_text=', name_or_text,
				' to_parent=', to_parent,
				' from_child=', from_child,
				' uploaded_at=', TO_CHAR(uploaded_at, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
); 
CREATE INDEX sch_fulltext ON hash_integrity USING GIN(ts);
CREATE INDEX idx_to_parent ON hash_integrity(to_parent, content_class);
CREATE INDEX idx_from_child ON hash_integrity(from_child, content_class);

INSERT INTO hash_integrity (id, content_class, name_or_text, content_json, uploaded_at, prior_sha256, new_sha256)
VALUES (0, 'init', 'Initialization', '{}', CURRENT_TIMESTAMP,
    '0000000000000000000000000000000000000000000000000000000000000000',
	 ENCODE( SHA256( CONCAT(
			'id=', 0::VARCHAR,
			' content_class=init'
			' content_json={}', 
			' name_or_text=Initialization',
			' to_parent=',
			' from_child=',
			' uploaded_at=', TO_CHAR(CURRENT_TIMESTAMP, 'YYYY.MM.DD HH24:MI:SS'),
			' prior_sha256=', '0000000000000000000000000000000000000000000000000000000000000000'
		)::BYTEA), 'hex')
);
