use crate::error::BotError;
use crate::{config::Config, db::sqlite::Database};
use log::{info, warn};
use matrix_sdk::Client;
use service::DicebotRpcService;
use std::sync::Arc;
use tenebrous_rpc::protos::dicebot::dicebot_server::DicebotServer;
use tonic::{metadata::MetadataValue, transport::Server, Request, Status};

pub(crate) mod service;

pub async fn serve_grpc(
    config: &Arc<Config>,
    db: &Database,
    client: &Client,
) -> Result<(), BotError> {
    match config.rpc_addr().zip(config.rpc_key()) {
        Some((addr, rpc_key)) => {
            let expected_bearer = MetadataValue::from_str(&format!("Bearer {}", rpc_key))?;
            let addr = addr.parse()?;

            let rpc_service = DicebotRpcService {
                db: db.clone(),
                config: config.clone(),
                client: client.clone(),
            };

            info!("Serving Dicebot gRPC service on {}", addr);

            let interceptor = move |req: Request<()>| match req.metadata().get("authorization") {
                Some(bearer) if bearer == expected_bearer => Ok(req),
                _ => Err(Status::unauthenticated("No valid auth token")),
            };

            let server = DicebotServer::with_interceptor(rpc_service, interceptor);

            Server::builder()
                .add_service(server)
                .serve(addr)
                .await
                .map_err(|e| e.into())
        }
        _ => noop().await,
    }
}

pub async fn noop() -> Result<(), BotError> {
    warn!("RPC address or shared secret not specified. Not enabling gRPC.");
    Ok(())
}
