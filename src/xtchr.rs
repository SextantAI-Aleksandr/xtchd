//! rows.rs contains a struct corresponding to a row for each of the main tables in schema.sql 
//! xtchr.rs contains the Xtchr struct, which "etches" (or writes) one row at a time to Postgres
//! with cryptographic verification. 

use chrono::{NaiveDate, DateTime, offset::Utc};
use pachydurable::{connect::{ConnPoolNoTLS, ClientNoTLS, pool_no_tls_from_env}, err::DiskError};
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
async fn get_last_row(c: &ClientNoTLS, query: &'static str) -> Result<LastRow, DiskError> {
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


    pub async fn get(&self) -> Result<Xtchr, DiskError> {
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

    /// This method is called to get the most recent articles (but not the associated text)
    /// Think of it as giving the headline for the most recent articles
    pub async fn latest_headlines(&self) -> Result<Vec<xrows::Article>, DiskError> {
        let query = "SELECT art_id, auth_id, title, image_file FROM articles 
            ORDER BY art_id DESC LIMIT 12";
        let rows = self.c.query(query, &[]).await?;
        let mut articles = Vec::new();
        for row in rows {
            let art_id: i32 = row.get(0);
            let auth_id: i32 = row.get(1);
            let title: String = row.get(2);
            let image_file: Option<String> = row.get(3);
            articles.push(xrows::Article{art_id, auth_id, title, image_file});
        }
        Ok(articles)
    }


    /// Get one article, specified by id 
    pub async fn article_text(&self, art_id: i32) -> Result<views::ArticleText, DiskError> {
        let query = "SELECT author, article, art_paras FROM article_text WHERE art_id = $1";
        let rows = self.c.query(query, &[&art_id]).await?;
        let row = match rows.get(0) {
            Some(val) => val,
            None => return Err(DiskError::missing_row())
        };
        let author: XtchdContent<xrows::Author> = row.get(0);
        let article: XtchdContent<xrows::Article> = row.get(1);
        let paragraphs: Vec<XtchdContent<xrows::ArticlePara>> = row.get(2);
        Ok(views::ArticleText{author, article, paragraphs})
    }


    /// Get the detail for one author, specified by auth_id
    pub async fn author_detail(&self, auth_id: i32) -> Result<views::AuthorDetail, DiskError> {
        let query = "SELECT prior_id, name, prior_sha256, write_timestamp, new_sha256, authored
            FROM author_detail WHERE auth_id = $1";
        let rows = self.c.query(query, &[&auth_id]).await?;
        let row = match rows.get(0) {
            Some(val) => val,
            None => return Err(DiskError::missing_row())
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


    /// Get an enriched article struct given the article id 
    pub async fn enriched_article(&self, rpool: &predis::RedisPool, art_id: i32) -> Result<views::EnrichedArticle, DiskError> {
        let oea: Option<views::EnrichedArticle> = predis::cached_or_cache(&self.c, rpool, &[&art_id]).await.unwrap();
        match oea {
            Some(ea) => Ok(ea),
            None => Err(DiskError::missing_row()),
        }
    }


    // add an author
    pub async fn add_author(&self, name: &str) -> Result<(xrows::Author, HashChainLink), DiskError> {
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
    pub async fn add_article(&self, auth_id: i32, title: &str, image_file: &Option<String>) -> Result<(xrows::Article, HashChainLink), DiskError> {
        let last_article = get_last_row(&self.c, "SELECT art_id, new_sha256 FROM articles ORDER BY art_id DESC LIMIT 1").await.unwrap();
        let art_id = last_article.next_id();
        let title = title.to_string();
        let image_file = match image_file {
            None => None,
            Some(filename) => {
                let _x = self.c.execute("INSERT INTO image_files (image_file) VALUES ($1)
                    ON CONFLICT (image_file) DO NOTHING", &[&filename]).await.unwrap();
                Some(filename.to_owned())
            }
        };
        let article = xrows::Article{art_id, auth_id, title, image_file};
        let hclink = HashChainLink::new(&last_article.prior_sha256, &article);
        let _x = self.c.execute("INSERT INTO articles
            (                   prior_id,  art_id, auth_id,          title,               prior_sha256,         write_timestamp,          new_sha256,          image_file)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ",
        &[&last_article.prior_id, &art_id, &auth_id, &article.title, &last_article.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256(), &article.image_file ]
        ).await.unwrap();
        Ok((article, hclink))
    }


    /// add a paragarph for an article 
    pub async fn add_article_para(&self, art_id: i32, md: &str) -> Result<(xrows::ArticlePara, HashChainLink), DiskError> {
        let last_para = get_last_row(&self.c, "SELECT apara_id, new_sha256 FROM article_para ORDER BY apara_id DESC LIMIT 1").await.unwrap();
        let apara_id = last_para.next_id();
        let md = md.to_string();
        let para = xrows::ArticlePara{apara_id, art_id, md};
        let hclink = HashChainLink::new(&last_para.prior_sha256, &para);
        let _x = self.c.execute("INSERT INTO article_para
            (       prior_id,  apara_id,   art_id,       md,                prior_sha256,         write_timestamp,           new_sha256)
                VALUES ($1, $2, $3, $4, $5, $6, $7) ",
        &[&last_para.prior_id, &apara_id, &art_id, &para.md, &last_para.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256() ]
        ).await.unwrap();
        Ok((para, hclink))
    }


    // create a new record for a youtube channel
    pub async fn add_youtube_channel(&self, url: &str, name: &str) -> Result<(xrows::YoutubeChannel, HashChainLink), DiskError> {
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
    pub async fn add_youtube_video(&self, chan_id: i32, vid_pk: &str, title: &str, date_uploaded: &NaiveDate) -> Result<(xrows::YoutubeVideo, HashChainLink), DiskError> {
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
    pub async fn add_image_immutable(&self, pair: xrows::ImagePair) -> Result<i32, DiskError> {
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
    pub async fn add_image_mutable(&self, mi: &xrows::MutableImage) -> Result<(), DiskError> {
        let _x = self.c.execute("INSERT INTO images_mut
            (            id,          src_full,          src_thmb,          alt,          url) VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT(id) DO UPDATE SET src_full = $2, src_thmb = $3, alt = $4, url = $5",
            &[&mi.id, &mi.pair.src_full, &mi.pair.src_thmb, &mi.pair.alt, &mi.pair.url]).await?;
        Ok(())
    }


    /// add a reference from an article to an article, returning the aref_id
    pub async fn add_ref_article(&self, req: xrows::ArticleRefArticleReq) -> Result<i32, DiskError> {
        let last_ref = get_last_row(&self.c, "SELECT aref_id, new_sha256 FROM article_ref_article ORDER BY aref_id DESC LIMIT 1").await.unwrap();
        let aref_id = last_ref.next_id();
        let aref = xrows::ArticleRefArticle::from_req(req, aref_id);
        let hclink = HashChainLink::new(&last_ref.prior_sha256, &aref);
        let _x = self.c.execute("INSERT INTO article_ref_article 
            (                       prior_id,  aref_id,       from_art,         from_para,       refs_art,       refs_para,          comment,           prior_sha256,         write_timestamp,          new_sha256) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[&last_ref.prior_id, &aref_id, &aref.rf.art_id, &aref.rf.apara_id, &aref.refs_art, &aref.refs_para, &aref.rf.comment, &last_ref.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]).await.unwrap();
        Ok(aref_id)
    }


    /// add a reference from an article to a video, returning the vref_id
    pub async fn add_ref_video(&self, req: xrows::ArticleRefVideoReq) -> Result<i32, DiskError> {
        let last_ref = get_last_row(&self.c, "SELECT vref_id, new_sha256 FROM article_ref_video ORDER BY vref_id DESC LIMIT 1").await.unwrap();
        let vref_id = last_ref.next_id();
        let vref = xrows::ArticleRefVideo::from_req(req, vref_id);
        let hclink = HashChainLink::new(&last_ref.prior_sha256, &vref);
        let _x = self.c.execute("INSERT INTO article_ref_video 
            (                  prior_id,  vref_id,          art_id,          apara_id,       vid_pk,       sec_req,          comment,           prior_sha256,         write_timestamp,         new_sha256) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[&last_ref.prior_id, &vref_id, &vref.rf.art_id, &vref.rf.apara_id, &vref.vid_pk, &vref.sec_req, &vref.rf.comment, &last_ref.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]).await.unwrap();
        Ok(vref_id)
    }


    /// add a reference from an article to an image, returning the iref_id
    pub async fn add_ref_image(&self, req: xrows::ArticleRefImageReq) -> Result<i32, DiskError> {
        let last_ref = get_last_row(&self.c, "SELECT iref_id, new_sha256 FROM article_ref_image ORDER BY iref_id DESC LIMIT 1").await.unwrap();
        let iref_id = last_ref.next_id();
        let iref = xrows::ArticleRefImage::from_req(req, iref_id);
        let hclink = HashChainLink::new(&last_ref.prior_sha256, &iref);
        let _x = self.c.execute("INSERT INTO article_ref_image 
            (                  prior_id,  iref_id,          art_id,          apara_id,       img_id,          comment,           prior_sha256,         write_timestamp,         new_sha256) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[&last_ref.prior_id, &iref_id, &iref.rf.art_id, &iref.rf.apara_id, &iref.img_id, &iref.rf.comment, &last_ref.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]).await.unwrap();
        Ok(iref_id)
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

    #[test]
    fn test_init_article() {
        // Test the article_detail function by getting the initia "seed" article
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = Pool::new_from_env().await;
            let x = pool.get().await.unwrap();
            let atxt = x.article_text(0).await.unwrap();
            assert_eq!(&atxt.author.content.name, &"Xtchd Admins".to_string());
            assert_eq!(&atxt.article.content.title, &"Initial Article".to_string());
            assert_eq!(&atxt.paragraphs.len(), &1);
        });
    }

}
