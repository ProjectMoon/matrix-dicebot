use rocket::{response::content, Rocket, State};
use std::env;
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
    let filter = if env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new("tenebrous_api=info,tonic=info,rocket=info")
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    log::info!("Setting up gRPC connection");
    let client = tenebrous_rpc::create_client("http://localhost:9090", "abc123").await?;

    let context = Context {
        dicebot_client: client,
    };

    let schema = schema::schema();

    Rocket::build()
        .manage(context)
        .manage(schema)
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch()
        .await
        .expect("server to launch");
    Ok(())
}
