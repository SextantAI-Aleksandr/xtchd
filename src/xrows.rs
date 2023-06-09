//! This module contains a struct corresponding to one row for each of the main tables in xtchd
//! Where cryptographic verification of each row is implemented, hence the name "xrows" for "xtchd rows".
//! Structs implement deserialization to aid in implementing the tokio_postgres::types::FromSql trait
//! and implement serialization to aid in passing them over http. 

use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde_json;
use tokio_postgres;
use pachydurable::{autocomplete::{AutoComp, WhoWhatWhere}, fulltext::FullText, redis::{CachedAutoComp, PreWarmDepth}};
use crate::integrity::{Xtchable, nonefmt};


#[derive(Serialize, Deserialize)]
pub struct Author {
    pub auth_id: i32,   // the primary key for this author
    pub name: String,
}

impl Xtchable for Author {
    fn state_string(&self) -> String {
        format!("auth_id={} name={}", &self.auth_id, &self.name)
    }
    fn dtype() -> &'static str {
        "Author"
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
        let data_type = <Author as Xtchable>::dtype().to_string();
        let auth_id: i32 = row.get(0);
        let name: String = row.get(1);
        WhoWhatWhere{data_type, pk: auth_id, name}
    }
}

impl CachedAutoComp<i32> for Author {
    fn dtype() -> &'static str {
        <Author as Xtchable>::dtype()
    }
    fn seconds_expiry() -> usize {
        // one month may seem like a long time, but authors change seldom, and you can always call pachydurable::redis::warm_the_cache()
        (60*60*24*31) as usize
    }
    fn prewarm_depth() -> PreWarmDepth {
        PreWarmDepth::Char2
    }
}


#[derive(Serialize, Deserialize)]
pub struct Article {
    pub art_id: i32,    // the primary key for this article
    pub auth_id: i32,   // the primary key fo r the author
    pub title: String,
    pub image_file: Option<String>,
}

impl Xtchable for Article {
    fn state_string(&self) -> String {
        format!("art_id={} auth_id={} title={}", &self.art_id, &self.auth_id, &self.title)
    }
    fn dtype() -> &'static str {
        "Article"
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
        let data_type = <Article as Xtchable>::dtype().to_string();
        let art_id: i32 = row.get(0);
        let title: String = row.get(1);
        WhoWhatWhere{data_type, pk: art_id, name: title}
    }
}

impl CachedAutoComp<i32> for Article {
    fn dtype() -> &'static str {
        <Article as Xtchable>::dtype()
    }
    fn seconds_expiry() -> usize {
        // a couple days may seem like a long time, but you can always call pachydurable::redis::warm_the_cache()
        (60*60*24*2) as usize
    }
    fn prewarm_depth() -> PreWarmDepth {
        PreWarmDepth::Char3
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
    fn dtype() -> &'static str {
        "ArticlePara"
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
    fn dtype() -> &'static str {
        "YoutubeChannel"
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
        let data_type = <YoutubeChannel as Xtchable>::dtype().to_string();
        let chan_id: i32 = row.get(0);
        let name: String = row.get(1);
        WhoWhatWhere{data_type, pk: chan_id, name}
    }
}

impl CachedAutoComp<i32> for YoutubeChannel {
    fn dtype() -> &'static str {
        <YoutubeChannel as Xtchable>::dtype()
    }
    fn seconds_expiry() -> usize {
        // one month may seem like a long time, but channels are not added very quickly, and you can always call pachydurable::redis::warm_the_cache()
        (60*60*24*31) as usize
    }
    fn prewarm_depth() -> PreWarmDepth {
        PreWarmDepth::Char2
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
    fn dtype() -> &'static str {
        "YoutubeVideo"
    }
}


impl AutoComp<String> for YoutubeVideo {
    fn query_autocomp() ->  &'static str {
        "SELECT vid_pk, title 
        FROM youtube_videos
        WHERE ac @@ to_tsquery('simple', $1)
        ORDER BY LENGTH(title) DESC
        LIMIT 10"
    }

    fn rowfunc_autocomp(row: &postgres::Row) -> WhoWhatWhere<String> {
        let data_type = <YoutubeVideo as Xtchable>::dtype().to_string();
        let vid_pk: String = row.get(0);
        let title: String = row.get(1);
        WhoWhatWhere{data_type, pk: vid_pk, name: title}
    }
}

impl CachedAutoComp<String> for YoutubeVideo {
    fn dtype() -> &'static str {
        <YoutubeVideo as Xtchable>::dtype()
    }
    fn seconds_expiry() -> usize {
        (60*60*24) as usize
    }
    fn prewarm_depth() -> PreWarmDepth {
        PreWarmDepth::Char3
    }
}




