use serde::{self, Deserialize, Serialize};
use reqwest::Client;
use crate::matrix::SyncCommand;

#[derive(Serialize, Deserialize, Debug)]
pub struct MatrixConfig {
    user: String,
    password: String,
    home_server: String,
    next_batch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    matrix: MatrixConfig,
}

pub struct DiceBot {
    config: Config,
    access_token: String,
    next_batch: Option<String>,
    client: Client,
}

#[derive(Serialize, Debug)]
struct LoginRequest<'a, 'b, 'c> {
    #[serde(rename = "type")]
    type_: &'a str,
    user: &'b str,
    password: &'c str,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    access_token: String,
}

impl DiceBot {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();
        let request = LoginRequest {
            type_: "m.login.password",
            user: &config.matrix.user,
            password: &config.matrix.password,
        };
        let response = client.post(&format!("https://{}/_matrix/client/r0/login", config.matrix.home_server))
            .body(serde_json::to_string(&request)?)
            .send()
            .await?;
        let body: LoginResponse = serde_json::from_str(&response.text().await?)?;
        Ok(DiceBot{
            client,
            config,
            access_token: body.access_token,
            next_batch: None,
        })
    }

    pub async fn sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut url = format!("https://{}/_matrix/client/r0/sync?access_token={}&timeout=3000",
                self.config.matrix.home_server,
                self.access_token);
        if let Some(since) = &self.next_batch {
            url.push_str(&format!("&since={}", since));
        }
        let body = self.client.get(&url)
            .send()
            .await?
            .text()
            .await?;
        let sync: SyncCommand = serde_json::from_str(&body).unwrap();
        println!("{:#?}", sync);
        self.next_batch = Some(sync.next_batch);
        Ok(())
    }

    pub async fn logout(self) -> Result<(), Box<dyn std::error::Error>> {
        self.client.post(&format!("https://{}/_matrix/client/r0/logout?access_token={}", self.config.matrix.home_server, self.access_token))
            .body("{}")
            .send()
            .await?;
        Ok(())
    }
}
