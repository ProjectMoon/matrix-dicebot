// async fn test_grpc_web() {
//     use grpc_web_client::Client as GrpcWebClient;
//     use tenebrous_rpc::protos::web_api::web_api_client::WebApiClient as TheCloud;
//     use tenebrous_rpc::protos::web_api::{RoomsListReply, UserIdRequest};

//     let client = GrpcWebClient::new("http://localhost:10000".to_string());
//     let mut client = TheCloud::new(client);

//     let request = tonic::Request::new(UserIdRequest {
//         user_id: "WebTonic".into(),
//     });

//     let response = client.list_room(request).await.unwrap().into_inner();
//     println!("Room reply: {:?}", response);
// }
