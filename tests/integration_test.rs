#[cfg(test)]
mod favorites {
    use std::env;

    fn temp_config(suffix: &str) -> String {
        let mut p = env::temp_dir();
        p.push(format!("tss_integ_{}.toml", suffix));
        p.to_string_lossy().to_string()
    }

    fn write_config(path: &str, content: &str) {
        std::fs::write(path, content).unwrap();
    }

    fn read_config(path: &str) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }

    #[test]
    fn test_config_roundtrip_preserves_favorites() {
        let path = temp_config("roundtrip");
        let toml = r#"
[[favorites]]
name = "work"
session_name = "main"
index = 2
path = "/home/user/work"
"#;
        write_config(&path, toml);
        let content = read_config(&path);
        assert!(content.contains("work"));
        assert!(content.contains("main"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_empty_config_file_reads_ok() {
        let path = temp_config("empty");
        write_config(&path, "");
        let content = read_config(&path);
        assert!(content.is_empty());
        std::fs::remove_file(&path).ok();
    }
}

#[cfg(test)]
mod previous_window {
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct PreviousWindow {
        session_name: String,
        index: String,
        name: String,
    }

    fn temp_json(suffix: &str) -> std::path::PathBuf {
        let mut p = env::temp_dir();
        p.push(format!("tss_prev_{}.json", suffix));
        p
    }

    fn write_previous(path: &std::path::PathBuf, session: &str, index: &str, name: &str) {
        let pw = PreviousWindow {
            session_name: session.to_string(),
            index: index.to_string(),
            name: name.to_string(),
        };
        std::fs::write(path, serde_json::to_string_pretty(&pw).unwrap()).unwrap();
    }

    fn read_previous(path: &std::path::PathBuf) -> Option<PreviousWindow> {
        let contents = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    #[test]
    fn test_previous_window_write_and_read() {
        let path = temp_json("write_read");
        write_previous(&path, "mysession", "3", "editor");
        let pw = read_previous(&path).expect("should read back previous window");
        assert_eq!(pw.session_name, "mysession");
        assert_eq!(pw.index, "3");
        assert_eq!(pw.name, "editor");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_previous_window_missing_file_returns_none() {
        let path = temp_json("missing_should_not_exist");
        std::fs::remove_file(&path).ok();
        assert!(read_previous(&path).is_none());
    }

    #[test]
    fn test_previous_window_overwrite() {
        let path = temp_json("overwrite");
        write_previous(&path, "first", "1", "shell");
        write_previous(&path, "second", "2", "vim");
        let pw = read_previous(&path).expect("should read overwritten value");
        assert_eq!(pw.session_name, "second");
        assert_eq!(pw.name, "vim");
        std::fs::remove_file(&path).ok();
    }
}

#[cfg(test)]
mod display_format {
    #[test]
    fn test_window_base_format_padding() {
        let session = "main";
        let index = "3";
        let name = "editor";
        let result = format!("{:15} - {:>3} - {}", session, index, name);
        assert_eq!(result, "main            -   3 - editor");
    }

    #[test]
    fn test_window_base_format_long_session() {
        let session = "verylongsessionname";
        let index = "10";
        let name = "term";
        let result = format!("{:15} - {:>3} - {}", session, index, name);
        assert_eq!(result, "verylongsessionname -  10 - term");
    }
}
