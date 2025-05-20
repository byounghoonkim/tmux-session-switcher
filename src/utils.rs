use std::path::PathBuf;

use home::home_dir;

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~") {
        let home = home_dir().expect("Could not determine home directory");
        return if path == "~" {
            home
        } else {
            home.join(path.strip_prefix("~/").unwrap_or(path))
        };
    }
    PathBuf::from(path)
}
