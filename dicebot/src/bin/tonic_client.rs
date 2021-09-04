use tenebrous_rpc::protos::dicebot::UserIdRequest;
use tenebrous_rpc::protos::dicebot::{dicebot_client::DicebotClient};
use tonic::{metadata::MetadataValue, transport::Channel, Request};

async fn create_client(
    shared_secret: &str,
) -> Result<DicebotClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://0.0.0.0:9090")
        .connect()
        .await?;

    let bearer = MetadataValue::from_str(&format!("Bearer {}", shared_secret))?;

    let client = DicebotClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", bearer.clone());
        Ok(req)
    });

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = create_client("example-key").await?;

    // let request = tonic::Request::new(GetVariableRequest {
    //     user_id: "@projectmoon:agnos.is".into(),
    //     room_id: "!agICWvldGfuCywUVUM:agnos.is".into(),
    //     variable_name: "stuff".into(),
    // });

    // let response = client.get_variable(request).await?.into_inner();

    let request = tonic::Request::new(UserIdRequest {
        user_id: "@projectmoon:agnos.is".into(),
    });

    let response = client.rooms_for_user(request).await?.into_inner();
    // println!("RESPONSE={:?}", response);
    // println!("User friendly response is: {:?}", response.value);
    println!("Rooms: {:?}", response.rooms);
    Ok(())
}
