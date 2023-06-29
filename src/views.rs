use std::{fmt::{self, Display}, vec::Vec};
use serde::{Serialize, Deserialize};
use serde_json;
use tokio_postgres;
use pachydurable::{autocomplete::{AutoComp, WhoWhatWhere}, fulltext::FullText, redis::{Cacheable, CachedAutoComp, PreWarmDepth}};
use tangentially::fd3d::{Node, ToNode, ToNodeJSON, Edge, ToEdge, ToEdgeJSON, Graph, ToGraph};
use crate::{integrity::{XtchdContent, XtchdSQL}, xrows::{self, Graph3dEdge, Graph3dNode}};


/// The ArticleText struct contains the auth, title, and paragraph texts for an article.
/// Compare to the EnrichedArticle struct, which also contains references and extracted topics
#[derive(Serialize)]
pub struct ArticleText {
    /// The author + hash integrity information
    pub author: XtchdContent<xrows::Author>,
    /// The article id/title + hash integrity information
    pub article: XtchdContent<xrows::Article>,
    /// The text of each paragraph + hash integrity information
    pub paragraphs: Vec<XtchdContent<xrows::ArticlePara>>,
}



/// A topic represents a person, place, or string that can be extracted from text
/// using Natural Language Processing (NLP)
#[derive(Serialize, Deserialize)]
pub struct Topic {
    /// The primary key for this topic
    pub tkey: String, 
    /// The part-of-speech: i.e. 'PER' for person, 'NCK' for noun chunck etc. See schema.sql/nlp_topic_pos 
    pub pos: String,
    /// The name, i.e. this topic as a string
    pub name: String,
    /// The frequency with which this topic has been extracted 
    pub count: i16,  
}




impl AutoComp<String> for Topic {
    fn query_autocomp() ->  &'static str {
        "SELECT tkey, name
        FROM nlp_topics 
        WHERE ac @@ to_tsquery('simple', $1)
        ORDER BY count DESC 
        LIMIT 10 "
    }

    fn rowfunc_autocomp(row: &pachydurable::connect::Row) -> WhoWhatWhere<String> {
        let data_type = String::from("Topic");
        let tkey: String = row.get(0);
        let name: String = row.get(1);
        WhoWhatWhere { data_type, pk: tkey, name }
    }
}


impl CachedAutoComp<String> for Topic {
    fn dtype() -> &'static str {
        "Topic"
    }
    fn seconds_expiry() -> usize {
        (60*60*24) as usize
    }
    fn prewarm_depth() -> PreWarmDepth {
        PreWarmDepth::Char3
    }
}


#[derive(Serialize, Deserialize)]
pub struct TopicProps {
    /// The part of speech
    pub pos: String,
    /// the frequency of this topic 
    pub ct: i16,   
}


impl ToNode<Graph3dNode, String, TopicProps> for Topic {
    fn node_variant(&self) -> Graph3dNode {
        Graph3dNode::Topic
    }
    fn node_pk(&self) -> String {
        self.tkey.clone()
    }
    fn node_name(&self) -> String {
        self.name.clone()
    }
    fn node_props(&self) -> TopicProps {
        let pos = self.pos.clone();
        let ct = self.count;
        TopicProps{pos, ct}
    }   
}

impl ToNodeJSON<Graph3dNode, String, TopicProps> for Topic {}


impl<'a> tokio_postgres::types::FromSql<'a> for Topic {
    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let topic: Topic = serde_json::from_slice(raw)?;
        Ok(topic)
    }
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}


/// When an article/paragraph includes a reference to an article, 
/// the source is obvious from the article associated with the reference
#[derive(Serialize, Deserialize)]
pub struct ArticleRef {
    /// The primary key for this reference
    pub aref_id: i32,
    /// The id for the article being referenced
    pub art_id: i32,
    /// Optional paragraph id for a paragraph within that article
    pub apara_id: Option<i32>,
    /// The title of the article being referenced 
    pub title: String,
    /// A comment on why the reference is relevant or what it shows
    pub comment: String,
}




