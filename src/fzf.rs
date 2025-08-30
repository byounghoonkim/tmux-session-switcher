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

pub enum SelectItemReturn<'a, T> {
    None,
    Item(&'a T),
    NewWindowTitle(String),
}

// return T or String or None
pub(crate) fn select_item<'a, T: Display + ?Sized>(
    items: &'a [Box<T>],
    title: &str,
) -> SelectItemReturn<'a, Box<T>> {
    let height = std::cmp::min(items.len() + 5, 40);
    let fzf_tmux = format!(
        r#"
        fzf \
            --tmux 80,{} \
            --layout=default \
            --border=rounded \
            --border-label ' {} ' \
            --prompt '⚡' \
            --bind 'tab:down,btab:up' \
            --print-query
        "#,
        height, title
    );

    let select_result = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo '{}' | {}",
            items.iter().map(|w| w.to_string()).collect::<String>(),
            fzf_tmux,
        ))
        .output()
        .expect("Failed to execute fzf")
        .stdout;
    let select_result = String::from_utf8_lossy(&select_result).trim().to_string();
    if select_result.is_empty() {
        return SelectItemReturn::None;
    }

    match select_result.split_once('\n') {
        Some((_, item)) => {
            let selected_item = items.iter().find(|w| w.to_string().trim() == item.trim());
            match selected_item {
                Some(item) => SelectItemReturn::Item(item),
                None => SelectItemReturn::None,
            }
        }
        None => {
            let selected_item = items
                .iter()
                .find(|w| w.to_string().trim() == select_result.trim());
            match selected_item {
                Some(item) => SelectItemReturn::Item(item),
                None => SelectItemReturn::NewWindowTitle(select_result),
            }
        }
    }
}
