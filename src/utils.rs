use std::path::PathBuf;

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~") {
        let home = dirs::home_dir().expect("Could not determine home directory");
        return if path == "~" {
            home
        } else {
            home.join(path.strip_prefix("~/").unwrap_or(path))
        };
    }
    PathBuf::from(path)
}

fn config_dir_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("tmux-session-switcher");
    path
}

pub fn get_config_dir() -> PathBuf {
    let path = config_dir_path();
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create config directory");
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_path_is_absolute() {
        let dir = config_dir_path();
        assert!(dir.is_absolute(), "config dir must be an absolute path");
        assert!(dir.ends_with("tmux-session-switcher"));
    }
}
