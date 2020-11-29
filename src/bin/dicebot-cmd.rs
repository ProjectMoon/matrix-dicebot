use chronicle_dicebot::commands;
use chronicle_dicebot::context::Context;
use chronicle_dicebot::db::Database;
use chronicle_dicebot::error::BotError;
use matrix_sdk::{
    identifiers::{room_id, user_id},
    Room,
};

fn dummy_room() -> Room {
    Room::new(
        &room_id!("!fakeroomid:example.com"),
        &user_id!("@fakeuserid:example.com"),
    )
}

#[tokio::main]
async fn main() -> Result<(), BotError> {
    let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let command = match commands::parser::parse_command(&input) {
        Ok(command) => command,
        Err(e) => return Err(e),
    };

    let context = Context {
        db: Database::new_temp()?,
        matrix_client: &matrix_sdk::Client::new("http://example.com")
            .expect("Could not create matrix client"),
        room: &dummy_room(),
        username: "@localuser:example.com",
        message_body: &input,
    };

    println!("{}", command.execute(&context).await.plain());
    Ok(())
}
