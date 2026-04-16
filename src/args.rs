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
pub enum PickerBackend {
    Native,
    Fzf,
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

    /// Picker backend: native (ratatui) or fzf
    #[arg(long, value_enum)]
    pub picker: Option<PickerBackend>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage favorites
    Favorite(FavoriteArgs),
    /// Internal: run TUI picker inside tmux display-popup (not for direct use)
    #[command(hide = true)]
    InternalPicker {
        items_path: String,
        result_path: String,
    },
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

    #[test]
    fn test_picker_flag_native() {
        let args = Args::try_parse_from(["tss", "--picker", "native"]).unwrap();
        assert!(matches!(args.picker, Some(PickerBackend::Native)));
    }

    #[test]
    fn test_picker_flag_fzf() {
        let args = Args::try_parse_from(["tss", "--picker", "fzf"]).unwrap();
        assert!(matches!(args.picker, Some(PickerBackend::Fzf)));
    }

    #[test]
    fn test_picker_flag_absent() {
        let args = Args::try_parse_from(["tss"]).unwrap();
        assert!(args.picker.is_none());
    }

    #[test]
    fn test_favorite_add_with_all_short_flags() {
        let args = Args::try_parse_from([
            "tss", "favorite", "add",
            "-n", "mywin",
            "-s", "main",
            "-i", "2",
            "-p", "/home/user",
        ])
        .unwrap();
        match args.command {
            Some(Commands::Favorite(fa)) => match fa.command {
                FavoriteCommands::Add { name, session_name, index, path } => {
                    assert_eq!(name, Some("mywin".to_string()));
                    assert_eq!(session_name, Some("main".to_string()));
                    assert_eq!(index, Some(2u16));
                    assert_eq!(path, Some("/home/user".to_string()));
                }
                _ => panic!("Expected Add"),
            },
            _ => panic!("Expected Favorite"),
        }
    }

    #[test]
    fn test_favorite_add_invalid_index_fails() {
        let result = Args::try_parse_from(["tss", "favorite", "add", "--index", "notanumber"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_internal_picker_subcommand() {
        let args = Args::try_parse_from([
            "tss", "internal-picker", "/tmp/items.json", "/tmp/result.txt",
        ])
        .unwrap();
        match args.command {
            Some(Commands::InternalPicker { items_path, result_path }) => {
                assert_eq!(items_path, "/tmp/items.json");
                assert_eq!(result_path, "/tmp/result.txt");
            }
            _ => panic!("Expected InternalPicker"),
        }
    }
}