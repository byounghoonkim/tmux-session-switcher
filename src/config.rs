use std::fs;

use serde::Deserialize;

use crate::tmux::favorite::Favorite;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub favorites: Vec<Favorite>,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        let contents = fs::read_to_string(config_file)
            .expect(format!("Failed to read config file : {}", config_file).as_str());
        let config: Config = toml::from_str(&contents).expect("Failed to parse config file");
        config
    }
}
