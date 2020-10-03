use serde::{self, Deserialize, Serialize};

/// The "matrix" section of the config, which gives home server, login information, and etc.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct MatrixConfig {
    /// Your homeserver of choice, as an FQDN without scheme or path
    home_server: String,

    /// Username to login as. Only the localpart.
    username: String,

    /// Bot account password.
    password: String,
}

const DEFAULT_OLDEST_MESSAGE_AGE: u64 = 15 * 60;

/// The "bot" section of the config file, for bot settings.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct BotConfig {
    /// How far back from current time should we process a message?
    oldest_message_age: Option<u64>,
}

impl BotConfig {
    /// Determine the oldest allowable message age, in seconds. If the
    /// setting is defined, use that value. If it is not defined, fall
    /// back to DEFAULT_OLDEST_MESSAGE_AGE (15 minutes).
    fn oldest_message_age(&self) -> u64 {
        match self.oldest_message_age {
            Some(seconds) => seconds,
            None => DEFAULT_OLDEST_MESSAGE_AGE,
        }
    }
}

/// Represents the toml config file for the dicebot. The sections of
/// the config are not directly accessible; instead the config
/// provides friendly methods that handle default values, etc.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    matrix: MatrixConfig,
    bot: Option<BotConfig>,
}

impl Config {
    /// The matrix homeserver URL.
    pub fn matrix_homeserver(&self) -> &str {
        &self.matrix.home_server
    }

    /// The username used to connect to the matrix server.
    pub fn matrix_username(&self) -> &str {
        &self.matrix.username
    }

    /// The password used to connect to the matrix server.
    pub fn matrix_password(&self) -> &str {
        &self.matrix.password
    }

    /// Figure out the allowed oldest message age, in seconds. This will
    /// be the defined oldest message age in the bot config, if the bot
    /// configuration and associated "oldest_message_age" setting are
    /// defined. If the bot config or the message setting are not defined,
    /// it will defualt to 15 minutes.
    pub fn oldest_message_age(&self) -> u64 {
        self.bot
            .as_ref()
            .map(|bc| bc.oldest_message_age())
            .unwrap_or(DEFAULT_OLDEST_MESSAGE_AGE)
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

        assert_eq!(15 * 60, cfg.oldest_message_age());
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

        assert_eq!(15 * 60, cfg.oldest_message_age());
    }
}
