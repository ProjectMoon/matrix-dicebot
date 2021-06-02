use barrel::backend::Sqlite;
use barrel::{types, types::Type, Migration};

fn primary_uuid() -> Type {
    types::text().unique(true).primary(true).nullable(false)
}

pub fn migration() -> String {
    let mut m = Migration::new();

    //Table of room ID, event ID, event timestamp
    m.create_table("accounts", move |t| {
        t.add_column("user_id", primary_uuid());
        t.add_column("password", types::text().nullable(false));
    });

    m.make::<Sqlite>()
}
