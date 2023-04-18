
use std::vec::Vec;
use sha2::{Sha256, Digest}; // Digest brings the ::new() method into scope
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_json;
use chrono::{self, NaiveDate};


pub fn today() -> NaiveDate {
    // Give a NaiveDate for the current local time
    let now = chrono::offset::Local::now();
    let now_str = now.to_string()[0..10].to_string();
    NaiveDate::parse_from_str(&now_str, "%Y-%m-%d").unwrap()
}

pub fn sha256(input: &str) -> String { 
    let mut hasher = Sha256::new();                                 
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result) // lowercase hexadecimal encoding
}



#[derive(Serialize, Deserialize)]
pub enum ContentClass {
    Article,
    ArticlePara,
    ArticleAddendum,
    Author,
    Image,
    Transcript,
    TranscriptPara,
}


impl ContentClass {
    pub fn to_str(&self) -> &'static str {
        match *self {
            ContentClass::Article => "Article",
            ContentClass::ArticlePara => "ArticlePara",
            ContentClass::ArticleAddendum => "ArticleAddendum",
            ContentClass::Author => "Author",
            ContentClass::Image => "Image",
            ContentClass::Transcript => "Transcript",
            ContentClass::TranscriptPara => "TranscriptPara",            
        }
    }
}


pub trait XtchdContent: Serialize + DeserializeOwned {
    fn class(&self) -> ContentClass;
    fn class_str(&self) -> &'static str {
        self.class().to_str()
    }
    /// Things like articles will have a name, things like ArticleParagraphs or images may not 
    fn name(&self) -> Option<String>;
    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}


pub struct VerifiedItem <T: XtchdContent> {
    prior_id: i32,
    prior_sha256: String,
    uploaded_on: NaiveDate,
    content: T,
}


impl<T: XtchdContent> VerifiedItem<T> {
    fn new(prior_id: i32, prior_sha256: &str, content: T) -> Self {
        let prior_sha256 = prior_sha256.to_string();
        let uploaded_on = today();
        VerifiedItem {prior_id, prior_sha256, uploaded_on, content}
    }

    fn item_id(&self) -> i32 {
        self.prior_id + 1
    }

    fn string_to_hash(&self) -> String {
        format!("item_id={} uploaded_on={} prior_sha256={} content_class={} content_json={}",
            &self.item_id(), &self.uploaded_on, &self.prior_sha256,  &self.content.class_str(), &self.content.json() )
    }

    fn item_sha256(&self) -> String {
        sha256(&self.string_to_hash())
    }
}

