use clap::{Parser, ValueEnum};
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
}