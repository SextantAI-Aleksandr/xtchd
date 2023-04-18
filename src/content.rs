use std::vec::Vec;
use serde::{Serialize, Deserialize};
use crate::integrity::{XtchdContent, ContentClass};

#[derive(Serialize, Deserialize)]
pub struct Reference {
    pub item_id: i32,
    pub item_name: Option<String>,
    pub item_sha256: String,
}


#[derive(Serialize, Deserialize)]
pub struct ClassedReference {
    pub content_class: ContentClass,
    pub reference: Reference,
}


#[derive(Serialize, Deserialize)]
pub struct ArticlePara {
    pub article_id: i32,
    pub text: String,
    pub references: Vec<ClassedReference>,
}


impl XtchdContent for ArticlePara {
    fn class(&self) -> ContentClass {
        ContentClass::ArticlePara
    }
    fn name(&self) -> Option<String> {
        None
    }
}


#[derive(Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub org: Option<String>,
}

impl XtchdContent for Author {
    fn class(&self) -> ContentClass {
        ContentClass::Author
    }
    fn name(&self) -> Option<String> {
        Some(self.name.clone())
    }
}


#[derive(Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub authors: Vec<Reference>,
    pub paragraphs: Vec<Reference>,
}

impl XtchdContent for Article {
    fn class(&self) -> ContentClass {
        ContentClass::Article
    }
    fn name(&self) -> Option<String> {
        Some(self.title.clone())
    }
}


#[derive(Serialize, Deserialize)]
pub struct TranscriptPara {
    pub video_id: String,   // CHAR(1))
    pub timestamp: f64,
    pub text: String
}

#[derive(Serialize, Deserialize)]
pub struct Transcript {
    pub video_id: String,
    pub title: String,
    pub paragraphs: Vec<Reference>,
}