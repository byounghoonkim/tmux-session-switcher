use clap::{Parser, ValueEnum};

use config::Config;
use fzf::{select_item, sort_by_priority};
use tmux::{
    Item, create_new_window, get_current_session, get_running_windows, load_previous_window,
    save_previous_window,
};
use utils::expand_tilde;

mod config;
mod fzf;
mod tmux;
mod utils;

#[derive(Clone, Debug, ValueEnum)]
enum BorderStyle {
    Rounded,
    Sharp,
    Bold,
    Block,
    Thinblock,
    Double,
    Horizontal,
    Vertical,
    Top,
    Bottom,
    Left,
    Right,
    None,
}

impl ToString for BorderStyle {
    fn to_string(&self) -> String {
        match self {
            BorderStyle::Rounded => "rounded",
            BorderStyle::Sharp => "sharp",
            BorderStyle::Bold => "bold",
            BorderStyle::Block => "block",
            BorderStyle::Thinblock => "thinblock",
            BorderStyle::Double => "double",
            BorderStyle::Horizontal => "horizontal",
            BorderStyle::Vertical => "vertical",
            BorderStyle::Top => "top",
            BorderStyle::Bottom => "bottom",
            BorderStyle::Left => "left",
            BorderStyle::Right => "right",
            BorderStyle::None => "none",
        }
        .to_string()
    }
}

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

    #[arg(short, long, default_value_t = BorderStyle::Rounded)]
    border: BorderStyle,
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

    match select_item(&ws, &args.title, &args.border.to_string()) {
        fzf::SelectItemReturn::None => {
            //println!("No item selected.");
        }
        fzf::SelectItemReturn::Item(item) => {
            // Save current active window as previous before switching, but only if it's different from selected
            if let Some(current_window) = current_active_window {
                if current_window.session_name != item.session_name()
                    || current_window.index != item.index()
                    || current_window.name != item.name()
                {
                    save_previous_window(
                        &current_window.session_name,
                        &current_window.index,
                        &current_window.name,
                    );
                }
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
