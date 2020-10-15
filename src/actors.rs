use crate::config::Config;
use crate::db::Database;
use actix::prelude::*;
use state::DiceBotState;
use std::sync::Arc;

pub mod state;

pub struct Actors {
    global_state: Addr<DiceBotState>,
}

impl Actors {
    pub fn new(config: &Arc<Config>, _db: &Database) -> Actors {
        let global_state = DiceBotState::new(&config);

        Actors {
            global_state: global_state.start(),
        }
    }

    pub fn global_state(&self) -> Addr<DiceBotState> {
        self.global_state.clone()
    }
}
