use crate::systems::GameSystem;
use barrel::backend::Sqlite;
use barrel::{types, types::Type, Migration};
use itertools::Itertools;
use strum::IntoEnumIterator;

fn primary_id() -> Type {
    types::text().unique(true).primary(true).nullable(false)
}

pub fn migration() -> String {
    let mut m = Migration::new();

    //Normally we would add a CHECK clause here, but types::custom requires a 'static string.
    //Which means we can't automagically generate one from the enum.
    m.create_table("room_info", move |t| {
        t.add_column("room_id", primary_id());
        t.add_column("game_system", types::text().nullable(false));
    });

    m.make::<Sqlite>()
}
