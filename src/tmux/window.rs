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
    pub(crate) marked: bool,
    pub(crate) bell: bool,
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
            if self.marked { " ♥️" } else { "" },
            if self.bell { " 🔔" } else { "" },
        )
    }
}

impl SortPriority for Window {
    fn sort_priority(&self) -> f32 {
        if self.active {
            return 0.0;
        }
        if self.bell {
            return 1.0;
        }
        if self.marked {
            return 2.0;
        }
        3.0
    }
}

impl crate::tmux::Item for Window {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window(active: bool, marked: bool, bell: bool) -> Window {
        Window {
            session_name: "work".to_string(),
            index: "1".to_string(),
            name: "editor".to_string(),
            active,
            marked,
            bell,
        }
    }

    #[test]
    fn test_display_bell_shows_icon() {
        let w = make_window(false, false, true);
        assert!(w.to_string().contains("🔔"));
    }

    #[test]
    fn test_display_no_bell_no_icon() {
        let w = make_window(false, false, false);
        assert!(!w.to_string().contains("🔔"));
    }

    #[test]
    fn test_sort_priority_bell_is_1() {
        let w = make_window(false, false, true);
        assert_eq!(w.sort_priority(), 1.0);
    }

    #[test]
    fn test_sort_priority_active_beats_bell() {
        let w = make_window(true, false, true);
        assert_eq!(w.sort_priority(), 0.0);
    }

    #[test]
    fn test_sort_priority_bell_beats_marked() {
        let bell_w = make_window(false, false, true);
        let marked_w = make_window(false, true, false);
        assert!(bell_w.sort_priority() < marked_w.sort_priority());
    }
}
