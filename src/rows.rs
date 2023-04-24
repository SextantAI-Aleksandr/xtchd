//! This module contains a struct corresponding to one row for each of the main tables in xtchr
//! Serialization and deserialization are implemented to enable passing structs over http

use serde::{Serialize, Deserialize};
use tokio_postgres;
use pachydurable::autocomplete::{AutoComp, WhoWhatWhere};
use crate::integrity::Xtchable;


#[derive(Serialize, Deserialize)]
pub struct Author {
    pub auth_id: i32,   // the primary key for this author
    pub name: String,
}

impl Xtchable for Author {
    fn state_string(&self) -> String {
        format!("auth_id={} name={}", &self.auth_id, &self.name)
    }
}

impl AutoComp<i32> for Author {
    fn query_autocomp() ->  & 'static str {
        "SELECT auth_id, name  
        FROM authors
        WHERE ac @@ to_tsquery('simple', $1)
        ORDER BY LENGTH(name) ASC 
        LIMIT 10;"
    }
    fn rowfunc_autocomp(row: &tokio_postgres::Row) -> WhoWhatWhere<i32> {
        let data_type = "Author";
        let auth_id: i32 = row.get(0);
        let name: String = row.get(1);
        WhoWhatWhere{data_type, pk: auth_id, name}
    }
}


#[derive(Serialize, Deserialize)]
pub struct Article {
    pub art_id: i32,    // the primary key for this article
    pub auth_id: i32,   // the primary key fo r the author
    pub title: String,
}

impl Xtchable for Article {
    fn state_string(&self) -> String {
        format!("art_id={} auth_id={} title={}", &self.art_id, &self.auth_id, &self.title)
    }
}

/// This struct corresponds to one article paragraph
#[derive(Serialize, Deserialize)]
pub struct ArticlePara {
    pub art_id: i32, 
    pub apara_id: i32,
    pub md: String,         // Markdown for this article
    pub html: String,       // HTML = Markdown + NLP enrichment 
}

impl Xtchable for ArticlePara {
    fn state_string(&self) -> String {
        format!("apara_id={} art_id={} md={} html={}", &self.apara_id, &self.art_id, &self.md, &self.html)
    }
}


#[derive(Serialize, Deserialize)]
pub struct YoutubeChannel {
    pub chan_id: i32,   // the primary key for this channel
    pub url: String,    // typically c/ChannelName etc.
    pub name: String,   // The name of this channel 
}

impl Xtchable for YoutubeChannel {
    fn state_string(&self) -> String {
        format!("chan_id={} name={} url={}", &self.chan_id, &self.name, &self.url)
    }
}
