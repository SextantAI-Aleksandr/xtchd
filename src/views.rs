use std::{vec::Vec};
use serde::{Serialize, Deserialize};
use serde_json;
use tokio_postgres;
use pachydurable::{autocomplete::{AutoComp, WhoWhatWhere}, redis::{Cacheable, CachedAutoComp, PreWarmDepth}};
use crate::{integrity::{XtchdContent, XtchdSQL}, xrows::{self, Graph3dEdge, Graph3dNode}};





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




impl<'a> tokio_postgres::types::FromSql<'a> for Topic {
    fn from_sql(_ty: &tokio_postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let topic: Topic = serde_json::from_slice(raw)?;
        Ok(topic)
    }
    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}



/// This struct captures properties associated with a video node
#[derive(Serialize, Deserialize)]
pub struct VideoProps {
    pub youtube_url: String,
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






