use tokio;
use pachydurable::err::GenericError;
use xtchd::xtchr::Xtchr;

#[tokio::main]
async fn main() -> Result<(), GenericError> {
    let mut xtchr = Xtchr::new_from_env().await;
    let (auth, hclink) = xtchr.add_author("Some guy").await?;
    println!("Created author '{}' with auth_id={} and new_sha256='{}'", &auth.name, &auth.auth_id, &hclink.new_sha256());
    Ok(())
}