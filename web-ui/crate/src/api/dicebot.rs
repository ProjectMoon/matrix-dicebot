use graphql_client::web::Client;
use graphql_client::GraphQLQuery;
use graphql_client_web::Response;

use super::ResponseExt;
use crate::error::UiError;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/queries/get_user_variable.graphql",
    response_derives = "Debug"
)]
struct GetUserVariable;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/queries/rooms_for_user.graphql",
    response_derives = "Debug"
)]
struct RoomsForUser;

pub async fn get_user_variable(
    room_id: &str,
    user_id: &str,
    variable_name: &str,
) -> Result<i64, UiError> {
    let client = Client::new("http://localhost:10000/graphql");
    let variables = get_user_variable::Variables {
        room_id: room_id.to_owned(),
        user_id: user_id.to_owned(),
        variable: variable_name.to_owned(),
    };

    let response = client.call(GetUserVariable, variables).await?;
    let response: graphql_client_web::Response<get_user_variable::ResponseData> = response;
    Ok(response.data()?.variable.value)
}

pub async fn rooms_for_user(
    user_id: &str,
) -> Result<Vec<rooms_for_user::RoomsForUserUserRoomsRooms>, UiError> {
    let client = Client::new("http://localhost:10000/graphql");
    let variables = rooms_for_user::Variables {
        user_id: user_id.to_owned(),
    };

    let response = client.call(RoomsForUser, variables).await?;
    let response: Response<rooms_for_user::ResponseData> = response;
    Ok(response.data()?.user_rooms.rooms)
}