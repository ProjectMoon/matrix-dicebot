//Needed for nested Result handling from tokio. Probably can go away after 1.47.0.
#![type_length_limit = "7605144"]
use futures::try_join;
use log::error;
use matrix_sdk::Client;
use std::env;
use std::sync::{Arc, RwLock};
use tenebrous_dicebot::bot::DiceBot;
use tenebrous_dicebot::config::*;
use tenebrous_dicebot::db::sqlite::Database;
use tenebrous_dicebot::error::BotError;
use tenebrous_dicebot::rpc;
use tenebrous_dicebot::state::DiceBotState;
use tracing_subscriber::filter::EnvFilter;

/// Attempt to create config object and ddatabase connection pool from
/// the given config path. An error is returned if config creation or
/// database pool creation fails for some reason.
async fn init(config_path: &str) -> Result<(Arc<Config>, Database, Client), BotError> {
    let cfg = read_config(config_path)?;
    let cfg = Arc::new(cfg);
    let sqlite_path = format!("{}/dicebot.sqlite", cfg.database_path());
    let db = Database::new(&sqlite_path).await?;
    let client = tenebrous_dicebot::matrix::create_client(&cfg)?;
    Ok((cfg, db, client))
}

#[tokio::main]
async fn main() -> Result<(), BotError> {
    let filter = if env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new("tonic=info,tenebrous_dicebot=info,dicebot=info,refinery=info")
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    match run().await {
        Ok(_) => (),
        Err(e) => error!("Error: {}", e),
    }

    Ok(())
}

async fn run() -> Result<(), BotError> {
    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Need a config as an argument");

    let (cfg, db, client) = init(&config_path).await?;
    let grpc = rpc::serve_grpc(&cfg, &db, &client);
    let bot = run_bot(&cfg, &db, &client);

    match try_join!(bot, grpc) {
        Ok(_) => (),
        Err(e) => error!("Error: {}", e),
    };

    Ok(())
}

async fn run_bot(cfg: &Arc<Config>, db: &Database, client: &Client) -> Result<(), BotError> {
    let state = Arc::new(RwLock::new(DiceBotState::new(&cfg)));

    match DiceBot::new(cfg, &state, db, client) {
        Ok(bot) => bot.run().await?,
        Err(e) => println!("Error connecting: {:?}", e),
    };

    Ok(())
}
