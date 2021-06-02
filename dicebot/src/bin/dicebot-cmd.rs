use matrix_sdk::identifiers::room_id;
use matrix_sdk::Client;
use tenebrous_dicebot::commands;
use tenebrous_dicebot::commands::ResponseExtractor;
use tenebrous_dicebot::context::{Context, RoomContext};
use tenebrous_dicebot::db::sqlite::Database;
use tenebrous_dicebot::error::BotError;
use tenebrous_dicebot::models::Account;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let command = match commands::parser::parse_command(&input) {
        Ok(command) => command,
        Err(e) => return Err(e),
    };

    let homeserver = Url::parse("http://example.com")?;
    let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
    let db = Database::new(
        db_path
            .path()
            .to_str()
            .expect("Could not get path to temporary db"),
    )
    .await?;

    let context = Context {
        db,
        account: Account::default(),
        matrix_client: Client::new(homeserver).expect("Could not create matrix client"),
        origin_room: RoomContext {
            id: &room_id!("!fakeroomid:example.com"),
            display_name: "fake room".to_owned(),
            secure: false,
        },
        active_room: RoomContext {
            id: &room_id!("!fakeroomid:example.com"),
            display_name: "fake room".to_owned(),
            secure: false,
        },
        username: "@localuser:example.com",
        message_body: &input,
    };

    let message = command.execute(&context).await.message_html("fakeuser");
    let message = html2text::from_read(message.as_bytes(), 80);
    println!("{}", message.trim());
    Ok(())
}
