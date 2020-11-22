use crate::db::Database;
use matrix_sdk::Client;

/// A context carried through the system providing access to things
/// like the database.
#[derive(Clone)]
pub struct Context<'a> {
    pub db: Database,
    pub matrix_client: &'a Client,
    pub room_id: &'a str,
    pub username: &'a str,
    pub message_body: &'a str,
}
