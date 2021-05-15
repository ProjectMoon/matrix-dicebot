//Needed for nested Result handling from tokio. Probably can go away after 1.47.0.
#![type_length_limit = "7605144"]
use env_logger::Env;
use log::error;
use std::sync::{Arc, RwLock};
use tenebrous_dicebot::bot::DiceBot;
use tenebrous_dicebot::config::*;
use tenebrous_dicebot::db::sqlite::Database;
use tenebrous_dicebot::error::BotError;
use tenebrous_dicebot::migrator;
use tenebrous_dicebot::state::DiceBotState;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("tenebrous_dicebot=info,dicebot=info"),
    )
    .init();
    match run().await {
        Ok(_) => (),
        Err(e) => error!("Error: {}", e),
    };
}

async fn run() -> Result<(), BotError> {
    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Need a config as an argument");

    let cfg = Arc::new(read_config(config_path)?);
    let sqlite_path = format!("{}/dicebot.sqlite", cfg.database_path());
    let db = Database::new(&sqlite_path).await?;
    let state = Arc::new(RwLock::new(DiceBotState::new(&cfg)));

    migrator::migrate(&sqlite_path).await?;

    match DiceBot::new(&cfg, &state, &db) {
        Ok(bot) => bot.run().await?,
        Err(e) => println!("Error connecting: {:?}", e),
    };

    Ok(())
}
