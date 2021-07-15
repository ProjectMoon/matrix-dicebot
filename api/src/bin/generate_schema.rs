use tenebrous_api::schema;
fn main() {
    println!("{}", schema::schema().as_schema_language());
}
