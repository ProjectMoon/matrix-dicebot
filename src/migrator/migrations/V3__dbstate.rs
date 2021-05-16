use barrel::backend::Sqlite;
use barrel::{types, Migration};
use log::info;

pub fn migration() -> String {
    let mut m = Migration::new();
    info!("Applying migration: {}", file!());

    //Basic state table with only device ID for now. Uses only one row.
    m.create_table("bot_state", move |t| {
        t.add_column("device_id", types::text());
    });

    m.make::<Sqlite>()
}
