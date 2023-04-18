
use serde::{Serialize, Deserialize};
use crate::{content, integrity::{XtchdContent, VerifiedItem}};

pub struct Article {
    pub title: String,
    pub authors: Vec<VerifiedItem<content::Author>>,
    pub paragraphs: Vec<VerifiedItem<content::ArticlePara>>,
}