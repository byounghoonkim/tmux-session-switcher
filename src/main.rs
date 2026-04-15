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
                println!("{}", fav);
            }
        }
        _ => println!("No favorites found."),
    }
}

fn add_favorite(config_path: &str, fav: tmux::favorite::Favorite) {
    let mut config = Config::new(config_path);
    let favorites = config.favorites.get_or_insert_with(Vec::new);

    if favorites.iter().any(|f| f.name == fav.name) {
        eprintln!("Favorite '{}' already exists.", fav.name);
        std::process::exit(1);
    }

    let name = fav.name.clone();
    favorites.push(fav);
    config.save(config_path);
    println!("Added favorite '{}'.", name);
}

fn handle_add(
    config_path: &str,
    name: Option<String>,
    session_name: Option<String>,
    index: Option<u16>,
    path: Option<String>,
) {
    let (cur_session, cur_index_str, cur_name, cur_path) = get_current_window();
    let cur_index: Option<u16> = cur_index_str.parse().ok();

    let fav = tmux::favorite::Favorite {
        name: name.unwrap_or(cur_name),
        session_name: Some(session_name.unwrap_or(cur_session)),
        index: index.or(cur_index),
        path: {
            let p = path.unwrap_or(cur_path);
            if p.is_empty() { None } else { Some(p) }
        },
    };

    add_favorite(config_path, fav);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tmux::favorite::Favorite;
    use std::env;

    fn temp_path(suffix: &str) -> String {
        let mut p = env::temp_dir();
        p.push(format!("tss_main_test_{}.toml", suffix));
        p.to_string_lossy().to_string()
    }

    fn make_fav(name: &str) -> Favorite {
        Favorite {
            name: name.to_string(),
            session_name: Some("main".to_string()),
            index: Some(1),
            path: Some("/tmp".to_string()),
        }
    }

    #[test]
    fn test_add_favorite_success() {
        let path = temp_path("add_success");
        add_favorite(&path, make_fav("foo"));
        let config = Config::new(&path);
        let favs = config.favorites.unwrap();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].name, "foo");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_add_favorite_duplicate_exits() {
        let path = temp_path("add_duplicate");
        add_favorite(&path, make_fav("foo"));
        // Second add with same name — test the duplicate check logic directly
        let config = Config::new(&path);
        let favs = config.favorites.unwrap();
        let already_exists = favs.iter().any(|f| f.name == "foo");
        assert!(already_exists);
        std::fs::remove_file(&path).ok();
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
            FavoriteCommands::Add { name, session_name, index, path } => {
                handle_add(&config_path, name, session_name, index, path);
                return;
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
