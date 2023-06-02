pub mod integrity;
pub mod xrows;
pub mod views;
pub mod xtchr;


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_enriched_article_fields() {
        // Ensure you can deserialize the fields needed to return an EnrichedArticle
        // See the implementations of tokio_postgres::types::FromSql in view.rs 
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = xtchr::Pool::new_from_env().await;
            let x = pool.get().await.unwrap();
            let rows = x.c.query("SELECT author, article, refs, paragraphs
                FROM enriched_article_fields
                ORDER BY art_id DESC LIMIT 3", &[]).await.unwrap();
            for row in rows {
                let _author: integrity::XtchdContent<xrows::Author> = row.get(0);
                let _article: integrity::XtchdContent<xrows::Article> = row.get(1);
                // Postgres: combined_refs.refs can be null due to LEFT JOIN if there are no -1 rows for that article
                let _opt_refs: Option<views::References> = row.get(2); 
                let _paragraphs: Vec<views::EnrichedPara> = row.get(3);
            }
        });
    }
}
