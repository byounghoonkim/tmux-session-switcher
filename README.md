# tmux-session-switcher

A fast tmux window/session switcher with a native TUI picker and optional fzf backend.

## Requirements

- tmux ≥ 3.2 (for `display-popup`)
- Rust ≥ 1.70 (for building from source)
- fzf (optional — only required when using `--picker fzf`)

## Installation

### From source

```bash
git clone https://github.com/yourusername/tmux-session-switcher
cd tmux-session-switcher
cargo build --release
cp target/release/tmux-session-switcher ~/.local/bin/
```

### Recommended tmux binding

Add to `~/.tmux.conf`:

```tmux
bind-key s run-shell "tmux-session-switcher"
```

## Usage

```bash
tmux-session-switcher [OPTIONS] [COMMAND]
```

Run with no arguments to open the window picker.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--picker <native\|fzf>` | `native` | Picker backend |
| `--theme <name>` | `catppuccin` | Color theme |
| `--title <text>` | `Switch` | Popup title |
| `--border <style>` | `rounded` | Border style (`rounded`, `double`, `bold`, `sharp`, `none`) |
| `--layout <style>` | `default` | Layout (`default` = prompt top; `reverse` = prompt bottom) |
| `--config <path>` | `~/.config/tmux-session-switcher/config.toml` | Config file path |

## Config file

`~/.config/tmux-session-switcher/config.toml`:

```toml
picker = "native"   # or "fzf"
theme  = "nord"     # catppuccin, nord, gruvbox, tokyo-night, solarized-dark
bell_fg = "#ff8c00" # override bell row highlight color (hex, native picker only)
```

## Themes

| Name | Description |
|------|-------------|
| `catppuccin` / `catppuccin-mocha` | Catppuccin Mocha palette (default) |
| `nord` | Nord arctic palette |
| `gruvbox` | Gruvbox dark palette |
| `tokyo-night` / `tokyonight` | Tokyo Night palette |
| `solarized-dark` | Solarized Dark palette |

## Key bindings (native picker)

| Key | Action |
|-----|--------|
| `↑` / `Ctrl-k` / `Ctrl-p` / `Shift-Tab` | Move up |
| `↓` / `Ctrl-j` / `Ctrl-n` / `Tab` | Move down |
| `Page Up` | Jump 10 items up |
| `Page Down` | Jump 10 items down |
| `Enter` | Select / create new window |
| `Esc` / `Ctrl-c` / `Ctrl-g` | Cancel |
| `←` / `Ctrl-b` | Cursor left |
| `→` / `Ctrl-f` | Cursor right |
| `Ctrl-a` | Cursor to start |
| `Ctrl-e` | Cursor to end |
| `Backspace` / `Ctrl-h` | Delete char backward |
| `Ctrl-w` | Delete word backward |
| `Ctrl-u` | Delete to start |

## fzf vs native picker

| Feature | native | fzf |
|---------|--------|-----|
| Bell row color (`bell_fg`) | ✅ | ❌ (fzf uses its own color system) |
| Fuzzy matching | ✅ (nucleo) | ✅ |
| New window on unmatched query | ✅ | ✅ |
| Requires external binary | No | Yes (`fzf` in PATH) |

## Favorites

```bash
# List favorites
tmux-session-switcher favorite list

# Add current window as favorite
tmux-session-switcher favorite add --name mywork

# Add specific window
tmux-session-switcher favorite add --name mywork --session main --index 2 --path /home/user/work

# Remove by name
tmux-session-switcher favorite remove --name mywork

# Remove interactively (opens picker)
tmux-session-switcher favorite remove
```

## Window list format

```
mysession       -   3 - editor 🟢
othersession    -   1 - shell  🔔
```

Icons: 🟢 active window · ♥️ marked · 🔔 bell · ⭐️ favorite · 🔙 previous

## License

MIT