impl ToNode<Graph3dNode, i32, xrows::ArticleProps> for ArticleRef {
    fn node_variant(&self) -> Graph3dNode {
        Graph3dNode::Article
    }
    fn node_pk(&self) -> i32 {
        self.art_id
    }
    fn node_name(&self) -> String {
        self.title.clone()
    }
    fn node_props(&self) -> xrows::ArticleProps {
        let auth_id: Option<i32> = None;
        xrows::ArticleProps{auth_id}
    }
}

impl ToNodeJSON<Graph3dNode, i32, xrows::ArticleProps> for ArticleRef {}


/// When an article/paragraph includes a reference to a video,
/// The source is obvious from the article/paragraph making the reference 
#[derive(Serialize, Deserialize)]

pub struct VideoRef {
    /// The primary key for this reference
    pub vref_id: i32,
    /// The CHAR(11) primary key for this video
    pub vid_pk: String,
    /// Optional timestamp within the video 
    pub sec_req: Option<i32>,
    /// The title of the video being referenc
    pub title: String,
    /// A comment on why the reference is relevant or what it shows
    pub comment: String,
}


/// This struct captures properties associated with a video node
#[derive(Serialize, Deserialize)]
pub struct VideoProps {
    pub youtube_url: String,
}

impl ToNode<Graph3dNode, String, VideoProps> for VideoRef {
    fn node_variant(&self) -> Graph3dNode {
        Graph3dNode::Video
    }
    fn node_pk(&self) -> String {
        self.vid_pk.clone()
    }
    fn node_name(&self) -> String {
        self.title.clone()
    }
    fn node_props(&self) -> VideoProps {
        let youtube_url = format!("https://www.youtube.com/watch?v={}", &self.vid_pk);
        VideoProps{youtube_url}
    }
}

impl ToNodeJSON<Graph3dNode, String, VideoProps> for VideoRef {}


/// When an article/paragraph includes a reference to an image,
/// The source is obvious from the article/paragraph making the reference 
#[derive(Serialize, Deserialize, Clone)]
pub struct ImageRef {
    /// the primary key for this reference
    pub iref_id: i32,
    /// the image being referenced
    pub img_id: i32,
    /// A thumbnail with the image, encoded as base64
    pub src_thmb: String,
    /// alternate text / caption
    pub alt: String,
    /// optional url the image was captured / downloaded from
    pub url: Option<String>, 
    /// A comment on why the reference is relevant or what it shows
    pub comment: String,   
}


impl ToNode<Graph3dNode, i32, ImageRef> for ImageRef {
    // Allow an image ref to be interpreted as a node for that image 
    fn node_variant(&self) -> Graph3dNode {
        Graph3dNode::Image
    }
    fn node_pk(&self) -> i32 {
        self.img_id
    }
    fn node_name(&self) -> String {
        self.alt.clone()
    }
    fn node_props(&self) -> ImageRef {
        self.clone()
    }
}

impl ToNodeJSON<Graph3dNode, i32, ImageRef> for ImageRef {}


/// This struct represents a group of references to articles, videos, and images
#[derive(Serialize, Deserialize)]
pub struct References {
    pub articles: Vec<ArticleRef>,
    pub videos: Vec<VideoRef>,
    pub images: Vec<ImageRef>,
}

impl<'a> tokio_postgres::types::FromSql<'a> for References {
    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let refs: References = serde_json::from_slice(raw)?;
        Ok(refs)
    }
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}


/// An enriched paragraph includes the paragraph content
/// as well as references and topics extracted using NLP
#[derive(Serialize, Deserialize)]
pub struct EnrichedPara {
    pub para: XtchdContent<xrows::ArticlePara>,
    pub refs: References,
    pub topics: Vec<Topic>,
}


