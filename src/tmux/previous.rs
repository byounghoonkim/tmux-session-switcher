use std::process::Command;
use serde::{Deserialize, Serialize};

use crate::tmux::{Item, SortPriority, Switchable, TMUX};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PreviousWindow {
    pub(crate) session_name: String,
    pub(crate) index: String,
    pub(crate) name: String,
}

impl Switchable for PreviousWindow {
    fn switch_window(&self) {
        Command::new(TMUX)
            .args([
                "switch",
                "-t",
                &format!("{}:{}", self.session_name, self.index),
            ])
            .status()
            .expect("Failed to execute tmux switch to previous window");
    }
}

impl std::fmt::Display for PreviousWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {} 🔙",
            self.session_name, self.index, self.name
        )
    }
}

impl SortPriority for PreviousWindow {
    fn sort_priority(&self) -> f32 {
        0.5 // Higher priority than marked/other windows, but lower than active
    }
}

impl Item for PreviousWindow {
    fn session_name(&self) -> String {
        self.session_name.clone()
    }
    
    fn index(&self) -> String {
        self.index.clone()
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
}