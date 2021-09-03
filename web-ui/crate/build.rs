use std::fs;
use tenebrous_api::schema;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_doc = schema::schema().as_schema_language();
    fs::write("schema.graphql", schema_doc)?;
    Ok(())
}
