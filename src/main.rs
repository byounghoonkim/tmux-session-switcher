use clap::Parser;

use config::Config;
use fzf::{select_item, sort_by_priority};
use tmux::{Item, create_new_window, get_current_session, get_running_windows, load_previous_window, save_previous_window};
use utils::expand_tilde;

mod config;
mod fzf;
mod tmux;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the config file
    #[arg(
        short,
        long,
        default_value = "~/.config/tmux-session-switcher/config.toml"
    )]
    config: String,

    #[arg(short, long, default_value = "Select Window")]
    title: String,
}

fn main() {
    let args = Args::parse();
    let config = Config::new(expand_tilde(&args.config).to_str().unwrap());

    let mut ws: Vec<Box<dyn Item>> = Vec::new();

    // Add favorites from config
    if let Some(favorites) = config.favorites {
        for favorite in favorites {
            ws.push(Box::new(favorite));
        }
    }

    // Add previous window if available
    if let Some(previous) = load_previous_window() {
        ws.push(Box::new(previous));
    }

    let current_session = get_current_session();
    let windows = get_running_windows(&current_session);
    
    // Find current active window to save before switching
    let current_active_window = windows.iter().find(|w| w.active);
    
    for window in &windows {
        ws.push(Box::new(window.clone()));
    }

    sort_by_priority(&mut ws);

    match select_item(&ws, &args.title) {
        fzf::SelectItemReturn::None => {
            //println!("No item selected.");
        }
        fzf::SelectItemReturn::Item(item) => {
            // Save current active window as previous before switching
            if let Some(current_window) = current_active_window {
                save_previous_window(
                    &current_window.session_name,
                    &current_window.index,
                    &current_window.name,
                );
            }
            item.switch_window();
        }
        fzf::SelectItemReturn::NewWindowTitle(title) => {
            // Save current active window as previous before creating new window
            if let Some(current_window) = current_active_window {
                save_previous_window(
                    &current_window.session_name,
                    &current_window.index,
                    &current_window.name,
                );
            }
            create_new_window(&current_session, &title);
        }
    }
}
