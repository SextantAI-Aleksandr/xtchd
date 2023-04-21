//! This module contains a struct corresponding to one row for each of the main tables in xtchr
//! Serialization and deserialization are implemented to enable passing structs over http

use serde::{Serialize, Deserialize};
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