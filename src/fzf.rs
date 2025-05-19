use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;
use std::fmt::Display;
use std::process::Command;

use super::tmux::SortPriority;

pub(crate) fn sort_by_priority<T: SortPriority + ?Sized>(items: &mut [Box<T>]) {
    items.sort_by(|a, b| {
        if a.sort_priority() > b.sort_priority() {
            return Greater;
        } else if a.sort_priority() < b.sort_priority() {
            return Less;
        }
        std::cmp::Ordering::Equal
    });
}

pub(crate) fn select_item<'a, T: Display + ?Sized>(
    items: &'a [Box<T>],
    size: &'a str,
    title: &str,
) -> Option<&'a T> {
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
            items.iter().map(|w| w.to_string()).collect::<String>(),
            fzf_tmux,
        ))
        .output()
        .expect("Failed to execute fzf-tmux")
        .stdout;
    let select_result = String::from_utf8_lossy(&select_result).trim().to_string();
    if select_result.is_empty() {
        return None;
    }
    let selected_item = items
        .iter()
        .find(|w| w.to_string().trim() == select_result)
        .expect("Selected window not found");

    Some(selected_item)
}
