use log::info;
use refinery::config::{Config, ConfigDbType};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;
use std::str::FromStr;
use thiserror::Error;

//pub mod migrations;

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("sqlite connection error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("refinery migration error: {0}")]
    RefineryError(#[from] refinery::Error),
}

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/db/sqlite/migrator/migrations");
}

/// Run database migrations against the sqlite database.
pub async fn migrate(db_path: &str) -> Result<(), MigrationError> {
    //Create database if missing.
    let conn = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path))?
        .create_if_missing(true)
        .connect()
        .await?;

    drop(conn);

    let mut conn = Config::new(ConfigDbType::Sqlite).set_db_path(db_path);
    info!("Running migrations");
    embedded::migrations::runner().run(&mut conn)?;
    Ok(())
}
