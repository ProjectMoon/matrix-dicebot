use serde::{self, Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Shortcut to defining db migration versions. Will probably
/// eventually be moved to a config file.
const MIGRATION_VERSION: u32 = 3;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("toml parsing error: {0}")]
    TomlParsingError(#[from] toml::de::Error),
}

pub fn read_config<P: Into<PathBuf>>(config_path: P) -> Result<Config, ConfigError> {
    let config_path = config_path.into();
    let config = {
        let contents = fs::read_to_string(&config_path)?;
        deserialize_config(&contents)?
    };

    Ok(config)
}

fn deserialize_config(contents: &str) -> Result<Config, ConfigError> {
    let config = toml::from_str(&contents)?;
    Ok(config)
}

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

fn db_path_from_env() -> String {
    env::var("DATABASE_PATH")
        .expect("could not find database path in config or environment variable")
}

/// The "bot" section of the config file, for bot settings.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct BotConfig {
    /// How far back from current time should we process a message?
    oldest_message_age: Option<u64>,
}

/// The "database" section of the config file.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct DatabaseConfig {
    /// Path to the database storage directory. Required.
    path: Option<String>,
}

impl DatabaseConfig {
    #[inline]
    #[must_use]
    fn path(&self) -> String {
        self.path.clone().unwrap_or_else(|| db_path_from_env())
    }
}

impl BotConfig {
    /// Determine the oldest allowable message age, in seconds. If the
    /// setting is defined, use that value. If it is not defined, fall
    /// back to DEFAULT_OLDEST_MESSAGE_AGE (15 minutes).
    #[inline]
    #[must_use]
    fn oldest_message_age(&self) -> u64 {
        self.oldest_message_age
            .unwrap_or(DEFAULT_OLDEST_MESSAGE_AGE)
    }
}

/// Represents the toml config file for the dicebot. The sections of
/// the config are not directly accessible; instead the config
/// provides friendly methods that handle default values, etc.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    matrix: MatrixConfig,
    database: Option<DatabaseConfig>,
    bot: Option<BotConfig>,
}

impl Config {
    /// The matrix homeserver URL.
    #[inline]
    #[must_use]
    pub fn matrix_homeserver(&self) -> &str {
        &self.matrix.home_server
    }

    /// The username used to connect to the matrix server.
    #[inline]
    #[must_use]
    pub fn matrix_username(&self) -> &str {
        &self.matrix.username
    }

    /// The password used to connect to the matrix server.
    #[inline]
    #[must_use]
    pub fn matrix_password(&self) -> &str {
        &self.matrix.password
    }

    /// The path to the database storage directory.
    #[inline]
    #[must_use]
    pub fn database_path(&self) -> String {
        self.database
            .as_ref()
            .map(|db| db.path())
            .unwrap_or_else(|| db_path_from_env())
    }

    /// The current migration version we expect of the database. If
    /// this number is higher than the one in the database, we will
    /// execute migrations to update the data.
    #[inline]
    #[must_use]
    pub fn migration_version(&self) -> u32 {
        MIGRATION_VERSION
    }

    /// Figure out the allowed oldest message age, in seconds. This will
    /// be the defined oldest message age in the bot config, if the bot
    /// configuration and associated "oldest_message_age" setting are
    /// defined. If the bot config or the message setting are not defined,
    /// it will default to 15 minutes.
    #[inline]
    #[must_use]
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
            database: Some(DatabaseConfig {
                path: Some("".to_owned()),
            }),
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
            database: Some(DatabaseConfig {
                path: Some("".to_owned()),
            }),
            bot: None,
        };

        assert_eq!(15 * 60, cfg.oldest_message_age());
    }

    #[test]
    fn db_path_uses_setting_first_test() {
        let cfg = Config {
            matrix: MatrixConfig {
                home_server: "".to_owned(),
                username: "".to_owned(),
                password: "".to_owned(),
            },
            database: Some(DatabaseConfig {
                path: Some("the-db-path".to_owned()),
            }),
            bot: None,
        };

        assert_eq!("the-db-path".to_owned(), cfg.database_path());
    }

    #[test]
    fn db_path_uses_env_if_setting_not_defined_test() {
        env::set_var("DATABASE_PATH", "the-db-path");

        let cfg = Config {
            matrix: MatrixConfig {
                home_server: "".to_owned(),
                username: "".to_owned(),
                password: "".to_owned(),
            },
            database: Some(DatabaseConfig { path: None }),
            bot: None,
        };

        assert_eq!("the-db-path".to_owned(), cfg.database_path());

        env::remove_var("DATABASE_PATH");
    }

    #[test]
    fn db_path_uses_env_if_section_not_defined_test() {
        env::set_var("DATABASE_PATH", "the-db-path");

        let cfg = Config {
            matrix: MatrixConfig {
                home_server: "".to_owned(),
                username: "".to_owned(),
                password: "".to_owned(),
            },
            database: None,
            bot: None,
        };

        assert_eq!("the-db-path".to_owned(), cfg.database_path());

        env::remove_var("DATABASE_PATH");
    }

    use indoc::indoc;

    #[test]
    fn deserialize_config_without_bot_section_test() {
        let contents = indoc! {"
            [matrix]
            home_server = 'https://matrix.example.com'
            username = 'username'
            password = 'password'

            [database]
            path = ''
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

            [database]
            path = ''

            [bot]
            not_a_real_setting = 2
        "};

        let cfg: Result<_, _> = deserialize_config(contents);
        assert_eq!(true, cfg.is_ok());
    }

    #[test]
    fn deserialize_config_without_db_path_setting_test() {
        let contents = indoc! {"
            [matrix]
            home_server = 'https://matrix.example.com'
            username = 'username'
            password = 'password'

            [database]
            not_a_real_setting = 1

            [bot]
            not_a_real_setting = 2
        "};

        let cfg: Result<_, _> = deserialize_config(contents);
        assert_eq!(true, cfg.is_ok());
    }

    #[test]
    fn deserialize_config_without_db_section_test() {
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
