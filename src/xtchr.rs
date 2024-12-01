//! rows.rs contains a struct corresponding to a row for each of the main tables in schema.sql 
//! xtchr.rs contains the Xtchr struct, which "etches" (or writes) one row at a time to Postgres
//! with cryptographic verification. 

use chrono::{NaiveDate, DateTime, offset::Utc};
use pachydurable::{connect::{ConnPoolNoTLS, ClientNoTLS, pool_no_tls_from_env}, err::{PachyDarn, MissingRowError}};
use pachydurable::redis as predis;
use crate::{xrows, views, integrity::{XtchdContent, HashChainLink}};


pub struct LastRow {
    /// This is the latest/highest id in the table. It will only be None for the very first entry 
    pub prior_id: Option<i32>,
    pub prior_sha256: String,
}

impl LastRow {
    pub fn next_id(&self) -> i32 {
        match self.prior_id {
            None => 0,
            Some(i) => i + 1,
        }
    }
}


/// This function is intended to get a query that sorts by id (returning the highest/latest)
/// for use in tables with hash integrity.
/// If no prior entry has been make, it will return a default value 
async fn get_last_row(c: &ClientNoTLS, query: &'static str) -> Result<LastRow, PachyDarn> {
    let rows = c.query(query, &[]).await?;
    let (prior_id, prior_sha256) = match rows.get(0) {
        Some(row) => (Some(row.get(0)), row.get(1)),
        None => (None, "0000000000000000000000000000000000000000000000000000000000000000".to_string()),
    };
    Ok(LastRow{prior_id, prior_sha256})
}


pub struct Pool {
    pub pool: ConnPoolNoTLS,
}

impl Pool {
    /// Instantiate a new pool from these environment variables:
    /// PSQL_HOST,  host        defaults to "127.0.0.1"
    /// PSQL_PORT,  port        defaults to 5432
    /// PSQL_PW,    password 
    /// PSQL_USER,  user        defaults to 'postgres'
    /// PSQL_DB,    database    defaults to 'postgres'
    pub async fn new_from_env() -> Self {
        let pool = pool_no_tls_from_env().await.unwrap();
        let _c = pool.get().await.unwrap(); // ensure you can connect
        Pool{pool}
    }


    pub async fn get(&self) -> Result<Xtchr, PachyDarn> {
        let c = self.pool.get().await.unwrap();
        Ok(Xtchr{c})
    }

}

/// The Xtrcr struct is essentially a Postgres client with special methods implemented on it
/// To write rows with hash chained integrity
pub struct Xtchr {
    pub c: ClientNoTLS
}

impl Xtchr {




    /// Get the detail for one author, specified by auth_id
    pub async fn author_detail(&self, auth_id: i32) -> Result<views::AuthorDetail, PachyDarn> {
        let query = "SELECT prior_id, name, prior_sha256, write_timestamp, new_sha256, authored
            FROM author_detail WHERE auth_id = $1";
        let rows = self.c.query(query, &[&auth_id]).await?;
        let row = match rows.get(0) {
            Some(val) => val,
            None => return Err(PachyDarn::from(MissingRowError::from_str("missing row in query for author_detail()"))),
        };
        let prior_id: Option<i32> = row.get(0);
        let name: String = row.get(1);
        let prior_sha256: String = row.get(2);
        let write_timestamp: DateTime<Utc> = row.get(3);
        let new_sha256: String = row.get(4);
        let articles:  Vec<views::NameId>  = row.get(5);
        let content = xrows::Author{auth_id, name};
        let author = XtchdContent::new(prior_id, prior_sha256, write_timestamp, content, new_sha256);
        Ok(views::AuthorDetail{author, articles})
    }

    // add an author
    pub async fn add_author(&self, name: &str) -> Result<(xrows::Author, HashChainLink), PachyDarn> {
        let last_author = get_last_row(&self.c, "SELECT auth_id, new_sha256 FROM authors ORDER BY auth_id DESC LIMIT 1").await.unwrap();
        let auth_id = last_author.next_id();
        let name = name.to_string();
        let author = xrows::Author{auth_id, name};
        let hclink = HashChainLink::new(&last_author.prior_sha256, &author);
        let _x = self.c.execute("INSERT INTO authors
            (                     prior_id,         auth_id,        name,               prior_sha256,         write_timestamp,         new_sha256) 
                VALUES ($1, $2, $3, $4, $5, $6)", 
            &[&last_author.prior_id, &author.auth_id, &author.name, &last_author.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]
        ).await.unwrap();
        Ok((author, hclink))
    }


    // add an article (but not the text thereof)
    pub async fn add_article_title(&self, auth_id: i32, title: &str) -> Result<(xrows::ArticleTitle, HashChainLink), PachyDarn> {
        let last_article = get_last_row(&self.c, "SELECT art_id, new_sha256 FROM articles ORDER BY art_id DESC LIMIT 1").await.unwrap();
        let art_id = last_article.next_id();
        let title = title.to_string();
        let art_title = xrows::ArticleTitle{art_id, auth_id, title};
        let hclink = HashChainLink::new(&last_article.prior_sha256, &art_title);
        let _x = self.c.execute("INSERT INTO article_titles_immut
            (                   prior_id,  art_id, auth_id,            title,               prior_sha256,         write_timestamp,          new_sha256)
                VALUES ($1, $2, $3, $4, $5, $6, $7) ",
        &[&last_article.prior_id, &art_id, &auth_id, &art_title.title, &last_article.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256() ]
        ).await.unwrap();
        Ok((art_title, hclink))
    }


