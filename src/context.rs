use crate::db::Database;

/// A context carried through the system providing access to things
/// like the database.
#[derive(Clone)]
pub struct Context {
    pub db: Database,
    pub room_id: String,
    pub username: String,
    pub message_body: String,
}

impl Context {
    pub fn new(db: &Database, room_id: &str, username: &str, message_body: &str) -> Context {
        Context {
            db: db.clone(),
            room_id: room_id.to_owned(),
            username: username.to_owned(),
            message_body: message_body.to_owned(),
        }
    }
}
