use clap::Parser;

use args::{Args, Commands, FavoriteCommands};
use config::Config;
use fzf::{select_item, sort_by_priority};
use tmux::{
    Item, create_new_window, get_current_session, get_current_window, get_running_windows,
    load_previous_window, save_previous_window,
};
use utils::expand_tilde;

mod args;
mod config;
mod fzf;
mod tmux;
mod utils;

fn handle_list(config_path: &str) {
    let config = Config::new(config_path);
    match config.favorites {
        Some(favs) if !favs.is_empty() => {
            for fav in &favs {
                print!("{}", fav);
            }
        }
        _ => println!("No favorites found."),
    }
}

fn main() {
    let args = Args::parse();
    let config_path = expand_tilde(&args.config)
        .to_str()
        .unwrap()
        .to_string();

    if let Some(Commands::Favorite(fa)) = args.command {
        match fa.command {
            FavoriteCommands::List => {
                handle_list(&config_path);
                return;
            }
            FavoriteCommands::Add { .. } => {
                todo!("add not yet implemented");
            }
            FavoriteCommands::Remove { .. } => {
                todo!("remove not yet implemented");
            }
        }
    }

    let config = Config::new(&config_path);
    let mut ws: Vec<Box<dyn Item>> = Vec::new();

    if let Some(favorites) = config.favorites {
        for favorite in favorites {
            ws.push(Box::new(favorite));
        }
    }

    if let Some(previous) = load_previous_window() {
        ws.push(Box::new(previous));
    }

    let current_session = get_current_session();
    let windows = get_running_windows(&current_session);
    let current_active_window = windows.iter().find(|w| w.active);

    for window in &windows {
        ws.push(Box::new(window.clone()));
    }

    sort_by_priority(&mut ws);

    match select_item(
        &ws,
        &args.title,
        &args.border.to_string(),
        &args.layout.to_string(),
    ) {
        fzf::SelectItemReturn::None => {}
        fzf::SelectItemReturn::Item(item) => {
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
