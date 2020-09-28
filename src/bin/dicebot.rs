//Needed for nested Result handling from tokio. Probably can go away after 1.47.0.
#![type_length_limit = "7605144"]
use actix::prelude::*;
use chronicle_dicebot::bot::DiceBot;
use chronicle_dicebot::config::*;
use chronicle_dicebot::error::BotError;
use chronicle_dicebot::state::DiceBotState;
use env_logger::Env;
use log::error;
use std::fs;
use std::path::PathBuf;

fn read_config<P: Into<PathBuf>>(config_path: P) -> Result<Config, BotError> {
    let config_path = config_path.into();
    let config = {
        let contents = fs::read_to_string(&config_path)?;
        deserialize_config(&contents)?
    };

    Ok(config)
}

fn deserialize_config(contents: &str) -> Result<Config, BotError> {
    let config = toml::from_str(&contents)?;
    Ok(config)
}

#[actix_rt::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(e) => error!("Error: {:?}", e),
    };
}

async fn run() -> Result<(), BotError> {
    env_logger::from_env(Env::default().default_filter_or("chronicle_dicebot=info")).init();

    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Need a config as an argument");

    let cfg = read_config(config_path)?;
    let bot_state = DiceBotState::new(&cfg).start();

    match DiceBot::new(&cfg, bot_state) {
        Ok(bot) => bot.run().await?,
        Err(e) => println!("Error connecting: {:?}", e),
    };

    System::current().stop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn deserialize_config_without_bot_section_test() {
        let contents = indoc! {"
            [matrix]
            home_server = 'https://matrix.example.com'
            username = 'username'
            password = 'password'
        "};

        let cfg: Result<_, _> = deserialize_config(contents);
        assert_eq!(true, cfg.is_ok());
    }

    #[test]
    fn deserialize_config_without_oldest_message_setting_test() {
        let contents = indoc! {"
            [matrix]
            home_server = 'https://matrix.example.com'
            username = 'username'
            password = 'password'

            [bot]
            not_a_real_setting = 2
        "};

        let cfg: Result<_, _> = deserialize_config(contents);
        assert_eq!(true, cfg.is_ok());
    }
}
