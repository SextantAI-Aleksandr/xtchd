

CREATE TABLE IF NOT EXISTS authors (
	prior_id INTEGER UNIQUE,
	auth_id INTEGER NOT NULL PRIMARY KEY,
	name VARCHAR NOT NULL UNIQUE,
	prior_sha256 CHAR(64) NOT NULL, -- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(auth_id, new_sha256), -- this allows the no_delete constraint below 
	ac tsvector GENERATED ALWAYS AS ( to_tsvector('simple', name )) STORED,
CONSTRAINT auth_prior CHECK ( (auth_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = auth_id - 1)) ),
CONSTRAINT auth_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES authors (auth_id, new_sha256),
CONSTRAINT auth_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
CONSTRAINT auth_verify_sha256 CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'auth_id=', auth_id::VARCHAR,
				' name=', name,
				' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
);
INSERT INTO authors (auth_id, name, prior_sha256, write_timestamp, new_sha256) VALUES
(0, 'Xtchd Admins', '0000000000000000000000000000000000000000000000000000000000000000',
	CURRENT_TIMESTAMP,
	ENCODE(
		SHA256(
			CONCAT(
				'auth_id=', 0::VARCHAR,
				' name=Xtchd Admins',
				' write_timestamp=', TO_CHAR(CURRENT_TIMESTAMP, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=0000000000000000000000000000000000000000000000000000000000000000'
			)::BYTEA
		),
	'hex')
);


CREATE TABLE IF NOT EXISTS image_files (
	-- filenames for non-hashed images (typically stock photos) stored at the /images/ path in 
	image_file VARCHAR NOT NULL PRIMARY KEY, 	-- the filename for the image 
	license_info VARCHAR,						-- the text to display next to the image
	alt VARCHAR									-- alternate text for accessability 
);

CREATE TABLE IF NOT EXISTS articles (
	prior_id INTEGER UNIQUE,
	art_id INTEGER NOT NULL PRIMARY KEY,
	auth_id INTEGER NOT NULL,
	title VARCHAR NOT NULL UNIQUE,
	prior_sha256 CHAR(64) NOT NULL, -- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(art_id, new_sha256), -- this allows the no_delete constraint below 
	ac tsvector GENERATED ALWAYS AS ( to_tsvector('simple', title )) STORED,
CONSTRAINT art_auth FOREIGN KEY (auth_id) REFERENCES authors(auth_id),
CONSTRAINT art_image FOREIGN KEY (image_file) REFERENCES image_files(image_file) ON UPDATE CASCADE,
CONSTRAINT art_prior CHECK ( (art_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = art_id - 1)) ),
CONSTRAINT art_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES articles (art_id, new_sha256),
CONSTRAINT art_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
CONSTRAINT art_verify_sha256 CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'art_id=', art_id::VARCHAR,
				' auth_id=', auth_id::VARCHAR,
				' title=', title,
				' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
);


CREATE TABLE IF NOT EXISTS article_mut (
	/* Properties of an article that can be changed, such as the cover image,
	are kept separately in this "mutable" table so they don't trigger an attempted rewrite if changed*/
	art_id INTEGER NOT NULL PRIMARY KEY,
	image_file VARCHAR,			-- an image to use as the 'cover' for an article
CONSTRAINT amuta FOREIGN KEY (art_id) REFERENCES articles(art_id)
);


