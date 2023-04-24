//! rows.rs contains a struct corresponding to a row for each of the main tables in schema.sql 
//! xtchr.rs contains the Xtchr struct, which "etches" (or writes) one row at a time to Postgres
//! with cryptographic verification. 
use std::hash::Hash;

// use postgres::types::ToSql;
use pachydurable::{connect::{ConnPoolNoTLS, ClientNoTLS, pool_no_tls_from_env}, err::GenericError};
use crate::{rows, integrity::{Xtchable, HashChainLink}};


pub struct LastRow {
    pub prior_id: i32,
    pub prior_sha256: String,
}


async fn get_last_row(c: &ClientNoTLS, query: &'static str) -> Result<LastRow, GenericError> {
    let rows = c.query(query, &[]).await.unwrap();
    let row = rows.get(0).unwrap();
    let prior_id: i32 = row.get(0);
    let prior_sha256: String = row.get(1);
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


    pub async fn get(&self) -> Result<Xtchr, GenericError> {
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

    // add an author
    pub async fn add_author(&self, name: &str) -> Result<(rows::Author, HashChainLink), GenericError> {
        let last_author = get_last_row(&self.c, "SELECT auth_id, new_sha256 FROM authors ORDER BY auth_id DESC LIMIT 1").await.unwrap();
        let auth_id = last_author.prior_id + 1;
        let name = name.to_string();
        let author = rows::Author{auth_id, name};
        let hclink = HashChainLink::new(&last_author.prior_sha256, &author);
        let _x = self.c.execute("INSERT INTO authors
            (                     prior_id,         auth_id,        name,               prior_sha256,         write_timestamp,         new_sha256) 
                VALUES ($1, $2, $3, $4, $5, $6)", 
            &[&last_author.prior_id, &author.auth_id, &author.name, &last_author.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]
        ).await.unwrap();
        Ok((author, hclink))
    }

    // add an article (but not the text thereof)
    pub async fn add_article(&self, auth_id: i32, title: &str) -> Result<(rows::Article, HashChainLink), GenericError> {
        let last_article = get_last_row(&self.c, "SELECT art_id, new_sha256 FROM articles ORDER BY art_id DESC LIMIT 1").await.unwrap();
        let art_id = last_article.prior_id + 1;
        let title = title.to_string();
        let article = rows::Article{art_id, auth_id, title};
        let hclink = HashChainLink::new(&last_article.prior_sha256, &article);
        let _x = self.c.execute("INSERT INTO articles
            (                   prior_id,  art_id, auth_id,          title,               prior_sha256,         write_timestamp,          new_sha256)
                VALUES ($1, $2, $3, $4, $5, $6, $7) ",
        &[&last_article.prior_id, &art_id, &auth_id, &article.title, &last_article.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256() ]
        ).await.unwrap();
        Ok((article, hclink))
    }


    pub async fn add_article_para(&self, art_id: i32, md: &str, html: &str) -> Result<(rows::ArticlePara, HashChainLink), GenericError> {
        let last_para = get_last_row(&self.c, "SELECT apara_id, new_sha256 FROM article_paragraphs ORDER BY apara_id DESC LIMIT 1").await.unwrap();
        let apara_id = last_para.prior_id + 1;
        let md = md.to_string();
        let html = html.to_string();
        let para = rows::ArticlePara{apara_id, art_id, md, html};
        let hclink = HashChainLink::new(&last_para.prior_sha256, &para);
        let _x = self.c.execute("INSERT INTO article_paragraphs
            (       prior_id,  apara_id,   art_id,      md,       html,            prior_sha256,         write_timestamp,           new_sha256)
                VALUES ($1, $2, $3, $4, $5, $6, $7) ",
        &[&last_para.prior_id, &apara_id, &art_id, &para.md, &para.html, &last_para.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256() ]
        ).await.unwrap();
        Ok((para, hclink))
    }


}



