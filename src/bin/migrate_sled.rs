use tenebrous_dicebot::db::sqlite::{Database as SqliteDatabase, Variables};
use tenebrous_dicebot::db::Database;
use tenebrous_dicebot::error::BotError;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    let sled_path = std::env::args()
        .skip(1)
        .next()
        .expect("Need a path to a Sled database as an arument.");

    let sqlite_path = std::env::args()
        .skip(2)
        .next()
        .expect("Need a path to an sqlite database as an arument.");

    let db = Database::new(&sled_path)?;

    let all_variables = db.variables.get_all_variables()?;

    let sql_db = SqliteDatabase::new(&sqlite_path).await?;

    for var in all_variables {
        if let ((username, room_id, variable_name), value) = var {
            println!(
                "Migrating {}::{}::{} = {} to sql",
                username, room_id, variable_name, value
            );

            sql_db
                .set_user_variable(&username, &room_id, &variable_name, value)
                .await;
        }
    }

    Ok(())
}
