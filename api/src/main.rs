use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, FieldError, FieldResult, GraphQLObject,
    RootNode,
};
use once_cell::sync::OnceCell;
use rocket::{response::content, Rocket, State};
use std::cell::RefCell;
use std::env;
use std::sync::{Arc, RwLock};
use tenebrous_rpc::protos::dicebot::dicebot_client::DicebotClient;
use tenebrous_rpc::protos::dicebot::GetVariableRequest;
use tonic::{metadata::MetadataValue, transport::Channel as TonicChannel, Request as TonicRequest};
use tracing_subscriber::filter::EnvFilter;

//grpc stuff
async fn create_client(
    shared_secret: &str,
) -> Result<DicebotClient<TonicChannel>, Box<dyn std::error::Error>> {
    let channel = TonicChannel::from_static("http://0.0.0.0:9090")
        .connect()
        .await?;

    let bearer = MetadataValue::from_str(&format!("Bearer {}", shared_secret))?;
    let client = DicebotClient::with_interceptor(channel, move |mut req: TonicRequest<()>| {
        req.metadata_mut().insert("authorization", bearer.clone());
        Ok(req)
    });

    Ok(client)
}

//api stuff
#[derive(GraphQLObject)]
#[graphql(description = "User variable in a room.")]
struct UserVariable {
    room_id: String,
    variable_name: String,
    value: i32,
}

//graphql shit
#[derive(Clone)]
struct Context {
    dicebot_client: DicebotClient<TonicChannel>,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

#[derive(Clone, Copy, Debug)]
struct Query;

#[graphql_object(
    // Here we specify the context type for the object.
    // We need to do this in every type that
    // needs access to the context.
    context = Context,
)]
impl Query {
    fn apiVersion() -> &str {
        "1.0"
    }

    async fn variable(
        context: &mut Context,
        room_id: String,
        user_id: String,
        variable: String,
    ) -> FieldResult<UserVariable> {
        let request = TonicRequest::new(GetVariableRequest {
            room_id: room_id.clone(),
            user_id: user_id.clone(),
            variable_name: variable.clone(),
        });

        let response = context
            .dicebot_client
            .clone()
            .get_variable(request)
            .await?
            .into_inner();

        Ok(UserVariable {
            room_id: room_id.clone(),
            variable_name: variable.clone(),
            value: response.value,
        })
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    )
}

//rocket stuff

#[rocket::get("/")]
fn graphiql() -> content::Html<String> {
    juniper_rocket_async::graphiql_source("/graphql", None)
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket_async::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket_async::GraphQLResponse {
    request.execute_sync(&*schema, &*context)
}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket_async::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket_async::GraphQLResponse {
    request.execute_sync(&*schema, &*context)
}

#[rocket::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = if env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new("warp_async")
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let log = warp::log("warp_server");
    let client = create_client("abc123").await?;

    log::info!("Listening on 127.0.0.1:8080");
    let context = Context {
        dicebot_client: client,
    };

    Rocket::build()
        .manage(client)
        .manage(schema())
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch()
        .await
        .expect("server to launch");
    Ok(())
}
