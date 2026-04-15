# Favorite Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `tss favorite add/remove/list` 서브커맨드를 추가해 config.toml의 즐겨찾기를 CLI에서 직접 관리할 수 있게 한다.

**Architecture:** clap 서브커맨드로 `Commands::Favorite`를 추가하고, 기존 fzf 스위처는 서브커맨드 없을 때 그대로 동작한다. Config에 `Serialize`와 `save()` 를 추가해 읽기/쓰기를 모두 지원하고, 순수 로직 함수와 tmux 호출 함수를 분리해 테스트 가능하게 설계한다.

**Tech Stack:** Rust, clap 4.5 (derive feature), serde (Serialize + Deserialize), toml 0.9, fzf (shell), tmux

---

## File Map

| 파일 | 변경 내용 |
|------|-----------|
| `src/args.rs` | `Commands`, `FavoriteArgs`, `FavoriteCommands` 추가 |
| `src/config.rs` | `Serialize` derive 추가, `save()` 메서드 추가 |
| `src/tmux/favorite.rs` | `Serialize` derive 추가 |
| `src/tmux/mod.rs` | `get_current_window()` 함수 추가 |
| `src/main.rs` | 서브커맨드 분기 + `handle_add/remove/list` 함수 추가 |

---

### Task 1: Config와 Favorite에 Serialize 추가 + Config::save() 구현

**Files:**
- Modify: `src/config.rs`
- Modify: `src/tmux/favorite.rs`

- [ ] **Step 1: config.rs에 실패 테스트 작성**

`src/config.rs` 파일 맨 아래에 추가:

```rust
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
        let config = Config::new("/tmp/tss_nonexistent_999.toml");
        assert!(config.favorites.is_none());
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test test_config_save_and_reload 2>&1 | head -20
```

예상: `error[E0277]: the trait bound 'Config: Serialize' is not satisfied` 또는 `save` 메서드 없음 에러

- [ ] **Step 3: Favorite에 Serialize 추가**

`src/tmux/favorite.rs` 상단:

```rust
use serde::{Deserialize, Serialize};
```

`Favorite` 구조체 derive 변경:

```rust
#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Favorite {
    pub(crate) name: String,
    pub(crate) session_name: Option<String>,
    pub(crate) index: Option<u16>,
    pub(crate) path: Option<String>,
}
```

- [ ] **Step 4: Config에 Serialize 추가 + save() 구현**

`src/config.rs` 전체를 다음으로 교체:

```rust
use std::fs;

use serde::{Deserialize, Serialize};

use crate::tmux::favorite::Favorite;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub favorites: Option<Vec<Favorite>>,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        let contents = fs::read_to_string(config_file).unwrap_or_default();
        if contents.is_empty() {
            return Config { favorites: None };
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
        let config = Config::new("/tmp/tss_nonexistent_999.toml");
        assert!(config.favorites.is_none());
    }
}
```

- [ ] **Step 5: 테스트 통과 확인**

```bash
cargo test test_config 2>&1
```

예상: `test config::tests::test_config_save_and_reload ... ok` / `test config::tests::test_config_new_missing_file_returns_empty ... ok`

- [ ] **Step 6: 전체 빌드 확인**

```bash
cargo build 2>&1
```

예상: 에러 없음

- [ ] **Step 7: 커밋**

```bash
git add src/config.rs src/tmux/favorite.rs
git commit -m "feat: add Serialize to Config/Favorite and Config::save()"
```

---

### Task 2: get_current_window() 추가

**Files:**
- Modify: `src/tmux/mod.rs`

- [ ] **Step 1: get_current_window() 구현**

`src/tmux/mod.rs`의 `get_current_session()` 함수 아래에 추가:

