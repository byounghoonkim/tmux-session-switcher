use std::fs;

use serde::{Deserialize, Serialize};

use crate::tmux::favorite::Favorite;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub favorites: Option<Vec<Favorite>>,
    pub picker: Option<String>,
    pub theme: Option<String>,
    pub bell_fg: Option<String>,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        let contents = fs::read_to_string(config_file).unwrap_or_default();
        if contents.is_empty() {
            return Config { favorites: None, picker: None, theme: None, bell_fg: None };
        }
        toml::from_str(&contents).expect("Failed to parse config file")
    }

    pub fn save(&self, config_file: &str) {
        if let Some(parent) = std::path::Path::new(config_file).parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        let contents = toml::to_string_pretty(self).expect("Failed to serialize config");
        fs::write(config_file, contents).expect("Failed to write config file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::favorite::Favorite;
    use std::env;

    fn temp_path(suffix: &str) -> String {
        let mut p = env::temp_dir();
        p.push(format!("tss_test_{}.toml", suffix));
        p.to_string_lossy().to_string()
    }

    #[test]
    fn test_config_save_and_reload() {
        let path = temp_path("save_reload");
        let config = Config {
            favorites: Some(vec![Favorite {
                name: "work".to_string(),
                session_name: Some("main".to_string()),
                index: Some(2),
                path: Some("/home/user/work".to_string()),
            }]),
            picker: None,
            theme: None,
            bell_fg: None,
        };
        config.save(&path);
        let loaded = Config::new(&path);
        let favs = loaded.favorites.unwrap();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].name, "work");
        assert_eq!(favs[0].session_name, Some("main".to_string()));
        assert_eq!(favs[0].index, Some(2));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_config_new_missing_file_returns_empty() {
        let path = temp_path("nonexistent_should_not_exist");
        std::fs::remove_file(&path).ok(); // ensure clean state
        let config = Config::new(&path);
        assert!(config.favorites.is_none());
    }

    #[test]
    fn test_config_picker_field() {
        let path = temp_path("picker_field");
        let config = Config {
            favorites: None,
            picker: Some("fzf".to_string()),
            theme: None,
            bell_fg: None,
        };
        config.save(&path);
        let loaded = Config::new(&path);
        assert_eq!(loaded.picker, Some("fzf".to_string()));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_config_bell_fg_field() {
        let path = temp_path("bell_fg_field");
        let config = Config {
            favorites: None,
            picker: None,
            theme: None,
            bell_fg: Some("#ff8c00".to_string()),
        };
        config.save(&path);
        let loaded = Config::new(&path);
        assert_eq!(loaded.bell_fg, Some("#ff8c00".to_string()));
        std::fs::remove_file(&path).ok();
    }
}