CREATE TABLE IF NOT EXISTS article_para (
	prior_id INTEGER UNIQUE,
	apara_id INTEGER NOT NULL UNIQUE,
	art_id INTEGER NOT NULL,
	md VARCHAR NOT NULL,		-- Markdown for this paragarph as provided by the author
	prior_sha256 CHAR(64) NOT NULL, -- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	PRIMARY KEY (art_id, apara_id),
	UNIQUE(apara_id, new_sha256), -- this allows the no_delete constraint below 
	ts tsvector GENERATED ALWAYS AS ( to_tsvector('english', md)) STORED,
CONSTRAINT apapa_art FOREIGN KEY (art_id) REFERENCES articles(art_id),
CONSTRAINT apara_prior CHECK ( (apara_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = apara_id - 1)) ),
CONSTRAINT apara_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES article_para (apara_id, new_sha256),
CONSTRAINT apara_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
CONSTRAINT apara_verify_sha256 CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'apara_id=', apara_id::VARCHAR,
				' art_id=', art_id::VARCHAR,
				' md=', md,
				' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
);
CREATE INDEX article_fulltext ON article_para USING GIN(ts);
INSERT INTO article_para (apara_id, art_id, md, prior_sha256, write_timestamp, new_sha256) 
/*VALUES (0, 0, '*First Paragraph* of [Initial Article](www.xtchd.com) markdown!',
	'0000000000000000000000000000000000000000000000000000000000000000',
	CURRENT_TIMESTAMP, 
	ENCODE(
		SHA256(
			CONCAT(
				'apara_id=', 0::VARCHAR,
				' art_id=', 0::VARCHAR,
				' md=', '*First Paragraph* of [Initial Article](www.xtchd.com) markdown!',
				' write_timestamp=', TO_CHAR(CURRENT_TIMESTAMP, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=','0000000000000000000000000000000000000000000000000000000000000000'
			)::BYTEA
		),
	'hex')
);*/



CREATE TABLE IF NOT EXISTS images_mut (
	/*This table stores images that are mutable- i.e. the rows can be deleted or changed
	This is typically used for article thumbnails, where the choice of image is a bit arbitrary
	and is not the main content of the article.*/
	id CHAR(16) NOT NULL PRIMARY KEY,	-- nanoID 
	src_full TEXT NOT NULL,				 		-- image source encoded as base64: "<img src="data:image/png;base64, iVBORw0KGgoA..." etc
	src_thmb TEXT NOT NULL,						-- thumbnail source encoded as base64: "<img src="data:image/png;base64, iVBORw0KGgoA..." etc
	alt VARCHAR NOT NULL,						-- caption / alt text for accessability
	url VARCHAR,								-- url for a screenshot or image download 
	ts tsvector GENERATED ALWAYS AS ( to_tsvector('english', alt )) STORED
);



CREATE TABLE IF NOT EXISTS images (
	/*This table stores images encoded as base64. 
	Many of them may be screenshots: the url field captures the source in those cases*/
	prior_id INTEGER,							-- id of the prior image
	img_id INTEGER NOT NULL PRIMARY KEY,		-- id for this image
	src_full TEXT NOT NULL,						-- image source encoded as base64: "<img src="data:image/png;base64, iVBORw0KGgoA..." etc
	src_thmb TEXT NOT NULL,						-- thumbnail source encoded as base64: "<img src="data:image/png;base64, iVBORw0KGgoA..." etc
	alt VARCHAR NOT NULL,						-- caption / alt text for accessability
	url VARCHAR,								-- url for a screenshot or image download 
	archive CHAR(5),							-- 5-character key for https://archive.is/83cXk and https://archive.is/83cXk/image etc.
	prior_sha256 CHAR(64) NOT NULL, 			-- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     	-- timestamp when this row was written 
	new_sha256 CHAR(64) NOT NULL,				-- new sha256 based on the below constraint
	UNIQUE(img_id, new_sha256),				-- this allows the below constraint 
	ts tsvector GENERATED ALWAYS AS ( to_tsvector('english', alt || ' ' || archive )) STORED,
	CONSTRAINT img_prior CHECK ( (img_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = img_id - 1)) ),
	CONSTRAINT img_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES images (img_id, new_sha256),
	CONSTRAINT img_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
	CONSTRAINT img_verify_sha256 CHECK (
		ENCODE(
			SHA256(
				CONCAT(
					'img_id=', img_id::VARCHAR,
					' src_full=', src_full,
					' src_thmb=', src_thmb,
					' alt=', alt,
					' url=', url,
					' archive=', archive,
					' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
					' prior_sha256=', prior_sha256
				)::BYTEA
			),
	'hex') = new_sha256)
);
CREATE INDEX image_ts ON images USING GIN(ts);