/// Images can be saved to either the images table (where they are immutable and have a sha256 value calculated)
/// or the images_mut table(where they are mutable and have not sha256 calculated).
/// In either case, they are provided as both a full image and a thumbnail, with a 
/// src/caption value and optional URL where they came from 
#[derive(Serialize, Deserialize)]
pub struct ImagePair {
    /// base64 encoded full image: i.e. "<img src="data:image/png;base64, iVBORw0KGgoA..." etc
    pub src_full: String,
    /// base64 encoded thumbnail: i.e. "<img src="data:image/png;base64, iVBORw0KGgoA..." etc
    pub src_thmb: String,
    /// caption / alt text for accessability
    pub alt: String,
    /// optional URL for screenshots and downloads
    pub url: Option<String>,
    /// optional 5-character primary key for an archive made with archive.is
    pub archive: Option<String>,
}


/// MutableImages are typically used for article thumbnails:
/// i.e. they are a bit arbitrary and only need to roughly indicate the content of the article
#[derive(Deserialize)]
pub struct MutableImage {
    /// a CHAR(16) nanoID, no need to be sequential
    pub id: String,
    /// The image pair being saved 
    pub pair: ImagePair
}


/// An ImmutableImage is used for images within an article. The assumption is that 
/// the image "matters" and needs to "prove a point" (in contrast to MutableImages),
/// Hence the Xtchable trait is implemented so that the integrity of an ImmutableImage can be verified 
pub struct ImmutableImage {
    /// an image_id provided by the database 
    pub img_id: i32,
    /// the image pair being saved 
    pub pair: ImagePair,
}


impl Xtchable for ImmutableImage {
    fn state_string(&self) -> String {
        format!("img_id={} src_full={} src_thmb={} alt={} url={} archive={}",
            &self.img_id, &self.pair.src_full, &self.pair.src_thmb, &self.pair.alt, nonefmt(&self.pair.url), nonefmt(&self.pair.archive))
    }
    fn dtype() -> &'static str {
        "Image"
    }

}


/// This struct is useful for autocompletion of results for immutable images 
#[derive(Serialize, Deserialize)]
pub struct ImageThumbnail {
    pub img_id: i32,
    pub src_thmb: String,
}


impl AutoComp<ImageThumbnail> for ImmutableImage {
    fn query_autocomp() ->  &'static str {
        "SELECT img_id, alt, src_thmb
        FROM images
        WHERE ts @@ to_tsquery('english', $1)
        ORDER BY LENGTH(alt) ASC 
        LIMIT 10;"
    }

    fn rowfunc_autocomp(row: &tokio_postgres::Row) -> WhoWhatWhere<ImageThumbnail> {
        let data_type = <ImmutableImage as Xtchable>::dtype().to_string();
        let img_id: i32 = row.get(0);
        let name: String = row.get(1);
        let src_thmb: String = row.get(2);
        let pk = ImageThumbnail{img_id, src_thmb};
        WhoWhatWhere{data_type, pk, name}
    }
}


impl CachedAutoComp<ImageThumbnail> for ImmutableImage {
    fn dtype() -> &'static str {
        <ImmutableImage as Xtchable>::dtype()
    }
    fn seconds_expiry() -> usize {
        (60*60*24*3) as usize
    }
    fn prewarm_depth() -> PreWarmDepth {
        PreWarmDepth::Char2 // remember the image is large: don't copy it across too many keys 
    }
}




/// For most rendering purposes, image thumbnails will be used (instead of the full image)
/// Therefore, searching for images by caption is implemented using the Fulltext trait on the thumbnail 
pub struct Thumbnail {
    pub img_id: i32,
    /// base64 encoded image: i.e. "<img src="data:image/png;base64, iVBORw0KGgoA..." etc
    pub thumb_src: String,
    /// caption / alt text for accessability
    pub alt: String,
}


impl FullText for Thumbnail {
    fn query_fulltext() -> &'static str {
        "SELECT img_id, thumb_src, atl
        FROM images
        WHERE ts @@ to_tsquery('english', $1)
        LIMIT 20;"
    }

    fn rowfunc_fulltext(row: &tokio_postgres::Row) -> Self {
        let img_id: i32 = row.get(0);
        let thumb_src = row.get(1);
        let alt: String = row.get(2);
        Thumbnail{img_id, thumb_src, alt}
    }

}



/// The RefFrom struct represents (1) the article from which a reference is made, with optional paragraph identifier,
/// and (2) a brief comment on why this reference is relevant or what it shows 
#[derive(Serialize, Deserialize)]
pub struct RefFrom {
    /// The id of the article making the reference
    pub art_id: i32,
    /// optional paragraph specifier if the reference is from one specific paragraph
    pub apara_id: Option<i32>,
    /// a brief comment on why this reference is relevant or what the reference shows
    pub comment: String,
}


