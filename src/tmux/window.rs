use std::process::Command;

use crate::tmux::SortPriority;
use crate::tmux::Switchable;
use crate::tmux::TMUX;

#[derive(Clone)]
pub(crate) struct Window {
    pub(crate) session_name: String,
    pub(crate) index: String,
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) last_flag: bool,
    pub(crate) marked: bool,
}

impl Switchable for Window {
    fn switch_window(&self) {
        Command::new(TMUX)
            .args([
                "switch",
                "-t",
                &format!("{}:{}", self.session_name, self.index,),
            ])
            .status()
            .expect("Failed to execute tmux switch");
    }
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {}{}{}{}",
            self.session_name,
            self.index,
            self.name,
            if self.active { " 🟢" } else { "" },
            if self.last_flag { "  ⃝" } else { "" },
            if self.marked { " ♥️" } else { "" },
        )
    }
}

impl SortPriority for Window {
    fn sort_priority(&self) -> f32 {
        // Sort windows by active, marked, last and others
        if self.active {
            return 0.0;
        }
        if self.last_flag {
            return 1.0;
        }
        if self.marked {
            return 2.0;
        }
        3.0
    }
}

impl crate::tmux::Item for Window {}
