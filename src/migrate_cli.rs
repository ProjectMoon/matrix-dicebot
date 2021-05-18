use std::env;
use tenebrous_dicebot::db::sqlite::migrator;

#[tokio::main]
async fn main() -> Result<(), migrator::MigrationError> {
    let args: Vec<String> = env::args().collect();
    let db_path: &str = match &args[..] {
        [_, path] => path.as_ref(),
        [_, _, ..] => panic!("Expected exactly 0 or 1 argument"),
        _ => "dicebot.sqlite",
    };

    println!("Using database: {}", db_path);

    migrator::migrate(db_path).await
}
