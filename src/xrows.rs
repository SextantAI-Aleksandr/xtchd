//! This module contains a struct corresponding to one row for each of the main tables in xtchd
//! Where cryptographic verification of each row is implemented, hence the name "xrows" for "xtchd rows".
//! Structs implement deserialization to aid in implementing the tokio_postgres::types::FromSql trait
//! and implement serialization to aid in passing them over http. 

use std::fmt;
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde_json;
use tokio_postgres;
use pachydurable::{autocomplete::{AutoComp, WhoWhatWhere}, fulltext::FullText, redis::{CachedAutoComp, PreWarmDepth}};
use crate::integrity::{Xtchable, nonefmt};





/// The PageSrc enum gives the various sources that can be used for a page 
/// Recall that the ArticlePage is a struct designed to be written but not read- 
/// This is reflected in the fact that Webpage, TwitterX, and YouTube sourcs all get lumped into
/// the WpTxYt struct which simply contains an img_id. 
/// On read, the src_type is inferred from the images table 
pub enum PageSrc {
    /// The page is the arthors's opinion, perhaps a preamble or conclusion.
    /// It contains a string referencing an image_file, typically a 'splash' page for the article 
    Author(String),
    /// If the source is a prior Xtchd article the source is the article id  
    Xtchd(i32),
    /// All other sources (which is most of them) are captured in the WpTxYt struct which
    /// references an img_id- see comment above to the PageSrc struct 
    WpTxYt(i32),
}

impl PageSrc {
    /// This page gives the values for these columns in the article_pages_immut table:
    ///                          (    img_id,     image_file,    refs_art_id)
    pub fn src_columns(&self) -> (Option<i32>, Option<String>, Option<i32>) {
        let (mut img_id, mut image_file, mut refs_art_id) = (None, None, None);
        match &self {
            // the image_file might be something like wiki/800px-Merkava-Mk4m-whiteback01.jpg
            PageSrc::Author(val) => { image_file = Some(val.to_owned()); },
            // refs_art_id is the id for another xtchd article
            PageSrc::Xtchd(val) => { refs_art_id = Some(val.to_owned()); },
            PageSrc::WpTxYt(val) => { img_id = Some(val.to_owned()); },
        }
        (img_id, image_file, refs_art_id)
    }
}


/// The ArticlePage struct captures the text and image for one page of one article 
pub struct ArticlePage {
    /// the id for the article this page is associated with 
    pub art_id: i32, 
    /// the globally unique id for this page 
    pub apage_id: i32,
    /// Paragraphs of plaintext. 
    /// Why no HTML??? You don't need to link to anything- the page is the link as captured via the
    /// .source property 
    pub paragraphs: Vec<String>,
    /// The source descibes where the information from the page was taken from 
    pub source: PageSrc,
}

impl Xtchable for ArticlePage {
    fn state_string(&self) -> String {
        let (img_id, image_file, refs_art_id) = &self.source.src_columns();
        format!("art_id={} apage_id={} paragraphs={} img_id={} image_file={} refs_art_id={}",
            &self.art_id, &self.apage_id, &self.paragraphs.join(" | "), nonefmt(&img_id), nonefmt(&image_file), nonefmt(&refs_art_id))
    }

    fn dtype() -> &'static str {
        "ArticlePage"
    }
}

impl ArticlePage {
    pub fn prior_id(&self) -> i32 {
        self.apage_id - 1
    }
}


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
        AND LOWER(name) LIKE '%' || LOWER($2) || '%'
        ORDER BY LENGTH(name) ASC 
        LIMIT 10;"
    }
    fn rowfunc_autocomp(row: &tokio_postgres::Row) -> WhoWhatWhere<i32> {
        let data_type = "author".to_string();
        let auth_id: i32 = row.get(0);
        let name: String = row.get(1);
        WhoWhatWhere{data_type, pk: auth_id, name}
    }
}

impl CachedAutoComp<i32> for Author {
    fn dtype() -> &'static str {
        "author"
    }
    fn seconds_expiry() -> usize {
        // one month may seem like a long time, but authors change seldom, and you can always call pachydurable::redis::warm_the_cache()
        (60*60*24*31) as usize
    }
    fn prewarm_depth() -> PreWarmDepth {
        PreWarmDepth::Char2
    }
}


/// The ArticleTitle shows the title of an article
#[derive(Serialize, Deserialize)]
pub struct ArticleTitle {
    // the primary key for this article
    pub art_id: i32,    
    // the primary key for the author
    pub auth_id: i32,   
    // the title of the article 
    pub title: String,
}

impl Xtchable for ArticleTitle {
    fn state_string(&self) -> String {
        format!("art_id={} auth_id={} title={}", &self.art_id, &self.auth_id, &self.title)
    }
    fn dtype() -> &'static str {
        "ArticleTitle"
    }
}


#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize)]
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
        "SELECT img_id, CONCAT(COALESCE(archive,''), ' ', alt) AS alt, src_thmb
        FROM images_immut
        WHERE ac @@ to_tsquery('simple', $1) AND CONCAT(COALESCE(archive,''), ' ', alt) ILIKE '%' || $2 || '%'
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
        (10) as usize // 10 seconds as images are added
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
        FROM images_immut
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



#[derive(Serialize, Deserialize)]
pub enum Graph3dEdge {
    Authored,
    References,
    Mentions,
    IncludesPara
}

impl Graph3dEdge {
    pub fn variant_str(&self) -> &'static str {
        match self {
            // recall that the to_node_json method on the tangentially::Node struct
            // just serializes things. Ensure you DON'T change these names so they match the form
            // given by serialization
            Graph3dEdge::Authored => "Authored",
            Graph3dEdge::References => "References",
            Graph3dEdge::Mentions => "Mentions",
            Graph3dEdge::IncludesPara => "IncludesPara",
        }
    }
}



impl fmt::Display for Graph3dEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant_str())
    }
}


/// Each node should include the table propery indicating whence it came:
/// Expressing that as an enum ensures it matches an exhaustive list 
#[derive(Serialize, Deserialize, Clone)]
pub enum Graph3dNode {
    Author,
    Article, 
    Channel,
    Topic,
    Video,
    Image,
}


impl Graph3dNode {
    pub fn variant_str(&self) -> &'static str {
        match self {
            // recall that the to_node_json method on the tangentially::Node struct
            // just serializes things. Ensure you DON'T change these names so they match the form
            // given by serialization
            Graph3dNode::Author => "Author",
            Graph3dNode::Article => "Article",
            Graph3dNode::Channel => "Channel",
            Graph3dNode::Topic => "Topic",
            Graph3dNode::Video => "Video",
            Graph3dNode::Image => "Image",
        }
    }
}


impl fmt::Display for Graph3dNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant_str())
    }
}
