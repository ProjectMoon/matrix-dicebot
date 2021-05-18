use barrel::backend::Sqlite;
use barrel::{types, types::Type, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    //Table of users in rooms.
    m.create_table("room_users", move |t| {
        t.add_column("room_id", types::text());
        t.add_column("username", types::text());
    });

    let mut res = m.make::<Sqlite>();

    //This is a hack that gives us a composite primary key.
    if res.ends_with(");") {
        res.pop();
        res.pop();
    }

    format!("{}, PRIMARY KEY (room_id, username));", res)
}
