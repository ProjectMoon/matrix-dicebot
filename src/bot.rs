use crate::matrix::{Event, MessageContent, RoomEvent, SyncCommand, NoticeMessage};
use crate::commands::parse_command;
use reqwest::{Client, Url};
use serde::{self, Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const USER_AGENT: &str =
    "AxFive Matrix DiceBot/0.1.0 (+https://gitlab.com/Taywee/axfive-matrix-dicebot)";

/// The "matrix" section of the config, which gives home server, login information, and etc.
#[derive(Serialize, Deserialize, Debug)]
pub struct MatrixConfig {
    /// Your homeserver of choice, as an FQDN without scheme or path
    pub home_server: String,

    /// The next batch to grab.  This should be set automatically
    #[serde(default)]
    pub next_batch: Option<String>,

    /// The transaction ID.  This should be set automatically
    #[serde(default)]
    pub txn_id: u64,
    
    /// The login table.  This may be set to whatever you wish, depending on your login method,
    /// though multi-step logins (like challenge-based) won't work here.
    pub login: toml::Value,
}

/// The base config, which is read from and written to by the bot
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub matrix: MatrixConfig,
}

/// The actual dicebot structure, which drives the entire operation.
///
/// This is the core of the dicebot program.
pub struct DiceBot {
    config_path: Option<PathBuf>,
    config: Config,
    access_token: String,
    next_batch: Option<String>,
    client: Client,
    home_server: Url,
    txn_id: u64,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    access_token: String,
}

impl DiceBot {
    /// Create a new dicebot from the given config path and config
    pub async fn new(
        config_path: Option<PathBuf>,
        config: Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let home_server: Url = format!("https://{}", config.matrix.home_server).parse()?;
        let client = Client::new();
        let request = serde_json::to_string(&config.matrix.login)?;
        let mut login_url = home_server.clone();
        login_url.set_path("/_matrix/client/r0/login");
        let response = client
            .post(login_url)
            .header("user-agent", USER_AGENT)
            .body(request)
            .send()
            .await?;
        let body: LoginResponse = serde_json::from_str(&response.text().await?)?;
        let next_batch = config.matrix.next_batch.clone();
        let txn_id = config.matrix.txn_id;
        Ok(DiceBot {
            home_server,
            config_path,
            client,
            config,
            access_token: body.access_token,
            next_batch,
            txn_id,
        })
    }

    /// Create a new dicebot, storing the config path to write it out  
    pub async fn from_path<P: Into<PathBuf>>(
        config_path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = config_path.into();
        let config = {
            let contents = fs::read_to_string(&config_path)?;
            toml::from_str(&contents)?
        };
        DiceBot::new(Some(config_path), config).await
    }

    /// Build a url using the current home server and the given path, as well as appending the
    /// access token
    fn url<S: AsRef<str>>(&self, path: S, query: &[(&str, &str)]) -> Url {
        let mut url = self.home_server.clone();
        url.set_path(path.as_ref());
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("access_token", &self.access_token);

            for pair in query.iter() {
                query_pairs.append_pair(pair.0, pair.1);
            }
        }

        url
    }

    /// Sync to the matrix homeserver, acting on events as necessary
    pub async fn sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sync_url = self.url("/_matrix/client/r0/sync", &[("timeout", "30000")]);

        // TODO: handle http 429
        if let Some(since) = &self.next_batch {
            sync_url.query_pairs_mut().append_pair("since", since);
        }
        let body = self
            .client
            .get(sync_url)
            .header("user-agent", USER_AGENT)
            .send()
            .await?
            .text()
            .await?;
        let sync: SyncCommand = serde_json::from_str(&body).unwrap();
        // First join invited rooms
        for room in sync.rooms.invite.keys() {
            let join_url = self.url(format!("/_matrix/client/r0/rooms/{}/join", room), &[]);
            self.client
                .post(join_url)
                .header("user-agent", USER_AGENT)
                .send()
                .await?;
        }

        for (room_id, room) in sync.rooms.join.iter() {
            for event in &room.timeline.events {
                if let Event::Room(RoomEvent {
                    sender,
                    event_id: _,
                    content: MessageContent::Text(message),
                    ..
                }) = event
                {
                    let (plain, html): (String, String) = match parse_command(message.body()) {
                        Ok(Some(command)) => {
                            let command = command.execute();
                            (command.plain().into(), command.html().into())
                        },
                        Ok(None) => continue,
                        Err(e) => {
                            let message = format!("Error parsing command: {}", e);
                            let html_message = format!("<p><strong>{}</strong></p>", message);
                            (message, html_message)
                        },
                    };

                    let plain = format!("{}\n{}", sender, plain);
                    let html = format!("<p>{}</p>\n{}", sender, html);

                    let message = NoticeMessage {
                        body: plain,
                        format: Some("org.matrix.custom.html".into()),
                        formatted_body: Some(html),
                    };

                    self.txn_id += 1;
                    let send_url = self.url(format!("/_matrix/client/r0/rooms/{}/send/m.room.message/{}", room_id, self.txn_id), &[]);
                    self.client
                        .put(send_url)
                        .header("user-agent", USER_AGENT)
                        .body(serde_json::to_string(&message)?)
                        .send()
                        .await?;
                }
            }
        }
        self.next_batch = Some(sync.next_batch);
        Ok(())
    }

    /// Log off of the matrix server, also writing out the config file if one was given in
    /// construction
    pub async fn logout(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let logout_url = self.url("/_matrix/client/r0/logout", &[]);
        self.client
            .post(logout_url)
            .header("user-agent", USER_AGENT)
            .body("{}")
            .send()
            .await?;

        self.config.matrix.next_batch = self.next_batch;
        self.config.matrix.txn_id = self.txn_id;

        if let Some(config_path) = self.config_path {
            let config = toml::to_string_pretty(&self.config)?;
            fs::write(config_path, config)?;
        }

        Ok(())
    }
}
