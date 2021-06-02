use std::net::SocketAddr;
use tenebrous_rpc::protos::web_api::{
    web_api_server::{WebApi, WebApiServer},
    RoomsListReply, UserIdRequest,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request, Response, Status};

struct WebApiService;

#[tonic::async_trait]
impl WebApi for WebApiService {
    async fn list_room(
        &self,
        request: Request<UserIdRequest>,
    ) -> Result<Response<RoomsListReply>, Status> {
        println!("Hello hopefully from a web browser");
        Ok(Response::new(RoomsListReply { rooms: vec![] }))
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 10000));
    let listener = TcpListener::bind(addr).await.expect("listener");
    let url = format!("http://{}", listener.local_addr().unwrap());
    println!("Listening at {}", url);

    let svc = tonic_web::config()
        .allow_origins(vec!["http://localhost:8000"])
        .enable(WebApiServer::new(WebApiService));

    let fut = Server::builder()
        .accept_http1(true)
        .add_service(svc)
        .serve_with_incoming(TcpListenerStream::new(listener));

    fut.await?;
    Ok(())
}