/// This struct is needed because you can't deserialize the XtchdContent directly
/// bec.ause the hcl.string_to_hash is never stored
#[derive(Deserialize)]
struct EnrichedParaSQL {
    para: XtchdSQL<xrows::ArticlePara>,
    refs: References, 
    topics: Vec<Topic>
}

impl<'a> tokio_postgres::types::FromSql<'a> for EnrichedPara {
    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let epsql: EnrichedParaSQL = serde_json::from_slice(raw)?;
        let ep = EnrichedPara{para: XtchdContent::from_sql(epsql.para), refs: epsql.refs, topics: epsql.topics};
        Ok(ep)
    }
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}



/// An enriched article includes identifying information for the article title and author along with hash integrity information
/// As well enriched paragraphs and references to the article as a whole
#[derive(Serialize, Deserialize)]
pub struct EnrichedArticle {
    /// The title and identification of the author of the article
    pub author: XtchdContent<xrows::Author>,
    /// The title and identification of the article
    pub article: XtchdContent<xrows::Article>,
    /// References made by the article as a whole, not from any one specific paragraph
    pub refs: References,
    /// Each of the paragraphs from the article, enriched with references and with topics extracted using NLP
    pub paragraphs: Vec<EnrichedPara>,
    /// Arrticles that reference this article
    pub refd_by: Vec<RefdByArticle>,
}


impl Cacheable for EnrichedArticle {

    fn key_prefix() ->  &'static str {
        "enr_art"
    }

    fn seconds_expiry() -> usize {
        (60*60*24) as usize
    }

    fn query() ->  &'static str {
        "SELECT author, article, refs, paragraphs, refd_by FROM enriched_article_fields WHERE art_id = $1"
    }


    fn from_row(row: &tokio_postgres::Row) -> Self {
        let author: XtchdContent<xrows::Author> = row.get(0);
        let article: XtchdContent<xrows::Article> = row.get(1);
        let refs: References = row.get(2);
        let paragraphs: Vec<EnrichedPara> = row.get(3);
        let refd_by: Vec<RefdByArticle> = row.get(4);
        EnrichedArticle{author, article, refs, paragraphs, refd_by}
        
    }
}



#[derive(Serialize, Deserialize)]
pub struct NameId {
    id: i32,
    name: String,
}

impl<'a> tokio_postgres::types::FromSql<'a> for NameId {
    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let name_id: NameId = serde_json::from_slice(raw)?;
        Ok(name_id)
    }
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}


/// This struct gives details for one author
/// It is typically returned when the user clicks on an author for more information
#[derive(Serialize)]
pub struct AuthorDetail {
    pub author: XtchdContent<xrows::Author>,
    pub articles: Vec<NameId>,
}




impl ToGraph for EnrichedArticle {
    fn mut_graph(&self, graph: &mut Graph) -> Result<(), serde_json::Error> {
        graph.add_node_from(&self.article.content)?;
        for aref in &self.refs.articles {
            graph.source_edge_target(&self.article.content, aref, Graph3dEdge::References, ())?;
        }
        for vref in &self.refs.videos {
            graph.source_edge_target(&self.article.content, vref, Graph3dEdge::References, ())?;
        }
        for iref in &self.refs.images {
            graph.source_edge_target(&self.article.content, iref, Graph3dEdge::References, ())?;
        }
        for para in &self.paragraphs {
            for aref in &para.refs.articles {
                graph.source_edge_target(&self.article.content, aref, Graph3dEdge::References, ())?;
            }
            for vref in &para.refs.videos {
                graph.source_edge_target(&self.article.content, vref, Graph3dEdge::References, ())?;
            }
            for iref in &para.refs.images {
                graph.source_edge_target(&self.article.content, iref, Graph3dEdge::References, ())?;
            }
        }
        Ok(())
    }
}


