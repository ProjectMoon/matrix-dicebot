use chronicle_dicebot::bot::run_bot;
use chronicle_dicebot::bot::Config;
use env_logger::Env;
use std::fs;
use std::path::PathBuf;

fn read_config<P: Into<PathBuf>>(config_path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = config_path.into();
    let config = {
        let contents = fs::read_to_string(&config_path)?;
        toml::from_str(&contents)?
    };

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::from_env(Env::default().default_filter_or("chronicle_dicebot=info")).init();

    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Need a config as an argument");

    let cfg = read_config(config_path)?;

    run_bot(cfg.matrix).await?;
    Ok(())
}
