use clap::Parser;

use tmux::Item;

mod config;
mod fzf;
mod tmux;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Size of the fzf window
    #[arg(short, long, default_value = "80,36")]
    size: String,

    /// Title of the fzf window
    #[arg(short, long, default_value = "Select Window")]
    title: String,

    /// Path to the config file
    #[arg(
        short,
        long,
        default_value = "~/.config/tmux-session-switcher/config.toml"
    )]
    config: String,
}

fn main() {
    let args = Args::parse();
    let config = config::Config::new(&args.config);

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

    fzf::sort_windows(&mut ws);
    if let Some(sw) = fzf::select_item::<dyn Item>(&ws, &args.size, &args.title) {
        sw.switch_window();
    }
}
