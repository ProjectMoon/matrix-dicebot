use barrel::backend::Sqlite;
use barrel::{types, Migration};
use log::info;

pub fn migration() -> String {
    let mut m = Migration::new();
    info!("Applying migration: {}", file!());

    //Table for basic room information: room ID, room name
    m.create_table("room_info", move |t| {
        t.add_column("id", types::primary());
        t.add_column("room_id", types::text());
        t.add_column("room_name", types::text());
    });

    //Table of users in rooms.
    m.create_table("room_users", move |t| {
        t.add_column("id", types::primary());
        t.add_column("room_id", types::text());
        t.add_column("username", types::text());
    });

    //Table of room ID, event ID, event timestamp
    m.create_table("room_events", move |t| {
        t.add_column("id", types::primary());
        t.add_column("room_id", types::text());
        t.add_column("event_id", types::text());
        t.add_column("event_timestamp", types::integer());
    });
    m.make::<Sqlite>()
}
