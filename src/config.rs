use std::fs;

use serde::Deserialize;

use crate::tmux::favorite::Favorite;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub favorites: Option<Vec<Favorite>>,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        let contents = fs::read_to_string(config_file).unwrap_or_default();
        let config: Config = toml::from_str(&contents).expect("Failed to parse config file");
        config
    }
}
