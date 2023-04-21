//! rows.rs contains a struct corresponding to a row for each of the main tables in schema.sql 
//! xtchr.rs contains the Xtchr struct, which "etches" (or writes) one row at a time to Postgres
//! with cryptographic verification. 
use postgres::types::ToSql;
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


pub struct Xtchr {
    pool: ConnPoolNoTLS,
    pub last_author: LastRow,
    pub last_article: LastRow,
}

impl Xtchr {

    // Instantiate a new writer
    pub async fn new_from_env() -> Self {
        let pool = pool_no_tls_from_env().await.unwrap();
        let c = pool.get().await.unwrap();
        let last_author = get_last_row(&c, "SELECT auth_id, new_sha256 FROM authors ORDER BY auth_id DESC LIMIT 1").await.unwrap();
        let last_article = get_last_row(&c, "SELECT art_id, new_sha256 FROM articles ORDER BY art_id DESC LIMIT 1").await.unwrap();
        Xtchr{pool, last_author, last_article}
    }

    // add an author, returning the auth_id
    pub async fn add_author(&mut self, name: &str) -> Result<(rows::Author, HashChainLink), GenericError> {
        let auth_id = self.last_author.prior_id + 1;
        let name = name.to_string();
        let author = rows::Author{auth_id, name};
        let hclink = HashChainLink::new(&self.last_author.prior_sha256, &author);
        let c = self.pool.get().await?;
        let _x = c.execute("INSERT INTO authors
            (                           prior_id,         auth_id,        name,                   prior_sha256,         write_timestamp,         new_sha256) 
                VALUES ($1, $2, $3, $4, $5, $6)", 
            &[&self.last_author.prior_id, &author.auth_id, &author.name, &self.last_author.prior_sha256, &hclink.write_timestamp, &hclink.new_sha256()]
        ).await.unwrap();
        // update the last author based on the one you just created 
        self.last_author.prior_id = auth_id;
        self.last_author.prior_sha256 = hclink.new_sha256();
        Ok((author, hclink))
    }

}



