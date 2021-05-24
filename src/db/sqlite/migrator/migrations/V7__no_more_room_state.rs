use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.drop_table_if_exists("room_info");
    m.drop_table_if_exists("room_users");
    m.make::<Sqlite>()
}
