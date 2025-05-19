use std::process::Command;

use serde::Deserialize;

use crate::tmux::Item;
use crate::tmux::SortPriority;
use crate::tmux::Switchable;
use crate::tmux::TMUX;

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct Favorite {
    pub(crate) name: String,
    pub(crate) session_name: Option<String>,
    pub(crate) index: Option<u16>,
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
            args.push("-k".to_string()); // -k : overwrite(kill) the existing target window
        } else {
            args.push("-S".to_string()); // -S : specify to reuse the name if there is no target
        }

        args.push("-n".to_string());
        args.push(self.name.to_string());

        //args.push("-P".to_string()); // -P : print the info of the new window to stdout

        if let Some(path) = &self.path {
            args.push("-c".to_string());
            args.push(path.to_string());
        }

        Command::new(TMUX)
            .args(args)
            .status()
            .expect("Failed to execute tmux windows create");
    }
}

impl std::fmt::Display for Favorite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {} ⭐️ {}",
            self.session_name.as_ref().unwrap_or(&"".to_string()),
            self.index.map(|i| i.to_string()).unwrap_or_default(),
            self.name,
            self.path.as_ref().unwrap_or(&"".to_string()),
        )
    }
}

impl SortPriority for Favorite {
    fn sort_priority(&self) -> f32 {
        5.0
    }
}

impl Item for Favorite {}
