
use std::vec::Vec;
use sha2::{Sha256, Digest}; // Digest brings the ::new() method into scope
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_json;
use chrono::{DateTime, offset::Utc};


pub fn now() -> DateTime<Utc> {
    // Give the current Utc tie
    Utc::now()
}

pub fn now_fmt(ts: &DateTime<Utc>) -> String {
    // format a timestamp like this:
    // 'YYYY.MM.DD HH24:MI:SS' (Postgres)
    ts.format("%Y-%m-%d %H:%M:%S")
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
    /// Overload this with the title for articles or the text for a paragraph etc.
    fn name_or_text(&self) -> Option<String>;
    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    // overloaded to be the article_id (for a paragraph), video_id (for transcript paragraph), or destination for a reference
    fn to_parent(&self) -> Option<i32>; 
    // If the item is a reference, this should be the source_id
    fn from_child(&self) -> Option<i32>;
}


pub struct VerifiedItem <T: XtchdContent> {
    pub prior_id: i32,
    pub prior_sha256: String,
    pub uploaded_at: DateTime<Utc>,
    pub content: T,
}


impl<T: XtchdContent> VerifiedItem<T> {
    pub fn new(prior_id: i32, prior_sha256: &str, content: T) -> Self {
        let prior_sha256 = prior_sha256.to_string();
        let uploaded_at = now();
        VerifiedItem {prior_id, prior_sha256, uploaded_at, content}
    }

    pub fn id(&self) -> i32 {
        self.prior_id + 1
    }

    pub fn string_to_hash(&self) -> String {
        let name_or_text = self.content.name_or_text().unwrap_or_default();
        let to_parent = match self.content.to_parent() {
            Some(id) => id.to_string(),
            None => "".to_string()
        };
        let from_child = match self.content.from_child() {
            Some(id) => id.to_string(),
            None => "".to_string()
        };
        let uploaded_at = now_fmt(&self.uploaded_at);
        format!("id={} content_class={} content_json={} name_or_text={} to_parent={} from_child={} uploaded_at={} prior_sha256={}",
            &self.id(), &self.content.class_str(), &self.content.json(), &name_or_text, &to_parent, &from_child, &uploaded_at, &self.prior_sha256
        )
    }

    pub fn new_sha256(&self) -> String {
        sha256(&self.string_to_hash())
    }
}