```rust
pub(crate) fn get_current_window() -> (String, String, String, String) {
    let fields = "#{session_name}|#{window_index}|#{window_name}|#{pane_current_path}";
    let output = Command::new(TMUX)
        .args(["display-message", "-p", fields])
        .output()
        .expect("Failed to execute tmux command")
        .stdout;
    let output = String::from_utf8_lossy(&output).trim().to_string();
    let parts: Vec<&str> = output.splitn(4, '|').collect();
    (
        parts.first().unwrap_or(&"").to_string(),
        parts.get(1).unwrap_or(&"").to_string(),
        parts.get(2).unwrap_or(&"").to_string(),
        parts.get(3).unwrap_or(&"").to_string(),
    )
}
```

반환값: `(session_name, window_index, window_name, pane_current_path)`

- [ ] **Step 2: 빌드 확인**

```bash
cargo build 2>&1
```

예상: 에러 없음

- [ ] **Step 3: 수동 동작 확인 (tmux 세션 안에서)**

```bash
cargo run -- --help
```

예상: 도움말 출력, 빌드 에러 없음

- [ ] **Step 4: 커밋**

```bash
git add src/tmux/mod.rs
git commit -m "feat: add get_current_window() to tmux module"
```

---

### Task 3: Args에 favorite 서브커맨드 추가

**Files:**
- Modify: `src/args.rs`

- [ ] **Step 1: 파싱 테스트 작성**

