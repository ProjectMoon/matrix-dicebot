pub mod protos {
    pub mod web_api {
        tonic::include_proto!("web_api");
    }

    pub mod dicebot {
        tonic::include_proto!("dicebot");
    }
}

use protos::dicebot::dicebot_client::DicebotClient;
use tonic::{metadata::MetadataValue, transport::Channel as TonicChannel, Request as TonicRequest};

#[cfg(feature = "default")]
pub async fn create_client(
    address: &'static str,
    shared_secret: &str,
) -> Result<DicebotClient<TonicChannel>, Box<dyn std::error::Error>> {
    let channel = TonicChannel::from_shared(address)?.connect().await?;

    let bearer = MetadataValue::from_str(&format!("Bearer {}", shared_secret))?;
    let client = DicebotClient::with_interceptor(channel, move |mut req: TonicRequest<()>| {
        req.metadata_mut().insert("authorization", bearer.clone());
        Ok(req)
    });

    Ok(client)
}
