mod filter;
mod input;
mod state;
pub(crate) mod theme;
mod ui;

use std::io;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};
use serde::{Deserialize, Serialize};

use filter::FuzzyFilter;
use input::{Action, key_to_action};
use state::PickerState;
use theme::Theme;

#[derive(Serialize, Deserialize)]
pub(crate) struct PickerConfig {
    pub items: Vec<String>,
    pub title: String,
    pub border: String,
    pub layout: String,
    pub theme: String,
    pub bell_fg: Option<String>,
}

pub(crate) enum PickerResult {
    Selected(usize),   // index into items
    New(String),       // unmatched query → new window name
    Cancelled,
}

/// TUI picker that runs inside a tmux display-popup.
/// Takes a PickerConfig and returns the user's selection.
pub(crate) fn run(config: PickerConfig) -> PickerResult {
    // Hook to restore terminal state if a panic occurs.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

    let result = run_loop(&mut terminal, config);

    disable_raw_mode().expect("Failed to disable raw mode");
    execute!(terminal.backend_mut(), LeaveAlternateScreen).expect("Failed to leave alternate screen");

    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: PickerConfig,
) -> PickerResult {
    // Destructure config before moving items into PickerState.
    let PickerConfig { items, layout, theme: theme_name, bell_fg, .. } = config;
    let mut theme = Theme::from_name(&theme_name);
    if let Some(ref hex) = bell_fg {
        if let Some(color) = theme::parse_hex_color(hex) {
            theme.bell_fg = color;
        }
    }
    let mut state = PickerState::new(items);
    let mut filter = FuzzyFilter::new();
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal
            .draw(|f| ui::render(f, &state, &layout, &theme, &mut list_state))
            .expect("Failed to draw");

        if let Event::Key(key) = event::read().expect("Failed to read event") {
            match key_to_action(key) {
                Action::Cancel => return PickerResult::Cancelled,

                Action::Confirm => {
                    return if let Some(idx) = state.selected_item_index() {
                        PickerResult::Selected(idx)
                    } else if !state.query.is_empty() {
                        PickerResult::New(state.query.clone())
                    } else {
                        PickerResult::Cancelled
                    };
                }

                Action::InsertChar(c) => {
                    state.insert_char(c);
                    refilter(&mut filter, &mut state, &mut list_state);
                }
                Action::DeleteCharBackward => {
                    state.delete_char_backward();
                    refilter(&mut filter, &mut state, &mut list_state);
                }
                Action::DeleteWordBackward => {
                    state.delete_word_backward();
                    refilter(&mut filter, &mut state, &mut list_state);
                }
                Action::DeleteToStart => {
                    state.delete_to_start();
                    refilter(&mut filter, &mut state, &mut list_state);
                }

                Action::CursorLeft => state.cursor_left(),
                Action::CursorRight => state.cursor_right(),
                Action::CursorToStart => state.cursor_to_start(),
                Action::CursorToEnd => state.cursor_to_end(),

                Action::MoveUp => {
                    state.move_up();
                    list_state.select(Some(state.selected));
                }
                Action::MoveDown => {
                    state.move_down();
                    list_state.select(Some(state.selected));
                }
                Action::PageUp => {
                    state.page_up(10);
                    list_state.select(Some(state.selected));
                }
                Action::PageDown => {
                    state.page_down(10);
                    list_state.select(Some(state.selected));
                }

                Action::Noop => {}
            }
        }
    }
}

fn refilter(filter: &mut FuzzyFilter, state: &mut PickerState, list_state: &mut ListState) {
    let results = filter.filter_with_indices(&state.query, &state.items);
    state.update_filter_full(results);
    list_state.select(Some(state.selected));
}
