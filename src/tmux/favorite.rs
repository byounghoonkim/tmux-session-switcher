use crate::tmux::Item;
use crate::tmux::SortPriority;
use crate::tmux::Switchable;

pub(crate) struct Favorite {
    pub(crate) name: String,
    pub(crate) session_name: Option<String>,
    pub(crate) index: Option<String>,
    pub(crate) path: Option<String>,
}

impl Switchable for Favorite {
    fn switch_window(&self) {
        todo!("Switching to favorite is not implemented yet");
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
        return 0.5;
    }
}

impl Item for Favorite {}
