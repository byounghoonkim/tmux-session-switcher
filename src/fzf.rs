use std::process::Command;

use std::cmp::Ordering::Greater;

use std::cmp::Ordering::Less;

pub(crate) fn sort_windows(windows: &mut [Window]) {
    // Sort windows by active, marked, last and others
    windows.sort_by(|a, b| {
        if a.actvie && !b.actvie {
            return Less;
        } else if !a.actvie && b.actvie {
            return Greater;
        }
        if a.marked && !b.marked {
            return Less;
        } else if !a.marked && b.marked {
            return Greater;
        }
        if a.last_flag && !b.last_flag {
            return Less;
        } else if !a.last_flag && b.last_flag {
            return Greater;
        }
        std::cmp::Ordering::Equal
    });
}

pub(crate) fn select_window<'a>(
    windows: &'a [Window],
    size: &'a str,
    title: &str,
) -> Option<&'a Window> {
    let fzf_tmux = format!(
        r#"
        fzf-tmux \
            -p {} \
            --border-label ' {} ' \
            --prompt '⚡' \
            --bind 'tab:down,btab:up'
        "#,
        size, title
    );

    let select_result = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo '{}' | {}",
            windows.iter().map(|w| w.to_string()).collect::<String>(),
            fzf_tmux,
        ))
        .output()
        .expect("Failed to execute fzf-tmux")
        .stdout;
    let select_result = String::from_utf8_lossy(&select_result).trim().to_string();
    if select_result.is_empty() {
        return None;
    }
    let selected_window = windows
        .iter()
        .find(|w| w.to_string().trim() == select_result)
        .expect("Selected window not found");

    Some(selected_window)
}

pub(crate) struct Window {
    pub(crate) session_name: String,
    pub(crate) index: String,
    pub(crate) name: String,
    pub(crate) actvie: bool,
    pub(crate) last_flag: bool,
    pub(crate) marked: bool,
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:15} - {:3} - {}{}{}{}",
            self.session_name,
            self.index,
            self.name,
            if self.actvie { " 🟢" } else { "" },
            if self.last_flag { "  ⃝" } else { "" },
            if self.marked { " ♥️" } else { "" },
        )
    }
}