/// When a user clicks on an image, this struct is returned to provide more detail on the image.
/// including the fullsize image, proof of immutability, and the articles referencing the images
#[derive(Serialize, Deserialize)]
pub struct EnrichedImage {
    /// the fullsize + thumbnail images with proof of immutability
    pub image: XtchdContent<xrows::ImmutableImage>,
    /// The articles that reference this image 
    pub refd_by: Vec<RefdByArticle>
}


impl Cacheable for EnrichedImage {

    fn key_prefix() ->  &'static str {
        "image"
    }

    fn seconds_expiry() -> usize {
        (60*60*24*7) as usize // one week expiry
    }

    fn query() ->  &'static str {
        "SELECT image, refd_by FROM enriched_image_fields WHERE img_id = $1"
    }

    fn from_row(row: &tokio_postgres::Row) -> Self {
        let image: XtchdContent<xrows::ImmutableImage> = row.get(0);
        let refd_by: Vec<RefdByArticle> = row.get(1);
        EnrichedImage{image, refd_by}
    }
}


impl ToGraph for EnrichedImage {
    fn mut_graph(&self, graph: &mut Graph) -> Result<(), serde_json::Error> {
        for rba in &self.refd_by {
            graph.source_edge_target(rba, &self.image.content, Graph3dEdge::References, ())?;
        }
        Ok(())
    }
}


/// When a user clicks on a youtube video, this struct is returned to provide more detail 
/// including the video, the channel, and the articles referencing the video 
#[derive(Serialize, Deserialize)]
pub struct EnrichedVideo {
    pub channel: xrows::YoutubeChannel,
    pub video: xrows::YoutubeVideo,
    pub refd_by: Vec<RefdByArticle>,
}

impl Cacheable for EnrichedVideo {

    fn key_prefix() ->  &'static str {
        "video"
    }

    fn seconds_expiry() -> usize {
        (60*60*24*7) as usize // one week expiry
    }

    fn query() ->  &'static str {
        "SELECT channel, video, refd_by FROM enriched_video_fields WHERE vid_pk = $1"
    }

    fn from_row(row: &tokio_postgres::Row) -> Self {
        let channel: xrows::YoutubeChannel = row.get(0);
        let video: xrows::YoutubeVideo = row.get(1);
        let refd_by: Vec<RefdByArticle> = row.get(2);
        EnrichedVideo{channel, video, refd_by}
    }
}


impl ToGraph for EnrichedVideo {
    fn mut_graph(&self, graph: &mut Graph) -> Result<(), serde_json::Error> {
        for rba in &self.refd_by {
            // link each article referencing the video
            graph.source_edge_target(rba, &self.video, Graph3dEdge::References, ())?;
        }
        // link the channel to the video
        graph.source_edge_target(&self.channel, &self.video, Graph3dEdge::Authored, ())?;
        Ok(())
    }
}




/// When something is referenced by an article, this struct captures that
#[derive(Serialize, Deserialize)]
pub struct RefdByArticle {
    /// The primary key for this reference
    /// This will be an aref_id, vref_id, or iref_id depending on the case
    pub ref_id: i32,
    /// The id for the article making the reference
    pub art_id: i32,
    /// Optional paragraph id for a paragraph within that article
    pub apara_id: Option<i32>,
    /// The title of the article making the referenced 
    pub title: String,
    /// A comment on why the reference is relevant or what it shows
    pub comment: String,
}


impl ToNode<Graph3dNode, i32, xrows::ArticleProps> for RefdByArticle {
    fn node_variant(&self) -> Graph3dNode {
        Graph3dNode::Article
    }
    fn node_pk(&self) -> i32 {
        self.art_id
    }
    fn node_name(&self) -> String {
        self.title.clone()
    }
    fn node_props(&self) -> xrows::ArticleProps {
        let auth_id: Option<i32> = None;
        xrows::ArticleProps{auth_id}
    }
}

impl<'a> tokio_postgres::types::FromSql<'a> for RefdByArticle {
    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let rba: RefdByArticle = serde_json::from_slice(raw)?;
        Ok(rba)
    }
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}