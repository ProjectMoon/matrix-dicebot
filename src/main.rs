use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use axfive_matrix_dicebot::matrix::SyncCommand;
use axfive_matrix_dicebot::bot::{DiceBot, Config};
use std::fs::read_to_string;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Need a config as an argument");
    let config = {
        let contents = read_to_string(config_path)?;
        toml::from_str(&contents)?
    };
    println!("Logging in");
    let mut bot = DiceBot::new(config).await?;
    println!("Logged in");

    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        println!("Loop");
        select! {
            _ = sigint.recv() => {
                break;
            }
            _ = bot.sync() => {
            }
        }
    }

    println!("Logging out");
    bot.logout().await
}