    /// add a (new) page to an article 
    pub async fn add_article_page(&self, art_id: i32, paragraphs: Vec<String>, source: xrows::PageSrc) -> Result<(xrows::ArticlePage, HashChainLink), PachyDarn> {
        let last_page = get_last_row(&self.c, "SELECT apage_id, new_sha256 FROM article_pages_immut ORDER BY apara_id DESC LIMIT 1").await.unwrap();
        let apage_id = last_page.next_id();
        let page = xrows::ArticlePage{art_id, apage_id, paragraphs, source};
        let hclink = HashChainLink::new(&last_page.prior_sha256, &page);
        let (img_id, image_file, refs_art_id) = &page.source.src_columns();
        let _x = self.c.execute("INSERT INTO article_pages_immut
            (       prior_id,  apage_id,   art_id,       paragraphs, img_id, image_file, refs_art_id,                prior_sha256,         write_timestamp,           new_sha256)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) ",
        &[&last_page.prior_id, &apage_id, &art_id, &page.paragraphs, &img_id, &image_file, &refs_art_id, &last_page.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256() ]
        ).await.unwrap();
        Ok((page, hclink))
    }


    // create a new record for a youtube channel
    pub async fn add_youtube_channel(&self, url: &str, name: &str) -> Result<(xrows::YoutubeChannel, HashChainLink), PachyDarn> {
        let last_chan = get_last_row(&self.c, "SELECT chan_id, new_sha256 FROM youtube_channels ORDER BY chan_id DESC LIMIT 1").await.unwrap();
        let chan_id = last_chan.next_id();
        let url = url.to_lowercase();
        let name = name.to_string();
        let chan = xrows::YoutubeChannel{chan_id, url, name};
        let hclink = HashChainLink::new(&last_chan.prior_sha256, &chan);
        let _x = self.c.execute("INSERT INTO youtube_channels 
            (                    prior_id, chan_id,       url,       name,             prior_sha256,        write_timestamp,           new_sha256)
                VALUES ($1, $2, $3, $4, $5, $6, $7) ",
            &[&last_chan.prior_id, &chan_id, &chan.url, &chan.name, &last_chan.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]
        ).await.unwrap();
        Ok((chan, hclink))
    }


    // create a new record for a youtube video 
    pub async fn add_youtube_video(&self, chan_id: i32, vid_pk: &str, title: &str, date_uploaded: &NaiveDate) -> Result<(xrows::YoutubeVideo, HashChainLink), PachyDarn> {
        let last_vid = get_last_row(&self.c, "SELECT vid_id, new_sha256 FROM youtube_videos ORDER BY vid_id DESC LIMIT 1").await.unwrap();
        let vid_id = last_vid.next_id();
        let vid_pk = vid_pk.to_string();
        let title = title.to_string();
        let date_uploaded = date_uploaded.clone();
        let video = xrows::YoutubeVideo{vid_id, vid_pk, chan_id, title, date_uploaded};
        let hclink = HashChainLink::new(&last_vid.prior_sha256, &video);
        let _x = self.c.execute("INSERT INTO youtube_videos 
            (                  prior_id,  vid_id,         vid_pk,       chan_id,        title,        date_uploaded,           prior_sha256,         write_timestamp,           new_sha256)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (vid_pk) DO NOTHING",
            &[&last_vid.prior_id, &vid_id, &video.vid_pk, &video.chan_id, &video.title, &video.date_uploaded, &last_vid.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]
        ).await.unwrap();
        Ok((video, hclink))
    }


    /// add a new immutable image/thumbnail pair, returning the img_id
    pub async fn add_image_immutable(&self, pair: xrows::ImagePair) -> Result<i32, PachyDarn> {
        let last_ref = get_last_row(&self.c, "SELECT img_id, new_sha256 FROM images ORDER BY img_id DESC LIMIT 1").await.unwrap();
        let img_id = last_ref.next_id();
        let ii = xrows::ImmutableImage{img_id, pair};
        let hclink = HashChainLink::new(&last_ref.prior_sha256, &ii);
        let _x = self.c.execute("INSERT INTO images 
            (                  prior_id,  img_id,          src_full,          src_thmb,          alt,          url,          archive,           prior_sha256,         write_timestamp,          new_sha256) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[&last_ref.prior_id, &img_id, &ii.pair.src_full, &ii.pair.src_thmb, &ii.pair.alt, &ii.pair.url, &ii.pair.archive, &last_ref.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]).await?;
        Ok(img_id)
    }


    /// add or update a new mutable image/thumbnail pair 
    pub async fn add_image_mutable(&self, mi: &xrows::MutableImage) -> Result<(), PachyDarn> {
        let _x = self.c.execute("INSERT INTO images_mut
            (            id,          src_full,          src_thmb,          alt,          url) VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT(id) DO UPDATE SET src_full = $2, src_thmb = $3, alt = $4, url = $5",
            &[&mi.id, &mi.pair.src_full, &mi.pair.src_thmb, &mi.pair.alt, &mi.pair.url]).await?;
        Ok(())
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_init_author() {
        // Test the author_detail function by getting the initia "seed" author
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = Pool::new_from_env().await;
            let x = pool.get().await.unwrap();
            let au = x.author_detail(0).await.unwrap();
            assert_eq!(au.author.content.name, "Xtchd Admins".to_string());
        });
    }

}
