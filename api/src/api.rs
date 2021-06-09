use crate::config::create_config;
use crate::schema::{self, Context, Schema};
use log::info;
use rocket::http::Method;
use rocket::{response::content, Rocket, State};
use rocket_cors::AllowedOrigins;
use std::env;
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

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let filter = if env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new("tenebrous_api=info,tonic=info,rocket=info,rocket_cors=info")
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    log::info!("Setting up gRPC connection");
    let rocket = Rocket::build();
    let config = create_config(&rocket);

    info!("Allowed CORS origins: {}", config.allowed_origins.join(","));

    //TODO move to config
    let client = tenebrous_rpc::create_client("http://localhost:9090", "abc123").await?;

    let context = Context {
        dicebot_client: client,
    };

    let schema = schema::schema();

    let allowed_origins = AllowedOrigins::some_exact(&config.allowed_origins);

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    let routes: Vec<rocket::Route> = {
        rocket::routes![graphiql, get_graphql_handler, post_graphql_handler]
            .into_iter()
            .chain(crate::auth::routes().into_iter())
            .collect()
    };

    rocket
        .mount("/", routes)
        .attach(cors)
        .manage(context)
        .manage(schema)
        .manage(config)
        .launch()
        .await
        .expect("server to launch");
    Ok(())
}
