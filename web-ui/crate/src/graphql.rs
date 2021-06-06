use graphql_client::web::Client;
use graphql_client::web::ClientError;
use graphql_client::GraphQLQuery;

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
) -> Result<i64, ClientError> {
    let client = Client::new("http://localhost:10000/graphql");
    let variables = get_user_variable::Variables {
        room_id: room_id.to_owned(),
        user_id: user_id.to_owned(),
        variable: variable_name.to_owned(),
    };

    //TODO don't unwrap() option. map to err instead.
    let response = client.call(GetUserVariable, variables).await?;
    let response: graphql_client_web::Response<get_user_variable::ResponseData> = response;
    let value = response.data.map(|d| d.variable.value).unwrap();
    Ok(value)
}

pub async fn rooms_for_user(
    user_id: &str,
) -> Result<Vec<rooms_for_user::RoomsForUserUserRoomsRooms>, ClientError> {
    let client = Client::new("http://localhost:10000/graphql");
    let variables = rooms_for_user::Variables {
        user_id: user_id.to_owned(),
    };

    //TODO don't unwrap() option. map to err instead.
    let response = client.call(RoomsForUser, variables).await?;
    let response: graphql_client_web::Response<rooms_for_user::ResponseData> = response;
    let rooms = response
        .data
        .map(|d| d.user_rooms.rooms)
        .unwrap_or_default();
    Ok(rooms)
}
