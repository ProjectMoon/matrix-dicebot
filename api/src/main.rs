use log::info;
use rocket::http::Method;
use rocket::serde::{json::Json, Deserialize};
use rocket::{response::content, Rocket, State};
use rocket_cors::AllowedOrigins;
use std::env;
use tenebrous_api::config::{create_config, Config};
use tenebrous_api::schema::{self, Context, Schema};
use tracing_subscriber::filter::EnvFilter;

#[rocket::get("/")]
fn graphiql() -> content::Html<String> {
    juniper_rocket_async::graphiql_source("/graphql", None)
}

#[rocket::get("/graphql?<request>")]
async fn get_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket_async::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket_async::GraphQLResponse {
    request.execute(&*schema, &*context).await
}

#[rocket::post("/graphql", data = "<request>")]
async fn post_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket_async::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket_async::GraphQLResponse {
    request.execute(&*schema, &*context).await
}

#[rocket::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tenebrous_api::api::run().await?;
    Ok(())
}
