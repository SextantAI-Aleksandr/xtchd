//! This module contains a struct corresponding to one row for each of the main tables in xtchr
//! Serialization and deserialization are implemented to enable passing structs over http

use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde_json;
use tokio_postgres;
use pachydurable::{autocomplete::{AutoComp, WhoWhatWhere}, fulltext::FullText};
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

impl AutoComp<i32> for Article {
    fn query_autocomp() -> &'static str {
        "SELECT art_id, title 
        FROM articles 
        WHERE ac @@ to_tsquery('simple', $1)
        ORDER BY LENGTH(title) ASC 
        LIMIT 10"
    }
    fn rowfunc_autocomp(row: &tokio_postgres::Row) -> WhoWhatWhere<i32>  {
        let data_type = "Author";
        let art_id: i32 = row.get(0);
        let title: String = row.get(1);
        WhoWhatWhere{data_type, pk: art_id, name: title}
    }
}

/// This struct corresponds to one article paragraph
#[derive(Serialize, Deserialize)]
pub struct ArticlePara {
    pub art_id: i32, 
    pub apara_id: i32,
    pub md: String,         // Markdown for this article
}

impl Xtchable for ArticlePara {
    fn state_string(&self) -> String {
        format!("apara_id={} art_id={} md={}", &self.apara_id, &self.art_id, &self.md)
    }
}


impl FullText for ArticlePara {
    fn query_fulltext() ->  & 'static str {
        "SELECT art_id, apara_id, md
        FROM article_para
        WHERE ts @@ to_tsquery('english', $1)
        LIMIT 20;"
    }
    fn rowfunc_fulltext(row: &tokio_postgres::Row) -> Self {
        let art_id: i32 = row.get(0);
        let apara_id: i32 = row.get(1);
        let md: String = row.get(2);
        ArticlePara{art_id, apara_id, md}
    }
}


impl<'a> tokio_postgres::types::FromSql<'a> for ArticlePara {
    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let apara: ArticlePara = serde_json::from_slice(raw)?;
        Ok(apara)
    }

    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
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

impl AutoComp<i32> for YoutubeChannel {
    fn query_autocomp() -> &'static str {
        "SELECT chan_id, name 
        FROM youtube_channels 
        WHERE ac @@ to_tsquery('simple', $1)
        ORDER BY LENGTH(name) ASC 
        LIMIT 10"
    }
    fn rowfunc_autocomp(row: &tokio_postgres::Row) -> WhoWhatWhere<i32>  {
        let data_type = "YoutubeChannel";
        let chan_id: i32 = row.get(0);
        let name: String = row.get(1);
        WhoWhatWhere{data_type, pk: chan_id, name}
    }
}

#[derive(Serialize, Deserialize)]
pub struct YoutubeVideo {
    pub chan_id: i32,       // The id for the channel,
    pub vid_id: i32,        // The id for this video 
    pub vid_pk: String,     // the CHAR(11) url/id for this video 
    pub title: String,
    pub date_uploaded: NaiveDate,
}

impl Xtchable for YoutubeVideo {
    fn state_string(&self) -> String {
        format!("vid_id={} vid_pk={} chan_id={} title={}", &self.vid_id, &self.vid_pk, &self.chan_id, &self.title)
    }
}


impl AutoComp<String> for YoutubeVideo {
    fn query_autocomp() ->  &'static str {
        "SELECT vid_pk, title 
        FROM youtube_vidoes
        WHERE ac @@ to_tsquery('simple', $1)
        ORDER BY LENGTH(title) DESC
        LIMIT 10"
    }

    fn rowfunc_autocomp(row: &postgres::Row) -> WhoWhatWhere<String> {
        let data_type = "YoutubeVideo";
        let vid_pk: String = row.get(0);
        let title: String = row.get(1);
        WhoWhatWhere{data_type, pk: vid_pk, name: title}
    }
}