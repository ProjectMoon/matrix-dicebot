use crate::config::*;
use log::info;
use std::sync::Arc;

/// Holds state of the dice bot, for anything requiring mutable
/// transitions. This is a simple mutable trait whose values represent
/// the current state of the dicebot. It provides mutable methods to
/// change state.
pub struct DiceBotState {
    logged_skipped_old_messages: bool,
    _config: Arc<Config>,
}

impl DiceBotState {
    /// Create initial dice bot state.
    pub fn new(config: &Arc<Config>) -> DiceBotState {
        DiceBotState {
            logged_skipped_old_messages: false,
            _config: config.clone(),
        }
    }

    pub fn logged_skipped_old_messages(&self) -> bool {
        self.logged_skipped_old_messages
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