CREATE TABLE IF NOT EXISTS youtube_channels (
	prior_id INTEGER UNIQUE,
	chan_id INTEGER NOT NULL PRIMARY KEY,
	url VARCHAR NOT NULL UNIQUE, -- 'ChannelName' for youtube.com/@ChannelName 
	name VARCHAR NOT NULL UNIQUE,
	prior_sha256 CHAR(64) NOT NULL, -- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(chan_id, new_sha256),
	ac tsvector GENERATED ALWAYS AS ( to_tsvector('simple', name )) STORED,
	CONSTRAINT ytchan_prior CHECK ( (chan_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = chan_id - 1)) ),
	CONSTRAINT ytchan_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES youtube_channels (chan_id, new_sha256),
	CONSTRAINT ytchan_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
	CONSTRAINT ytchan_verify_sha256 CHECK (
		ENCODE(
			SHA256(
				CONCAT(
					'chan_id=', chan_id::VARCHAR,
					' name=', name,
					' url=', url,
					' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
					' prior_sha256=', prior_sha256
				)::BYTEA
			),
	'hex') = new_sha256)
);
CREATE INDEX ytchan_autocomp ON youtube_channels USING GIN(ac);
INSERT INTO youtube_channels (chan_id, url, name, write_timestamp, prior_sha256, new_sha256) 
VALUES (0, 'SextantAI', 'SextantAI', CURRENT_TIMESTAMP, '0000000000000000000000000000000000000000000000000000000000000000',
	ENCODE(
            SHA256(
                CONCAT(
                    'chan_id=', 0::VARCHAR,
                    ' name=', 'SextantAI',
                    ' url=', 'SextantAI',
                    ' write_timestamp=', TO_CHAR(CURRENT_TIMESTAMP, 'YYYY.MM.DD HH24:MI:SS'),
                    ' prior_sha256=', '0000000000000000000000000000000000000000000000000000000000000000'
                )::BYTEA
            ),
    'hex')
);


CREATE TABLE IF NOT EXISTS youtube_videos (
	prior_id INTEGER UNIQUE,
	vid_id INTEGER NOT NULL UNIQUE,
	vid_pk CHAR(11) NOT NULL PRIMARY KEY, 	-- this is the key assigned by Youtube
	chan_id INTEGER NOT NULL,
	title VARCHAR NOT NULL,
	date_uploaded DATE NOT NULL, -- date the video was loaded to youtube 
	prior_sha256 CHAR(64) NOT NULL, 
	write_timestamp TIMESTAMPTZ NOT NULL,
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(vid_id, new_sha256),
	ac tsvector GENERATED ALWAYS AS ( to_tsvector('simple', title || ' '|| vid_pk)) STORED,
	CONSTRAINT ytvid_chan FOREIGN KEY (chan_id) REFERENCES youtube_channels (chan_id),
	CONSTRAINT ytchan_prior CHECK ( (vid_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = vid_id - 1)) ),
	CONSTRAINT ytvid_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES youtube_videos (vid_id, new_sha256),
	CONSTRAINT ytvid_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
	CONSTRAINT ytvid_verify_sha256 CHECK (
		ENCODE(
			SHA256(
				CONCAT(
					'vid_id=', vid_id::VARCHAR,
					' vid_pk=', vid_pk,
					' chan_id=', chan_id::VARCHAR,
					' title=', title,
					' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
					' prior_sha256=', prior_sha256
				)::BYTEA
			),
	'hex') = new_sha256)
);
CREATE INDEX ytvid_autocomp ON youtube_videos USING GIN(ac);




