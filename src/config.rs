use serde::{self, Deserialize, Serialize};

/// The "matrix" section of the config, which gives home server, login information, and etc.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatrixConfig {
    /// Your homeserver of choice, as an FQDN without scheme or path
    pub home_server: String,

    /// Username to login as. Only the localpart.
    pub username: String,

    /// Bot account password.
    pub password: String,
}

const DEFAULT_OLDEST_MESSAGE_AGE: u64 = 15 * 60;

/// The "bot" section of the config file, for bot settings.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BotConfig {
    /// How far back from current time should we process a message?
    oldest_message_age: Option<u64>,
}

impl BotConfig {
    pub fn new() -> BotConfig {
        BotConfig {
            oldest_message_age: Some(DEFAULT_OLDEST_MESSAGE_AGE),
        }
    }

    /// Determine the oldest allowable message age, in seconds. If the
    /// setting is defined, use that value. If it is not defined, fall
    /// back to DEFAULT_OLDEST_MESSAGE_AGE (15 minutes).
    pub fn oldest_message_age(&self) -> u64 {
        match self.oldest_message_age {
            Some(seconds) => seconds,
            None => DEFAULT_OLDEST_MESSAGE_AGE,
        }
    }
}

/// Represents the toml config file for the dicebot.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub matrix: MatrixConfig,
    pub bot: Option<BotConfig>,
}

impl Config {
    pub fn bot(self) -> BotConfig {
        let none_cfg;
        let bot_cfg = match self.bot {
            Some(cfg) => cfg,
            None => {
                none_cfg = BotConfig::new();
                none_cfg
            }
        };

        bot_cfg
    }

    /// Figure out the allowed oldest message age, in seconds. This will
    /// be the defined oldest message age in the bot config, if the bot
    /// configuration and associated "oldest_message_age" setting are
    /// defined. If the bot config or the message setting are not defined,
    /// it will defualt to 15 minutes.
    pub fn get_oldest_message_age(&self) -> u64 {
        self.bot
            .as_ref()
            .unwrap_or(&BotConfig::new())
            .oldest_message_age()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oldest_message_default_no_setting_test() {
        let cfg = Config {
            matrix: MatrixConfig {
                home_server: "".to_owned(),
                username: "".to_owned(),
                password: "".to_owned(),
            },
            bot: Some(BotConfig {
                oldest_message_age: None,
            }),
        };

        assert_eq!(15 * 60, cfg.get_oldest_message_age());
    }

    #[test]
    fn oldest_message_default_no_bot_config_test() {
        let cfg = Config {
            matrix: MatrixConfig {
                home_server: "".to_owned(),
                username: "".to_owned(),
                password: "".to_owned(),
            },
            bot: None,
        };

        assert_eq!(15 * 60, cfg.get_oldest_message_age());
    }
}
