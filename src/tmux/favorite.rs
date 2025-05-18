use std::process::Command;

use crate::tmux::Item;
use crate::tmux::SortPriority;
use crate::tmux::Switchable;
use crate::tmux::TMUX;

pub(crate) struct Favorite {
    pub(crate) name: String,
    pub(crate) session_name: Option<String>,
    pub(crate) index: Option<String>,
    pub(crate) path: Option<String>,
}

impl Switchable for Favorite {
    fn switch_window(&self) {
        let mut args: Vec<String> = Vec::new();
        args.push("new-window".to_string());

        let target = match (&self.session_name, &self.index) {
            (Some(session_name), Some(index)) => Some(format!("{}:{}", session_name, index)),
            (Some(session_name), None) => Some(session_name.to_string()),
            (None, Some(index)) => Some(index.to_string()),
            (None, None) => None,
        };

        if let Some(target) = target {
            args.push("-t".to_string());
            args.push(target);
            // -k : overwrite(kill) the existing target window
            args.push("-k".to_string());
        } else {
            // -S : specify to reuse the name if there is no target
            args.push("-S".to_string());
        }

        if let Some(path) = &self.path {
            args.push("-c".to_string());
            args.push(path.to_string());
        }

        args.push("-n".to_string());
        args.push(self.name.to_string());

        Command::new(TMUX)
            .args(args)
            .status()
            .expect("Failed to execute tmux switch");

        // TODO: get session and index from command and switch to it

        // Command::new(TMUX)
        //     .args([
        //         "switch",
        //         "-t",
        //         &format!("{}:{}", self.session_name, self.index,),
        //     ])
        //     .status()
        //     .expect("Failed to execute tmux switch");
    }
}

impl std::fmt::Display for Favorite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {} ⭐️ {}",
            self.session_name.as_ref().unwrap_or(&"".to_string()),
            self.index.as_ref().unwrap_or(&"".to_string()),
            self.name,
            self.path.as_ref().unwrap_or(&"".to_string()),
        )
    }
}

impl SortPriority for Favorite {
    fn sort_priority(&self) -> f32 {
        0.5
    }
}

impl Item for Favorite {}
