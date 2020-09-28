use crate::config::*;
use actix::prelude::*;
use log::info;

#[derive(Message)]
#[rtype(result = "bool")]
pub struct LogSkippedOldMessages;

/// Holds state of the dice bot, for anything requiring mutable
/// transitions. This is a simple mutable trait whose values represent
/// the current state of the dicebot. It provides mutable methods to
/// change state.
pub struct DiceBotState {
    logged_skipped_old_messages: bool,
    config: Config,
}

impl Actor for DiceBotState {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!(
            "Oldest allowable message time is {} seconds ago",
            &self.config.get_oldest_message_age()
        );
    }
}

impl DiceBotState {
    /// Create initial dice bot state.
    pub fn new(config: &Config) -> DiceBotState {
        DiceBotState {
            logged_skipped_old_messages: false,
            config: config.clone(),
        }
    }

    /// Log and record that we have skipped some old messages. This
    /// method will log once, and then no-op from that point on.
    pub fn skipped_old_messages(&mut self) {
        if !self.logged_skipped_old_messages {
            info!("Skipped some messages received while offline because they are too old.");
        }

        self.logged_skipped_old_messages = true;
    }
}

impl Handler<LogSkippedOldMessages> for DiceBotState {
    type Result = bool;

    fn handle(&mut self, _msg: LogSkippedOldMessages, _ctx: &mut Context<Self>) -> Self::Result {
        self.skipped_old_messages();
        self.logged_skipped_old_messages
    }
}