CREATE TABLE IF NOT EXISTS article_ref_article (
	/*When an article (or a paragraph in an article) references another article (with optional paragraph),
	a row is added to capture that reference */
	prior_id INTEGER UNIQUE,
	aref_id INTEGER NOT NULL PRIMARY KEY,
	from_art INTEGER NOT NULL,						-- the article making the reference
	from_para INTEGER,								-- (optional) paragraph within the article 
	refs_art INTEGER NOT NULL,						-- the article being referenced
	refs_para INTEGER,								-- (optional) paragraph within the article being referenced 
	comment VARCHAR NOT NULL,						-- a brief explanation of why this is relevant or what it shows
	prior_sha256 CHAR(64) NOT NULL, 				-- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(aref_id, new_sha256), 					-- this allows the no_delete constraint below 
	ts tsvector GENERATED ALWAYS AS ( to_tsvector('english', comment)) STORED,
CONSTRAINT arafa FOREIGN KEY (from_art) REFERENCES articles(art_id),
CONSTRAINT arafp FOREIGN KEY (from_art, from_para) REFERENCES article_para (art_id, apara_id),
CONSTRAINT arata FOREIGN KEY (refs_art) REFERENCES articles(art_id),
CONSTRAINT aratp FOREIGN KEY (refs_art, refs_para) REFERENCES article_para (art_id, apara_id),
CONSTRAINT ara_prior CHECK ( (aref_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = aref_id - 1)) ),
CONSTRAINT ara_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES article_ref_article (aref_id, new_sha256),
CONSTRAINT ara_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
CONSTRAINT ara_verify_sha256 CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'aref_id=', aref_id::VARCHAR,
				' from_art=', from_art::VARCHAR,
				' from_para=', from_para::VARCHAR,
				' refs_art=', refs_art::VARCHAR,
				' refs_para=', refs_para::VARCHAR,
				' comment=', comment,
				' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
); 
CREATE INDEX aref_com_ts ON article_ref_article USING GIN(ts);



CREATE TABLE IF NOT EXISTS article_ref_video (
	/*When an article (or a paragraph in an article) references a youtube video (with an optional timestamp),
	a row is added to capture that reference */
	prior_id INTEGER UNIQUE,
	vref_id INTEGER NOT NULL PRIMARY KEY,
	art_id INTEGER NOT NULL,						-- the article making the reference
	apara_id INTEGER,								-- (optional) paragraph within the article 
	vid_pk CHAR(11) NOT NULL,						-- the vid_pk for the youtube video 
	sec_req SMALLINT,								-- (optional) requested timestamp in seconds 
	comment VARCHAR NOT NULL,						-- a brief explanation of why this is relevant or what it shows
	prior_sha256 CHAR(64) NOT NULL, 				-- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(vref_id, new_sha256), 					-- this allows the no_delete constraint below 
	UNIQUE(vref_id, vid_pk),						-- allows FOREIGN KEY contraits tying vid_pk to the references
	ts tsvector GENERATED ALWAYS AS ( to_tsvector('english', comment)) STORED,
CONSTRAINT arrefvar FOREIGN KEY (art_id) REFERENCES articles(art_id),
CONSTRAINT arrefvap FOREIGN KEY (art_id, apara_id) REFERENCES article_para (art_id, apara_id),
CONSTRAINT arrefytt FOREIGN KEY (vid_pk) REFERENCES youtube_videos(vid_pk),
CONSTRAINT arrefv_prior CHECK ( (vref_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = vref_id - 1)) ),
CONSTRAINT arrefv_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES article_ref_video (vref_id, new_sha256),
CONSTRAINT arrefv_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
CONSTRAINT arrefv_verify_sha256 CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'vref_id=', vref_id::VARCHAR,
				' art_id=', art_id::VARCHAR,
				' apara_id=', apara_id::VARCHAR,
				' vid_pk=', vid_pk,
				' sec_req=', sec_req::VARCHAR,
				' comment=', comment,
				' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
); 
CREATE INDEX vref_com_ts ON article_ref_video USING GIN(ts);




