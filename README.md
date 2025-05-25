# tmux-session-switcher

A simple tool to switch between sessions using a fuzzy finder.

## Installation

```bash
git clone
cd tmux-session-switcher
cargo build --release
chmod +x target/release/tmux-session-switcher
sudo mv target/release/tmux-session-switcher /usr/local/bin/
```

## Usage

```bash
tmux-session-switcher
```

## Requirements

- `tmux`
- `fzf`

## License

This project is licensed under the MIT License -
see the [LICENSE](LICENSE) file for details.
