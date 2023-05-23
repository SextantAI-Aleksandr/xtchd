
use std::hash::Hash;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_json;
use tokio_postgres;
use sha2::{Sha256, Digest}; // Digest brings the ::new() method into scope
use chrono::{DateTime, offset::Utc};


pub fn now() -> DateTime<Utc> {
    // Give the current Utc tie
    Utc::now()
}

pub fn time_fmt(ts: &DateTime<Utc>) -> String {
    // format a timestamp like this:
    // 'YYYY.MM.DD HH24:MI:SS' (Postgres)
    ts.format("%Y.%m.%d %H:%M:%S").to_string()
}

pub fn sha256(input: &str) -> String { 
    let mut hasher = Sha256::new();                                 
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result) // lowercase hexadecimal encoding
}



pub trait Xtchable {
    // This should define the "state" of an object, apart from the prior_sha256 and the write_timestamp
    // It will be used in generating a hash to verify row integrity
    fn state_string(&self) -> String;
}


/// When an instance of a struct implementing the Xtchable trait is written to disk,
/// data including the prior_id, write_timestamp, and new_sha_256 are written as well.
/// This data is used in Postgres to cryptographically verify the integrity of the row being written. 
/// When the corresponding row is read back from disk, the content can be "wrapped" in a XtchdContent struct 
/// to allow demonstration of the new_sha256 matching the calculated sha256 (typically in JavaScript in the user's browser.)
#[derive(Serialize, Deserialize)]
pub struct XtchdContent<T: Xtchable> {
    pub prior_id: Option<i32>, // must only be None for the very first entry 
    pub prior_sha256: String,
    pub content: T,
    pub hcl: HashChainLink,
    pub new_sha256: String,
}

impl<T: Xtchable + Serialize + DeserializeOwned> XtchdContent<T> {

    pub fn new(prior_id: Option<i32>, prior_sha256: String, write_timestamp: DateTime<Utc>, content: T, new_sha256: String) -> Self {
        let hcl = HashChainLink::from_timestamp(&prior_sha256, write_timestamp.clone(), &content);
        XtchdContent{prior_id, prior_sha256, content, hcl, new_sha256}
    }

}


impl<'a, T: Xtchable + Serialize + DeserializeOwned> tokio_postgres::types::FromSql<'a> for XtchdContent<T> {

    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let xc: XtchdContent<T> = serde_json::from_slice(raw)?;
        Ok(xc)
    }

    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}

/// The hash chain link contains key information needed to help write Postgres rows
/// Creating a hash chain between the prior row and a new row with its content 
#[derive(Serialize, Deserialize)]
pub struct HashChainLink {
    pub write_timestamp: DateTime<Utc>,
    pub string_to_hash: String,
}

impl HashChainLink {

    
    pub fn new<T: Xtchable>(prior_sha256: &str, content: &T) -> Self {
        let write_timestamp = now();
        HashChainLink::from_timestamp(prior_sha256, write_timestamp, content)
    }

    pub fn from_timestamp<T: Xtchable>(prior_sha256: &str, write_timestamp: DateTime<Utc>, content: &T) -> Self {
        let string_to_hash = format!("{} write_timestamp={} prior_sha256={}",
            content.state_string(), time_fmt(&write_timestamp), &prior_sha256); 
        HashChainLink{write_timestamp, string_to_hash}
    }


    pub fn new_sha256(&self) -> String {
        sha256(&self.string_to_hash)
    }
}



