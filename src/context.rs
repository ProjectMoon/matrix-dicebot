use crate::db::Database;

/// A context carried through the system providing access to things
/// like the database.
#[derive(Clone)]
pub struct Context<'a> {
    pub db: &'a Database,
    pub room_id: &'a str,
    pub username: &'a str,
    pub message_body: &'a str,
}

impl<'a> Context<'a> {
    pub fn new(
        db: &'a Database,
        room_id: &'a str,
        username: &'a str,
        message_body: &'a str,
    ) -> Context<'a> {
        Context {
            db: db,
            room_id: room_id,
            username: username,
            message_body: message_body,
        }
    }
}
