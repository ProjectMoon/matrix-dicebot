use barrel::backend::Sqlite;
use barrel::{types, types::Type, Migration};

fn primary_uuid() -> Type {
    types::text().unique(true).primary(true).nullable(false)
}

pub fn migration() -> String {
    let mut m = Migration::new();

    //Table for basic room information: room ID, room name
    m.create_table("room_info", move |t| {
        t.add_column("room_id", primary_uuid());
        t.add_column("room_name", types::text());
    });

    m.make::<Sqlite>()
}
