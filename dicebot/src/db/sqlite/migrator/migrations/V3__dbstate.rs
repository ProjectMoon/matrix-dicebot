use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    //Basic state table with only device ID for now. Uses only one row.
    m.create_table("bot_state", move |t| {
        t.add_column("device_id", types::text());
    });

    m.make::<Sqlite>()
}
