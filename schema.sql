

CREATE TABLE IF NOT EXISTS authors (
	prior_id INTEGER UNIQUE,
	auth_id INTEGER NOT NULL PRIMARY KEY,
	name VARCHAR NOT NULL UNIQUE,
	prior_sha256 CHAR(64) NOT NULL, -- included for checking integrity
	write_timestamp TIMESTAMP NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(auth_id, new_sha256), -- this allows the constraint below 
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


CREATE TABLE IF NOT EXISTS articles (
	prior_id INTEGER UNIQUE,
	art_id INTEGER NOT NULL PRIMARY KEY,
	auth_id INTEGER NOT NULL,
	title VARCHAR NOT NULL UNIQUE,
	prior_sha256 CHAR(64) NOT NULL, -- included for checking integrity
	write_timestamp TIMESTAMP NOT NULL,     
	new_sha256 CHAR(64) NOT NULL,
	UNIQUE(art_id, new_sha256), -- this allows the constraint below 
	ac tsvector GENERATED ALWAYS AS ( to_tsvector('simple', title )) STORED,
CONSTRAINT art_auth FOREIGN KEY (auth_id) REFERENCES authors(auth_id),
CONSTRAINT art_prior CHECK ( (art_id = 0) OR ((prior_id IS NOT NULL) AND (prior_id = art_id - 1)) ),
CONSTRAINT art_no_delete FOREIGN KEY (prior_id, prior_sha256) REFERENCES articles (art_id, new_sha256),
CONSTRAINT art_no_rewrite_later CHECK (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - write_timestamp)) <= 1),
CONSTRAINT art_verify_sha256 CHECK (
	ENCODE(
		SHA256(
			CONCAT(
				'art_id=', art_id::VARCHAR,
				' auth_id=', art_id::VARCHAR,
				' title=', title,
				' write_timestamp=', TO_CHAR(write_timestamp, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', prior_sha256
			)::BYTEA
		),
	'hex') = new_sha256)
);
INSERT INTO articles (art_id, auth_id, title, prior_sha256, write_timestamp, new_sha256) 
VALUES (0, 0, 'Initial Article', '0000000000000000000000000000000000000000000000000000000000000000',
	CURRENT_TIMESTAMP, 
	ENCODE(
		SHA256(
			CONCAT(
				'art_id=', 0::VARCHAR,
				' auth_id=', 0::VARCHAR,
				' title=', 'Initial Article',
				' write_timestamp=', TO_CHAR(CURRENT_TIMESTAMP, 'YYYY.MM.DD HH24:MI:SS'),
				' prior_sha256=', '0000000000000000000000000000000000000000000000000000000000000000'
			)::BYTEA
		),
	'hex')
);