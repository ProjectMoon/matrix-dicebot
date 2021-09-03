use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, FieldResult, GraphQLObject, RootNode,
};
use tenebrous_rpc::protos::dicebot::GetVariableRequest;
use tenebrous_rpc::protos::dicebot::{dicebot_client::DicebotClient, UserIdRequest};
use tonic::{transport::Channel as TonicChannel, Request as TonicRequest};

/// Hide generic type behind alias.
pub type DicebotGrpcClient = DicebotClient<TonicChannel>;

/// Single room for a user.
#[derive(GraphQLObject)]
#[graphql(description = "A matrix room.")]
struct Room {
    room_id: String,
    display_name: String,
}

/// List of rooms a user is in.
#[derive(GraphQLObject)]
#[graphql(description = "List of rooms a user is in.")]
struct UserRoomList {
    user_id: String,
    rooms: Vec<Room>,
}

/// A single user variable in a room.
#[derive(GraphQLObject)]
#[graphql(description = "User variable in a room.")]
struct UserVariable {
    room_id: String,
    user_id: String,
    variable_name: String,
    value: i32,
}

/// Context passed to every GraphQL function that holds stuff we need
/// (GRPC client).
#[derive(Clone)]
pub struct Context {
    pub dicebot_client: DicebotGrpcClient,
}

/// Marker trait to make the context object usable in GraphQL.
impl juniper::Context for Context {}

#[derive(Clone, Copy, Debug)]
pub struct Query;

#[graphql_object(
   context = Context,
)]
impl Query {
    fn api_version() -> &str {
        "1.0"
    }

    async fn variable(
        context: &Context,
        room_id: String,
        user_id: String,
        variable: String,
    ) -> FieldResult<UserVariable> {
        let request = TonicRequest::new(GetVariableRequest {
            room_id,
            user_id,
            variable_name: variable,
        });

        let response = context
            .dicebot_client
            .clone()
            .get_variable(request)
            .await?
            .into_inner();

        Ok(UserVariable {
            user_id: response.user_id,
            room_id: response.room_id,
            variable_name: response.variable_name,
            value: response.value,
        })
    }

    async fn user_rooms(context: &Context, user_id: String) -> FieldResult<UserRoomList> {
        let request = TonicRequest::new(UserIdRequest { user_id });

        let response = context
            .dicebot_client
            .clone()
            .rooms_for_user(request)
            .await?
            .into_inner();

        Ok(UserRoomList {
            user_id: response.user_id,
            rooms: response
                .rooms
                .into_iter()
                .map(|grpc_room| Room {
                    room_id: grpc_room.room_id,
                    display_name: grpc_room.display_name,
                })
                .collect(),
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
