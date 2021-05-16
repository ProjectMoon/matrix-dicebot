use barrel::backend::Sqlite;
use barrel::{types, types::Type, Migration};
use log::info;

fn primary_uuid() -> Type {
    types::text().unique(true).nullable(false)
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

    //Table of room ID, event ID, event timestamp
    m.create_table("room_events", move |t| {
        t.add_column("room_id", types::text().nullable(false));
        t.add_column("event_id", types::text().nullable(false));
        t.add_column("event_timestamp", types::integer());
    });

    let mut res = m.make::<Sqlite>();

    //This is a hack that gives us a composite primary key.
    if res.ends_with(");") {
        res.pop();
        res.pop();
    }

    let x = format!("{}, PRIMARY KEY (room_id, event_id));", res);
    println!("{}", x);
    x
}
