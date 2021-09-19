use crate::db::{errors::DataError, Variables};
use crate::error::BotError;
use crate::matrix;
use crate::{config::Config, db::sqlite::Database};
use futures::stream;
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use matrix_sdk::ruma::UserId;
use matrix_sdk::{room::Joined, Client};
use std::convert::TryFrom;
use std::sync::Arc;
use tenebrous_rpc::protos::dicebot::{
    dicebot_server::Dicebot, rooms_list_reply::Room, GetAllVariablesReply, GetAllVariablesRequest,
    RoomsListReply, SetVariableReply, SetVariableRequest, UserIdRequest,
};
use tenebrous_rpc::protos::dicebot::{GetVariableReply, GetVariableRequest};
use tonic::{Code, Request, Response, Status};

impl From<BotError> for Status {
    fn from(error: BotError) -> Status {
        Status::new(Code::Internal, error.to_string())
    }
}

impl From<DataError> for Status {
    fn from(error: DataError) -> Status {
        Status::new(Code::Internal, error.to_string())
    }
}

#[derive(Clone)]
pub(super) struct DicebotRpcService {
    pub(super) config: Arc<Config>,
    pub(super) db: Database,
    pub(super) client: Client,
}

#[tonic::async_trait]
impl Dicebot for DicebotRpcService {
    async fn set_variable(
        &self,
        request: Request<SetVariableRequest>,
    ) -> Result<Response<SetVariableReply>, Status> {
        let SetVariableRequest {
            user_id,
            room_id,
            variable_name,
            value,
        } = request.into_inner();

        self.db
            .set_user_variable(&user_id, &room_id, &variable_name, value)
            .await?;

        Ok(Response::new(SetVariableReply { success: true }))
    }

    async fn get_variable(
        &self,
        request: Request<GetVariableRequest>,
    ) -> Result<Response<GetVariableReply>, Status> {
        let request = request.into_inner();
        let value = self
            .db
            .get_user_variable(&request.user_id, &request.room_id, &request.variable_name)
            .await?;

        Ok(Response::new(GetVariableReply { value }))
    }

    async fn get_all_variables(
        &self,
        request: Request<GetAllVariablesRequest>,
    ) -> Result<Response<GetAllVariablesReply>, Status> {
        let request = request.into_inner();
        let variables = self
            .db
            .get_user_variables(&request.user_id, &request.room_id)
            .await?;

        Ok(Response::new(GetAllVariablesReply { variables }))
    }

    async fn rooms_for_user(
        &self,
        request: Request<UserIdRequest>,
    ) -> Result<Response<RoomsListReply>, Status> {
        let UserIdRequest { user_id } = request.into_inner();
        let user_id = UserId::try_from(user_id).map_err(BotError::from)?;

        let rooms_for_user = matrix::get_rooms_for_user(&self.client, &user_id)
            .err_into::<BotError>()
            .await?;

        let mut rooms: Vec<Room> = stream::iter(rooms_for_user)
            .filter_map(|room: Joined| async move {
                let room: Result<Room, _> = room.display_name().await.map(|room_name| Room {
                    room_id: room.room_id().to_string(),
                    display_name: room_name,
                });

                Some(room)
            })
            .err_into::<BotError>()
            .try_collect()
            .await?;

        let sort = |r1: &Room, r2: &Room| {
            r1.display_name
                .to_lowercase()
                .cmp(&r2.display_name.to_lowercase())
        };

        rooms.sort_by(sort);

        Ok(Response::new(RoomsListReply { rooms }))
    }
}
