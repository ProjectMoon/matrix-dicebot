use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use axfive_matrix_dicebot::bot::DiceBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Need a config as an argument");
    println!("Logging in");
    let mut bot = DiceBot::from_path(config_path).await?;
    println!("Logged in");

    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        println!("Loop");
        select! {
            _ = sigint.recv() => {
                break;
            }
            result = bot.sync() => {
                result?;
            }
        }
    }

    println!("Logging out");
    bot.logout().await
}
