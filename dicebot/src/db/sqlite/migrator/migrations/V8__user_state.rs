use barrel::backend::Sqlite;
use barrel::{types, types::Type, Migration};

fn primary_uuid() -> Type {
    types::text().unique(true).primary(true).nullable(false)
}

pub fn migration() -> String {
    let mut m = Migration::new();

    // Keep track of contextual user state.
    m.create_table("user_state", move |t| {
        t.add_column("user_id", primary_uuid());
        t.add_column("active_room", types::text().nullable(true));
    });

    m.make::<Sqlite>()
}
