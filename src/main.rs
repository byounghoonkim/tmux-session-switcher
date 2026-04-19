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
mod picker;
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

fn add_favorite(config_path: &str, fav: tmux::favorite::Favorite) -> Result<(), String> {
    let mut config = Config::new(config_path);
    let favorites = config.favorites.get_or_insert_with(Vec::new);

    if favorites.iter().any(|f| f.name == fav.name) {
        return Err(format!("Favorite '{}' already exists.", fav.name));
    }

    let name = fav.name.clone();
    favorites.push(fav);
    config.save(config_path);
    println!("Added favorite '{}'.", name);
    Ok(())
}

/// Returns true if removed, false if not found
fn try_remove_favorite_by_name(config_path: &str, name: &str) -> bool {
    let mut config = Config::new(config_path);
    let Some(favorites) = config.favorites.as_mut() else {
        return false;
    };
    let len_before = favorites.len();
    favorites.retain(|f| f.name != name);
    if favorites.len() == len_before {
        return false;
    }
    config.save(config_path);
    true
}

fn remove_favorite_by_name(config_path: &str, name: &str) {
    if !try_remove_favorite_by_name(config_path, name) {
        eprintln!("Favorite '{}' not found.", name);
        std::process::exit(1);
    }
    println!("Removed favorite '{}'.", name);
}

fn remove_favorite_interactive(config_path: &str, use_fzf: bool, theme: &str) {
    let config = Config::new(config_path);
    let favorites = match config.favorites {
        Some(ref f) if !f.is_empty() => f.clone(),
        _ => {
            println!("No favorites found.");
            return;
        }
    };

    let item_strings: Vec<String> = favorites.iter().map(|f| f.to_string()).collect();

    match fzf::dispatch_picker(&item_strings, "Remove Favorite", "rounded", "default", use_fzf, theme, None) {
        fzf::PickerOutput::Selected(idx) => {
            if let Some(fav) = favorites.get(idx) {
                remove_favorite_by_name(config_path, &fav.name);
            }
        }
        fzf::PickerOutput::Cancelled | fzf::PickerOutput::New(_) => {}
    }
}

fn handle_remove(config_path: &str, name: Option<String>, use_fzf: bool, theme: &str) {
    match name {
        Some(name) => remove_favorite_by_name(config_path, &name),
        None => remove_favorite_interactive(config_path, use_fzf, theme),
    }
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

    let resolved_name = name.unwrap_or(cur_name);
    if resolved_name.is_empty() {
        eprintln!("Could not determine window name. Use --name to specify one.");
        std::process::exit(1);
    }

    let fav = tmux::favorite::Favorite {
        name: resolved_name,
        session_name: Some(session_name.unwrap_or(cur_session)),
        index: index.or(cur_index),
        path: {
            let p = path.unwrap_or(cur_path);
            if p.is_empty() { None } else { Some(p) }
        },
    };

    if let Err(e) = add_favorite(config_path, fav) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
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
        add_favorite(&path, make_fav("foo")).unwrap();
        let config = Config::new(&path);
        let favs = config.favorites.unwrap();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].name, "foo");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_add_favorite_duplicate_returns_err() {
        let path = temp_path("add_duplicate");
        add_favorite(&path, make_fav("foo")).unwrap();
        let result = add_favorite(&path, make_fav("foo"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_remove_favorite_by_name_success() {
        let path = temp_path("remove_success");
        add_favorite(&path, make_fav("bar")).unwrap();
        remove_favorite_by_name(&path, "bar");
        let config = Config::new(&path);
        let favs = config.favorites.unwrap_or_default();
        assert!(favs.iter().all(|f| f.name != "bar"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_remove_favorite_not_found() {
        let path = temp_path("remove_not_found");
        // empty config — removing nonexistent name returns false
        let result = try_remove_favorite_by_name(&path, "nonexistent");
        assert!(!result);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_remove_favorite_leaves_others_intact() {
        let path = temp_path("remove_leaves_others");
        add_favorite(&path, make_fav("keep")).unwrap();
        add_favorite(&path, make_fav("remove_me")).unwrap();
        remove_favorite_by_name(&path, "remove_me");
        let config = Config::new(&path);
        let favs = config.favorites.unwrap_or_default();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].name, "keep");
        std::fs::remove_file(&path).ok();
    }
}

fn main() {
    let args = Args::parse();
    let config_path = expand_tilde(&args.config)
        .to_str()
        .unwrap()
        .to_string();

    let config = Config::new(&config_path);

    let effective_use_fzf = match &args.picker {
        Some(args::PickerBackend::Fzf) => true,
        Some(args::PickerBackend::Native) => false,
        None => config.picker.as_deref() == Some("fzf"),
    };

    let effective_theme = args.theme.as_deref()
        .or(config.theme.as_deref())
        .unwrap_or("catppuccin")
        .to_string();
    let bell_fg = config.bell_fg.clone();

    if let Some(cmd) = args.command {
        match cmd {
            Commands::Favorite(fa) => match fa.command {
                FavoriteCommands::List => handle_list(&config_path),
                FavoriteCommands::Add { name, session_name, index, path } => {
                    handle_add(&config_path, name, session_name, index, path);
                }
                FavoriteCommands::Remove { name } => handle_remove(&config_path, name, effective_use_fzf, &effective_theme),
            },
            Commands::InternalPicker { items_path, result_path } => {
                let json = std::fs::read_to_string(&items_path)
                    .expect("Failed to read items file");
                let picker_config: picker::PickerConfig = serde_json::from_str(&json)
                    .expect("Failed to parse picker config");

                let result = picker::run(picker_config);

                match result {
                    picker::PickerResult::Selected(idx) => {
                        std::fs::write(&result_path, idx.to_string())
                            .expect("Failed to write result");
                    }
                    picker::PickerResult::New(title) => {
                        std::fs::write(&result_path, format!("new:{}", title))
                            .expect("Failed to write result");
                    }
                    picker::PickerResult::Cancelled => {
                        // 결과 파일 미작성 → outer process가 Cancelled로 처리
                    }
                }
            }
        }
        return;
    }

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
    let windows = match get_running_windows(&current_session) {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
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
        effective_use_fzf,
        &effective_theme,
        bell_fg,
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
