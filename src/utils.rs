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

pub fn get_config_dir() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("tmux-session-switcher");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create config directory");
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_dir_is_absolute() {
        let dir = get_config_dir();
        assert!(dir.is_absolute(), "config dir must be an absolute path");
        assert!(dir.ends_with("tmux-session-switcher"));
    }

    #[test]
    fn test_get_config_dir_creates_directory() {
        let dir = get_config_dir();
        assert!(dir.exists(), "get_config_dir must create the directory");
    }
}
