

use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_json;
use tokio_postgres;
use sha2::{Sha256, Digest}; // Digest brings the ::new() method into scope
use chrono::{DateTime, offset::Utc};


/// Rust does not allow Options to be displayed using the "{}" format
/// They can be displayed with the {:?} format, but this wraps the Some variant in 'Some()'
/// and returns 'None' for the None variant.  
/// In contrast, Postgres renders NULL values as simply "" in string formatting.   
/// To ensure that hash values calculated using the Xtchable trait match those implemented with
/// Postgres constraints, the nonefmt function returns a blank string for None variants
/// and removes the Some() wrapper for Some variants 
pub fn nonefmt<T: std::fmt::Display>(opt: &Option<T>) -> String {
    match opt {
        Some(val) => format!("{}", val),
        None => String::new(),
    }
}

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



/// The Xtchable trait is the key trait that should be implemented on a struct to allow hash chain integrity.
/// Simply put, a struct implementing Xtchable will have a .state_string() method which returns a string describing
/// the state of the struct. This is implemented in a manner which matches a corresponding CHECK CONSTRAINT in Postgres
/// which checks the hash integrity of each entry prior to a row being written.
pub trait Xtchable {
    
    /// The state_string() method describes the state of the object, similar to serialization.
    /// The exact implementation is a bit arbitrary, but whatever implementation is chosen, it must
    /// match a corresponding CHECK CONSTRAINT in Postgres.
    fn state_string(&self) -> String;

    /// The dtype indicates the "data type" for a struct, simply just the name of the struct itself 
    /// (this cannot be easily implemented otherwise as Rust does not have reflection).  
    /// A dtype is useful so the XtchedContent struct can include a .dtype field which indicates which
    /// type of Xtched content is included in non-strictly typed languages, namely Javascript. s
    fn dtype() -> &'static str;
}


/// When an instance of a struct implementing the Xtchable trait is written to disk,
/// data including the prior_id, write_timestamp, and new_sha_256 are written as well.
/// This data is used in Postgres to cryptographically verify the integrity of the row being written. 
/// When the corresponding row is read back from disk, the content can be "wrapped" in a XtchdContent struct 
/// to allow demonstration of the new_sha256 matching the calculated sha256 (typically in JavaScript in the user's browser.)
#[derive(Serialize, Deserialize)]
pub struct XtchdContent<T: Xtchable> {
    pub dtype: String,
    pub prior_id: Option<i32>, // must only be None for the very first entry 
    pub prior_sha256: String,
    pub content: T,
    pub hcl: HashChainLink,
    /// the write_timestamp but formatted with time_fmt
    pub write_timestamp_str: String,    
    pub new_sha256: String,
}



/// When constructing an XtchdContent<T> instance, you need to know the HashChainLink which includes the .string_to_hash property
/// But the string_to_hash is not stored in postgres!
/// Instead, the XtchdSQL struct can be deserializsed from an SQL row that containsthe content<T>, a write_timestamp, and a prior_sha256
/// This is sufficient to generate the HashChainLink.  
/// NOTE: This means that the SQL query for XtchdContent<T> should actually return the JSON that should be deserialized to XtchdSQL<T>
#[derive(Deserialize)]
pub struct XtchdSQL<T: Xtchable> {
    pub prior_id: Option<i32>, // must only be None for the very first entry 
    pub prior_sha256: String,
    pub content: T,
    pub write_timestamp: DateTime<Utc>,
    pub new_sha256: String,
}


impl<T: Xtchable + Serialize + DeserializeOwned> XtchdContent<T> {

    pub fn new(prior_id: Option<i32>, prior_sha256: String, write_timestamp: DateTime<Utc>, content: T, new_sha256: String) -> Self {
        let hcl = HashChainLink::from_timestamp(&prior_sha256, write_timestamp.clone(), &content);
        let dtype = T::dtype().to_string();
        let write_timestamp_str = time_fmt(&write_timestamp);
        XtchdContent{dtype, prior_id, prior_sha256, content, hcl, new_sha256, write_timestamp_str}
    }

    pub fn from_sql(xsql: XtchdSQL<T>) -> Self {
        XtchdContent::new(xsql.prior_id, xsql.prior_sha256, xsql.write_timestamp, xsql.content, xsql.new_sha256)
    }

}


impl<'a, T: Xtchable + Serialize + DeserializeOwned> tokio_postgres::types::FromSql<'a> for XtchdContent<T> {

    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let xsql: XtchdSQL<T> = serde_json::from_slice(raw)?;
        let xc = XtchdContent::from_sql(xsql);
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