/// This struct captures a reference from one article (or a paragraph therein)
/// to another article (or a paragraph therein), with a brief comment as to why
/// this reference is relevant or what it shows
#[derive(Serialize, Deserialize)]
pub struct ArticleRefArticle {
    /// The primary key for this reference
    pub aref_id: i32,
    /// The article making the reference and why it was made
    pub rf: RefFrom,
    /// The id of the article being referenced 
    pub refs_art: i32,
    /// optional paragraph specifier if the reference is to one specific paragraph 
    pub refs_para: Option<i32>,
}

impl ArticleRefArticle {
    pub fn from_req(req: ArticleRefArticleReq, aref_id: i32) -> Self {
        let rf = req.rf;
        let refs_art = req.refs_art;
        let refs_para = req.refs_para;
        ArticleRefArticle{aref_id, rf, refs_art, refs_para}
    }
}


impl Xtchable for ArticleRefArticle {
    fn state_string(&self) -> String {
        format!("aref_id={} from_art={} from_para={} refs_art={} refs_para={} comment={}",
            &self.aref_id, &self.rf.art_id, nonefmt(&self.rf.apara_id), &self.refs_art, nonefmt(&self.refs_para), &self.rf.comment)
    }
    fn dtype() -> &'static str {
        "ArticleRefArticle"
    }
}


/// This struct is the same as ArticleRefArticle but without the aref_id which needs 
/// to be generated.  
/// This struct is passed via http when authors are adding references to an article.
#[derive(Deserialize)]
pub struct ArticleRefArticleReq {
    /// The article making the reference and why it was made
    pub rf: RefFrom,
    /// The id of the article being referenced 
    pub refs_art: i32,
    /// optional paragraph specifier if the reference is to one specific paragraph 
    pub refs_para: Option<i32>,
}


/// This struct captures a reference from one article (or a paragraph therein)
/// to a video (with optional timestamp), with a brief comment as to why
/// this reference is relevant or what it shows 
#[derive(Deserialize)]
pub struct ArticleRefVideo {
    /// The primary key for this reference
    pub vref_id: i32,
    /// The article making the reference and why it was made
    pub rf: RefFrom,
    /// The video being referenced 
    pub vid_pk: String,
    /// Optional timestamp (in seconds) within the video 
    pub sec_req: Option<i16>,
}


impl ArticleRefVideo {
    pub fn from_req(req: ArticleRefVideoReq, vref_id: i32) -> Self {
        let rf = req.rf;
        let vid_pk = req.vid_pk;
        let sec_req = req.sec_req;
        ArticleRefVideo{vref_id, rf, vid_pk, sec_req}
    }
}

impl Xtchable for ArticleRefVideo {
    fn state_string(&self) -> String {
        format!("vref_id={} art_id={} apara_id={} vid_pk={} sec_req={} comment={}",
            &self.vref_id, &self.rf.art_id, nonefmt(&self.rf.apara_id), &self.vid_pk, nonefmt(&self.sec_req), &self.rf.comment)
    }
    fn dtype() -> &'static str {
        "ArticleRefVideo"
    }
}

/// This struct is the same as ArticleRefVideo but without the vref_id
/// which needs to be generated  
/// This struct is passed via http when authors are adding references to an article.
#[derive(Deserialize)]
pub struct ArticleRefVideoReq {
    /// The article making the reference and why it was made
    pub rf: RefFrom,
    /// The video being referenced 
    pub vid_pk: String,
    /// Optional timestamp (in seconds) within the video 
    pub sec_req: Option<i16>,
}




/// This struct captures a reference from one article (or a paragraph therein)
/// to an image, with a brief comment as to why
/// this reference is relevant or what it shows 
#[derive(Deserialize)]
pub struct ArticleRefImage {
    /// The primary key for this reference
    pub iref_id: i32,
    /// The article making the reference and why it was made
    pub rf: RefFrom,
    /// The id for the video being referenced
    pub img_id: i32,
}

impl ArticleRefImage {
    pub fn from_req(req: ArticleRefImageReq, iref_id: i32) -> Self {
        let rf = req.rf;
        let img_id = req.img_id;
        ArticleRefImage{iref_id, rf, img_id}
    }
}


impl Xtchable for ArticleRefImage {
    fn state_string(&self) -> String {
        format!("iref_id={} art_id={} apara_id={} img_id={} comment={}",
            &self.iref_id, &self.rf.art_id, nonefmt(&self.rf.apara_id), &self.img_id, &self.rf.comment)
    }
    fn dtype() -> &'static str {
        "ArticleRefImage"
    }
}

/// This struct is the same as ArticleRefImage but without the iref_id
/// which needs to be generated.  
/// This struct is passed via http when authors are adding references to an article.
#[derive(Deserialize)]
pub struct ArticleRefImageReq {
    /// The article making the reference and why it was made
    pub rf: RefFrom,
    /// The id for the video being referenced
    pub img_id: i32,
}
