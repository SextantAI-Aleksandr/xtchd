
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



/// The hash chain link contains key information needed to help write Postgres rows
/// Creating a hash chain between the prior row and a new row with its content 
pub struct HashChainLink {
    pub write_timestamp: DateTime<Utc>,
    pub string_to_hash: String,
}

impl HashChainLink {

    pub fn new<T: Xtchable>(prior_sha256: &str, content: &T) -> Self {
        let write_timestamp = now();
        let string_to_hash = format!("{} write_timestamp={} prior_sha256={}",
            content.state_string(), time_fmt(&write_timestamp), &prior_sha256); 
        HashChainLink{write_timestamp, string_to_hash}
    }

    pub fn new_sha256(&self) -> String {
        sha256(&self.string_to_hash)
    }
}



