use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, FieldResult, GraphQLInputObject,
    GraphQLObject, RootNode,
};
use rocket::{response::content, Rocket, State};
use std::env;
use tenebrous_rpc::protos::dicebot::dicebot_client::DicebotClient;
use tenebrous_rpc::protos::dicebot::GetVariableRequest;
use tonic::{transport::Channel as TonicChannel, Request as TonicRequest};

//api stuff
#[derive(GraphQLInputObject)]
struct UserVariableArgument {
    room_id: String,
    user_id: String,
    variable_name: String,
}

#[derive(GraphQLObject)]
#[graphql(description = "User variable in a room.")]
struct UserVariable {
    room_id: String,
    user_id: String,
    variable_name: String,
    value: i32,
}

//graphql shit
#[derive(Clone)]
pub struct Context {
    pub dicebot_client: DicebotClient<TonicChannel>,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

#[derive(Clone, Copy, Debug)]
pub struct Query;

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
        context: &Context,
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
            user_id: user_id.clone(),
            room_id: room_id.clone(),
            variable_name: variable.clone(),
            value: response.value,
        })
    }
}

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    )
}
