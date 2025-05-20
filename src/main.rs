use clap::Parser;

use tmux::Item;
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
    let config = config::Config::new(expand_tilde(&args.config).to_str().unwrap());

    let mut ws: Vec<Box<dyn Item>> = Vec::new();

    // Add favorites from config
    if let Some(favorites) = config.favorites {
        for favorite in favorites {
            ws.push(Box::new(favorite));
        }
    }

    let current_session = tmux::get_current_session();
    let windows = tmux::get_running_windows(&current_session);
    for window in &windows {
        ws.push(Box::new(window.clone()));
    }

    fzf::sort_by_priority(&mut ws);
    if let Some(sw) = fzf::select_item::<dyn Item>(&ws, &args.title) {
        sw.switch_window();
    }
}