CREATE TABLE IF NOT EXISTS article_ref_image (
	/*When an article (or a paragraph in an article) references an image,
	a row is added to capture that reference */
	prior_id INTEGER UNIQUE,
	iref_id INTEGER NOT NULL PRIMARY KEY,			-- the id for this reference. Incremented for each new reference 
	art_id INTEGER NOT NULL,						-- the article making the reference
	apara_id INTEGER,								-- (optional) paragraph within the article 
	img_id INTEGER NOT NULL,						-- the image being referenced 
	comment VARCHAR NOT NULL,						-- a brief explanation of why this is relevant or what it shows
	prior_sha256 CHAR(64) NOT NULL, 				-- included for checking integrity
	write_timestamp TIMESTAMPTZ NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(iref_id, new_sha256), 					-- this allows the imref_no_delete constraint below 
	ts tsvector GENERATED ALWAYS AS ( to_tsvector('english', comment)) STORED,
CONSTRAINT imrefar FOREIGN KEY (art_id) REFERENCES articles(art_id),
CONSTRAINT imrefpa FOREIGN KEY (art_id, apara_id) REFERENCES article_para (art_id, apara_id),
CONSTRAINT imrefim FOREIGN KEY (img_id) REFERENCES images (img_id),
CONSTRAINT imref_prior CHECK ( (iref_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = iref_id - 1)) ),
CONSTRAINT imref_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES article_ref_image (iref_id, new_sha256),
CONSTRAINT imref_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
CONSTRAINT imref_verify_sha256 CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'iref_id=', iref_id::VARCHAR,
				' art_id=', art_id::VARCHAR,
				' apara_id=', apara_id::VARCHAR,
				' img_id=', img_id::VARCHAR,
				' comment=', comment,
				' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
); 
CREATE INDEX iref_com_ts ON article_ref_image USING GIN(ts);


CREATE TABLE IF NOT EXISTS nlp_topic_pos (
	/*This records the distinct types of topics based on NLP */ 
	pos CHAR(3) NOT NULL PRIMARY KEY,
	descrip VARCHAR NOT NULL UNIQUE
); 
INSERT INTO nlp_topic_pos (pos, descrip) VALUES 
('ORG', 'Organization, i.e. a company, nonprofit, etc.'), 
('GPE', 'GeoPolitical entity, typically a geographic place'),
('PER', 'Person'), 
('FAC', 'Facility'),
('NCK', 'Noun Chunk');


CREATE TABLE IF NOT EXISTS nlp_topics (
	/*Identifying topics using NLP is a convenient way to connect related articles
	using NLP */
	pos CHAR(3) NOT NULL,				-- the part-of-speech this topic belongs to
	tkey VARCHAR NOT NULL PRIMARY KEY,	-- primary key, string
	name VARCHAR NOT NULL,				-- the name, i.e. the topic
	count SMALLINT NOT NULL DEFAULT 1,	-- the frequency with which this topic has been identified 
	ac tsvector GENERATED ALWAYS AS ( to_tsvector('simple', name )) STORED,
CONSTRAINT nlpentpos FOREIGN KEY (pos) REFERENCES nlp_topic_pos(pos)
);

CREATE TABLE IF NOT EXISTS apara_ment_topic (
	/*Each time an article paragraph mentions a topic, 
	add or increment a row in this table*/
	tkey VARCHAR NOT NULL,
	art_id INTEGER NOT NULL,
	apara_id INTEGER NOT NULL,
	PRIMARY KEY (tkey, art_id, apara_id),
CONSTRAINT aparatop FOREIGN KEY (tkey) REFERENCES nlp_topics(tkey),
CONSTRAINT aparavid FOREIGN KEY (art_id, apara_id) REFERENCES article_para (art_id, apara_id) 
);
CREATE INDEX byart ON apara_ment_topic(art_id, apara_id);

