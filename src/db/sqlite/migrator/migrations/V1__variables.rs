use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("user_variables", |t| {
        t.add_column("room_id", types::text());
        t.add_column("user_id", types::text());
        t.add_column("key", types::text());
        t.add_column("value", types::integer());
    });

    m.make::<Sqlite>()
}
