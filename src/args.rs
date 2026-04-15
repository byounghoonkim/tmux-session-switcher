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