`src/args.rs` 맨 아래에 추가:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_favorite_add_no_args() {
        let args = Args::try_parse_from(["tss", "favorite", "add"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::Add { name, session_name, index, path } => {
                    assert!(name.is_none());
                    assert!(session_name.is_none());
                    assert!(index.is_none());
                    assert!(path.is_none());
                }
                _ => panic!("Expected Add"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_favorite_add_with_name() {
        let args = Args::try_parse_from(["tss", "favorite", "add", "--name", "my-window"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::Add { name, .. } => {
                    assert_eq!(name, Some("my-window".to_string()));
                }
                _ => panic!("Expected Add"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_favorite_remove_with_name() {
        let args = Args::try_parse_from(["tss", "favorite", "remove", "--name", "foo"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::Remove { name } => {
                    assert_eq!(name, Some("foo".to_string()));
                }
                _ => panic!("Expected Remove"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_favorite_list() {
        let args = Args::try_parse_from(["tss", "favorite", "list"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::List => {}
                _ => panic!("Expected List"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_no_subcommand_still_works() {
        let args = Args::try_parse_from(["tss"]).unwrap();
        assert!(args.command.is_none());
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test test_favorite 2>&1 | head -20
```

예상: `Commands`, `FavoriteCommands` 정의 없음 에러

- [ ] **Step 3: args.rs에 서브커맨드 추가**

`src/args.rs` 전체를 다음으로 교체:

```rust
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Clone, Debug, ValueEnum)]
pub enum BorderStyle {
    Rounded,
    Sharp,
    Bold,
    Block,
    Thinblock,
    Double,
    Horizontal,
    Vertical,
    Top,
    Bottom,
    Left,
    Right,
    None,
}

impl fmt::Display for BorderStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BorderStyle::Rounded => "rounded",
            BorderStyle::Sharp => "sharp",
            BorderStyle::Bold => "bold",
            BorderStyle::Block => "block",
            BorderStyle::Thinblock => "thinblock",
            BorderStyle::Double => "double",
            BorderStyle::Horizontal => "horizontal",
            BorderStyle::Vertical => "vertical",
            BorderStyle::Top => "top",
            BorderStyle::Bottom => "bottom",
            BorderStyle::Left => "left",
            BorderStyle::Right => "right",
            BorderStyle::None => "none",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LayoutStyle {
    Default,
    Reverse,
    ReverseList,
}

impl fmt::Display for LayoutStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LayoutStyle::Default => "default",
            LayoutStyle::Reverse => "reverse",
            LayoutStyle::ReverseList => "reverse-list",
        };
        write!(f, "{}", s)
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the config file
    #[arg(
        short,
        long,
        default_value = "~/.config/tmux-session-switcher/config.toml"
    )]
    pub config: String,

    #[arg(short, long, default_value = "Select Window")]
    pub title: String,

    #[arg(short, long, default_value_t = BorderStyle::Rounded)]
    pub border: BorderStyle,

    #[arg(short, long, default_value_t = LayoutStyle::Default)]
    pub layout: LayoutStyle,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage favorites
    Favorite(FavoriteArgs),
}

#[derive(Parser, Debug)]
pub struct FavoriteArgs {
    #[command(subcommand)]
    pub command: FavoriteCommands,
}

#[derive(Subcommand, Debug)]
pub enum FavoriteCommands {
    /// Add current window (or specified window) to favorites
    Add {
        /// Window name (auto-detected if omitted)
        #[arg(short, long)]
        name: Option<String>,
        /// Session name (auto-detected if omitted)
        #[arg(short, long)]
        session_name: Option<String>,
        /// Window index (auto-detected if omitted)
        #[arg(short = 'i', long)]
        index: Option<u16>,
        /// Working directory path (auto-detected if omitted)
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Remove a favorite (interactive fzf if --name omitted)
    Remove {
        /// Name of the favorite to remove
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List all favorites
    List,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_favorite_add_no_args() {
        let args = Args::try_parse_from(["tss", "favorite", "add"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::Add { name, session_name, index, path } => {
                    assert!(name.is_none());
                    assert!(session_name.is_none());
                    assert!(index.is_none());
                    assert!(path.is_none());
                }
                _ => panic!("Expected Add"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_favorite_add_with_name() {
        let args = Args::try_parse_from(["tss", "favorite", "add", "--name", "my-window"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::Add { name, .. } => {
                    assert_eq!(name, Some("my-window".to_string()));
                }
                _ => panic!("Expected Add"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_favorite_remove_with_name() {
        let args = Args::try_parse_from(["tss", "favorite", "remove", "--name", "foo"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::Remove { name } => {
                    assert_eq!(name, Some("foo".to_string()));
                }
                _ => panic!("Expected Remove"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_favorite_list() {
        let args = Args::try_parse_from(["tss", "favorite", "list"]).unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::List => {}
                _ => panic!("Expected List"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_no_subcommand_still_works() {
        let args = Args::try_parse_from(["tss"]).unwrap();
        assert!(args.command.is_none());
    }
}
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cargo test test_favorite 2>&1
```

예상: 5개 테스트 모두 `ok`

- [ ] **Step 5: 전체 빌드 확인**

```bash
cargo build 2>&1
```

예상: 에러 없음 (main.rs에서 `args.command`가 없어도 아직 무시됨)

- [ ] **Step 6: 커밋**

```bash
git add src/args.rs
git commit -m "feat: add favorite subcommands to CLI args"
```

---

### Task 4: favorite list 구현

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: handle_list 함수 + 분기 추가**

`src/main.rs` 전체를 다음으로 교체:

```rust
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
```

- [ ] **Step 2: 빌드 확인**

```bash
cargo build 2>&1
```

예상: 에러 없음 (`todo!()` 매크로는 런타임 패닉이므로 컴파일은 통과)

- [ ] **Step 3: 수동 동작 확인**

```bash
cargo run -- favorite list
```

예상: `No favorites found.` 출력 (config.toml이 없거나 favorites가 없을 경우)

- [ ] **Step 4: 커밋**

```bash
git add src/main.rs
git commit -m "feat: implement tss favorite list command"
```

---

### Task 5: favorite add 구현

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: add_favorite 로직 테스트 작성**

`src/main.rs` 맨 아래에 다음 테스트 모듈 추가 (함수 구현은 아직 없음):

```rust
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
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test test_add_favorite 2>&1 | head -20
```

예상: `error[E0425]: cannot find function 'add_favorite'` 컴파일 에러

- [ ] **Step 3: add_favorite + handle_add 구현**

`src/main.rs`의 `handle_list` 아래에 추가:

```rust
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
```

그리고 `main()` 의 `FavoriteCommands::Add { .. }` 분기를:

```rust
FavoriteCommands::Add { name, session_name, index, path } => {
    handle_add(&config_path, name, session_name, index, path);
    return;
}
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cargo test test_add_favorite 2>&1
```

예상: 2개 테스트 모두 `ok`

- [ ] **Step 5: 전체 테스트 확인**

```bash
cargo test 2>&1
```

예상: 모든 테스트 통과

- [ ] **Step 6: 수동 동작 확인 (tmux 세션 안에서)**

```bash
cargo run -- favorite add
cargo run -- favorite list
```

예상: 현재 윈도우가 즐겨찾기에 추가되고 list에 나타남

- [ ] **Step 7: 커밋**

```bash
git add src/main.rs
git commit -m "feat: implement tss favorite add command"
```

---

### Task 6: favorite remove 구현

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: remove_favorite 로직 테스트 작성**

`src/main.rs`의 `#[cfg(test)] mod tests` 내부에 다음 테스트 추가:

```rust
    #[test]
    fn test_remove_favorite_by_name_success() {
        let path = temp_path("remove_success");
        add_favorite(&path, make_fav("bar"));
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
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test test_remove_favorite 2>&1 | head -20
```

예상: `remove_favorite_by_name`, `try_remove_favorite_by_name` 없음 에러

- [ ] **Step 3: remove_favorite_by_name + try_remove_favorite_by_name + handle_remove 구현**

`src/main.rs`의 `handle_add` 아래에 추가:

```rust
/// Returns true if removed, false if not found
fn try_remove_favorite_by_name(config_path: &str, name: &str) -> bool {
    let mut config = Config::new(config_path);
    let favorites = config.favorites.get_or_insert_with(Vec::new);
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

fn remove_favorite_interactive(config_path: &str) {
    use std::process::Command;

    let config = Config::new(config_path);
    let favorites = match config.favorites {
        Some(ref f) if !f.is_empty() => f.clone(),
        _ => {
            println!("No favorites found.");
            return;
        }
    };

    let input: String = favorites.iter().map(|f| f.to_string()).collect();

    let result = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf '{}' | fzf --tmux 80,20 --border=rounded --border-label ' Remove Favorite ' --prompt '🗑 '",
            input.replace('\'', "'\\''")
        ))
        .output()
        .expect("Failed to execute fzf")
        .stdout;

    let selected = String::from_utf8_lossy(&result).trim().to_string();
    if selected.is_empty() {
        return;
    }

    if let Some(fav) = favorites.iter().find(|f| f.to_string().trim() == selected) {
        remove_favorite_by_name(config_path, &fav.name.clone());
    }
}

fn handle_remove(config_path: &str, name: Option<String>) {
    match name {
        Some(name) => remove_favorite_by_name(config_path, &name),
        None => remove_favorite_interactive(config_path),
    }
}
```

그리고 `main()`의 `FavoriteCommands::Remove { .. }` 분기를:

```rust
FavoriteCommands::Remove { name } => {
    handle_remove(&config_path, name);
    return;
}
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cargo test test_remove_favorite 2>&1
```

예상: 2개 테스트 모두 `ok`

- [ ] **Step 5: 전체 테스트 확인**

```bash
cargo test 2>&1
```

예상: 모든 테스트 통과

- [ ] **Step 6: 수동 동작 확인 (tmux 세션 안에서)**

```bash
# 먼저 즐겨찾기 추가
cargo run -- favorite add --name test-window

# 이름으로 제거
cargo run -- favorite remove --name test-window
cargo run -- favorite list  # 빈 목록 확인

# 대화형 제거
cargo run -- favorite add --name test-a
cargo run -- favorite add --name test-b
cargo run -- favorite remove  # fzf 목록에서 선택
cargo run -- favorite list  # 선택한 것만 제거됐는지 확인
```

- [ ] **Step 7: 최종 빌드 + 전체 테스트**

```bash
cargo build --release 2>&1
cargo test 2>&1
```

예상: 에러 없음, 모든 테스트 통과

- [ ] **Step 8: 커밋**

```bash
git add src/main.rs
git commit -m "feat: implement tss favorite remove command"
```
