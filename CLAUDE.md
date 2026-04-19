# CLAUDE.md — tmux-session-switcher

## Project Overview

tmux-session-switcher is a Rust CLI that replaces the default tmux session list with a fuzzy-searchable window picker. It runs as a tmux `display-popup` subprocess and communicates results via temp files.

## Architecture

```
main.rs           Entry point: parses args, builds item list, calls select_item()
  └─ fzf.rs       Picker dispatcher: PickerConfig, dispatch_picker(), select_item()
       ├─ invoke_picker()  Spawns self as `internal-picker` via tmux display-popup
       └─ invoke_fzf()     Runs fzf in a tmux popup
  └─ picker/      Native TUI picker (runs inside the popup subprocess)
       ├─ mod.rs   Entry: run(), PickerConfig (IPC struct), PickerResult
       ├─ state.rs Query, cursor, filtered list state
       ├─ filter.rs Fuzzy matching via nucleo-matcher
       ├─ input.rs  Key event → Action mapping
       ├─ ui.rs     ratatui rendering
       └─ theme.rs  Color themes (Theme struct, parse_hex_color)
  └─ tmux/        tmux data types and commands
       ├─ mod.rs   run_command(), get_running_windows(), format_window_base()
       ├─ window.rs Window struct + Display + Switchable + SortPriority
       ├─ favorite.rs Saved favorite windows
       └─ previous.rs PreviousWindow struct + JSON persistence
  └─ config.rs    Config struct (TOML: favorites, picker, theme, bell_fg)
  └─ args.rs      CLI argument definitions (clap)
  └─ utils.rs     expand_tilde(), get_config_dir()
```

## Module Responsibilities

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Orchestration: build item list → sort → dispatch picker → switch window |
| `fzf.rs` | Public picker API: `PickerConfig`, `dispatch_picker`, `select_item` |
| `picker/` | Native TUI picker (crossterm + ratatui), only used inside popup |
| `tmux/mod.rs` | tmux command execution, window list parsing, format helpers |
| `tmux/window.rs` | Running window data and switching |
| `tmux/favorite.rs` | Saved favorite windows |
| `tmux/previous.rs` | Last active window tracking |
| `config.rs` | TOML config file read/write |
| `args.rs` | CLI argument parsing |
| `utils.rs` | Path utilities |

## Two-Process Architecture

The native picker uses a two-process design to avoid terminal control conflicts:

1. **Outer process** (`main.rs`): serializes config+items to a temp file, spawns `tmux display-popup` running `self internal-picker <items_path> <result_path>`
2. **Inner process** (`picker/mod.rs`): reads config, runs ratatui TUI, writes result to result_path
3. **Outer process**: reads result file, interprets Selected/New/Cancelled

## Running Tests

```bash
cargo test              # run all tests
cargo test -- --nocapture  # show println output
cargo test test_name    # run a specific test
cargo test --test integration_test  # run integration tests only
```

## Adding a New Theme

1. Add a new `fn my_theme() -> Self` in `src/picker/theme.rs`
2. Add a match arm in `Theme::from_name()`:
   ```rust
   "my-theme" => Self::my_theme(),
   ```
3. Add a test verifying `bell_fg != Color::Reset`
4. Document it in README.md themes table and CLAUDE.md (if needed)

## Development Workflow

```bash
cargo build             # debug build
cargo build --release   # optimized build
cargo clippy -- -D warnings  # lint (must pass clean)
cargo test              # must pass before commit
```

Config file for development: `~/.config/tmux-session-switcher/config.toml`
