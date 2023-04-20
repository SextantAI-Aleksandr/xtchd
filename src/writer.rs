use postgres::types::ToSql;
use pachydurable::{connect::{ConnPoolNoTLS, ClientNoTLS, pool_no_tls_from_env}, err::GenericError};
use crate::integrity::{XtchdContent, VerifiedItem};


pub struct Pool {
    conn: ConnPoolNoTLS,
}

impl Pool {
    pub async fn new_from_env() -> Self {
        let conn = pool_no_tls_from_env().await.unwrap();
        Pool{conn}
    }

    pub async fn get(&self) -> Result<Writer, GenericError> {
        let c = self.conn.get().await?;
        let rows = c.query("
            SELECT item_id, new_sha256 FROM hash_integrity 
            ORDER BY item_id DESC LIMIT 1", &[]).await.unwrap();
        let row = rows.get(0).unwrap();
        let prior_id: i32 = row.get(0);
        let prior_sha256: String = row.get(1);
        Ok(Writer{c, prior_id, prior_sha256})
    }
}


pub struct Writer {
    pub c: ClientNoTLS,
    pub prior_id: i32,
    pub prior_sha256: String,
}

impl Writer {
    pub async fn write_row<T: XtchdContent>(&mut self, content:T) -> Result<(), GenericError> {
        let vi = VerifiedItem::new(self.prior_id, &self.prior_sha256, content);
        let _x = self.c.execute("INSERT INTO hash_integrity 
            (            prior_id,      id,           content_class,                name_or_text,       content_json,              to_parent,               from_child,      uploaded_at,     prior_sha256,        new_sha256)
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s)",
            &[&vi.prior_id, &vi.id(), &vi.content.class_str(), &vi.content.name_or_text(), &vi.content.json(), &vi.content.to_parent(), &vi.content.from_child(), &vi.uploaded_at, &vi.prior_sha256, &vi.new_sha256()]
        ).await.unwrap();
        Ok(())

    }
}



