use ratatui::style::Color;

pub(crate) struct Theme {
    pub prompt_fg: Color,
    pub separator_fg: Color,
    pub status_fg: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub item_fg: Color,
    pub match_fg: Color,
}

impl Theme {
    pub(crate) fn from_name(name: &str) -> Self {
        match name {
            "catppuccin" | "catppuccin-mocha" => Self::catppuccin_mocha(),
            "nord" => Self::nord(),
            "gruvbox" => Self::gruvbox(),
            "tokyo-night" | "tokyonight" => Self::tokyo_night(),
            "solarized" | "solarized-dark" => Self::solarized_dark(),
            _ => Self::default_theme(),
        }
    }

    fn catppuccin_mocha() -> Self {
        Self {
            prompt_fg: Color::Rgb(137, 180, 250),   // Blue
            separator_fg: Color::Rgb(108, 112, 134), // Overlay0
            status_fg: Color::Rgb(166, 173, 200),    // Subtext0
            highlight_bg: Color::Rgb(69, 71, 90),    // Surface1
            highlight_fg: Color::Rgb(203, 166, 247), // Mauve
            item_fg: Color::Rgb(205, 214, 244),      // Text
            match_fg: Color::Rgb(249, 226, 175),     // Yellow
        }
    }

    fn nord() -> Self {
        Self {
            prompt_fg: Color::Rgb(136, 192, 208),    // nord8
            separator_fg: Color::Rgb(76, 86, 106),   // nord3
            status_fg: Color::Rgb(97, 110, 136),     // nord3/4 사이
            highlight_bg: Color::Rgb(59, 66, 82),    // nord1
            highlight_fg: Color::Rgb(136, 192, 208), // nord8
            item_fg: Color::Rgb(216, 222, 233),      // nord4
            match_fg: Color::Rgb(235, 203, 139),     // nord13 yellow
        }
    }

    fn gruvbox() -> Self {
        Self {
            prompt_fg: Color::Rgb(131, 165, 152),   // aqua
            separator_fg: Color::Rgb(102, 92, 84),  // bg4
            status_fg: Color::Rgb(146, 131, 116),   // gray
            highlight_bg: Color::Rgb(60, 56, 54),   // bg1
            highlight_fg: Color::Rgb(250, 189, 47), // yellow
            item_fg: Color::Rgb(235, 219, 178),     // fg1
            match_fg: Color::Rgb(254, 128, 25),     // orange
        }
    }

    fn tokyo_night() -> Self {
        Self {
            prompt_fg: Color::Rgb(122, 162, 247),    // blue
            separator_fg: Color::Rgb(65, 72, 104),   // overlay
            status_fg: Color::Rgb(86, 95, 137),      // comment
            highlight_bg: Color::Rgb(36, 40, 59),    // surface
            highlight_fg: Color::Rgb(187, 154, 247), // purple
            item_fg: Color::Rgb(192, 202, 245),      // text
            match_fg: Color::Rgb(224, 175, 104),     // yellow
        }
    }

    fn solarized_dark() -> Self {
        Self {
            prompt_fg: Color::Rgb(38, 139, 210),    // blue
            separator_fg: Color::Rgb(88, 110, 117), // base01
            status_fg: Color::Rgb(101, 123, 131),   // base00
            highlight_bg: Color::Rgb(7, 54, 66),    // base02
            highlight_fg: Color::Rgb(42, 161, 152), // cyan
            item_fg: Color::Rgb(131, 148, 150),     // base0
            match_fg: Color::Rgb(181, 137, 0),      // yellow
        }
    }

    fn default_theme() -> Self {
        Self {
            prompt_fg: Color::Reset,
            separator_fg: Color::DarkGray,
            status_fg: Color::DarkGray,
            highlight_bg: Color::Blue,
            highlight_fg: Color::White,
            item_fg: Color::Reset,
            match_fg: Color::Yellow,
        }
    }
}
