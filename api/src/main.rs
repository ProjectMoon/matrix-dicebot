#[rocket::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tenebrous_api::api::run().await?;
    Ok(())
}
