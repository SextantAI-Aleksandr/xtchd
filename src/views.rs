use std::vec::Vec;
use serde::{Serialize, Deserialize};
use serde_json;
use crate::{integrity::XtchdContent, xrows};


#[derive(Serialize)]
pub struct ArticleDetail {
    pub author: XtchdContent<xrows::Author>,
    pub article: XtchdContent<xrows::Article>,
    pub paragraphs: Vec<XtchdContent<xrows::ArticlePara>>,
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

