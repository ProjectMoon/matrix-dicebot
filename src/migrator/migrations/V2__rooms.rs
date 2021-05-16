use barrel::backend::Sqlite;
use barrel::{types, types::Type, Migration};
use log::info;

fn primary_uuid() -> Type {
    types::text().unique(true).primary(true).nullable(false)
}

fn autoincrement_int() -> Type {
    types::integer()
        .increments(true)
        .unique(true)
        .primary(false)
}

pub fn migration() -> String {
    let mut m = Migration::new();
    info!("Applying migration: {}", file!());

    //Table for basic room information: room ID, room name
    m.create_table("room_info", move |t| {
        t.add_column("room_id", primary_uuid());
        t.add_column("room_name", types::text());
    });

    //Table of users in rooms.
    m.create_table("room_users", move |t| {
        t.add_column("room_id", autoincrement_int());
        t.add_column("username", types::text());
    });

    m.make::<Sqlite>()
}
