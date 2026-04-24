use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use super::state::PickerState;
use super::theme::Theme;

/// Splits text into normal/match Spans based on match positions.
fn highlight_spans<'a>(
    text: &str,
    positions: &[u32],
    normal: Style,
    match_s: Style,
) -> Line<'a> {
    if positions.is_empty() {
        return Line::from(Span::styled(text.to_string(), normal));
    }
    let matched: std::collections::HashSet<u32> = positions.iter().copied().collect();
    let mut spans = Vec::new();
    let mut current_is_match = false;
    let mut current_text = String::new();

    for (i, ch) in text.chars().enumerate() {
        let is_match = matched.contains(&(i as u32));
        if is_match != current_is_match && !current_text.is_empty() {
            let style = if current_is_match { match_s } else { normal };
            spans.push(Span::styled(current_text.clone(), style));
            current_text.clear();
        }
        current_is_match = is_match;
        current_text.push(ch);
    }
    if !current_text.is_empty() {
        let style = if current_is_match { match_s } else { normal };
        spans.push(Span::styled(current_text, style));
    }
    Line::from(spans)
}

/// layout = "default": prompt at top, list below
/// layout = "reverse": prompt at bottom, list above (mirrors fzf --layout=reverse)
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    layout: &str,
    theme: &Theme,
    list_state: &mut ListState,
) {
    let area = frame.area();
    let inner = Block::default().inner(area);

    // Prompt area: ">" + current query
    let prompt_text = format!("> {}", state.query);
    let prompt = Paragraph::new(prompt_text).style(Style::default().fg(theme.prompt_fg));

    // Separator line
    let sep_char = "─".repeat(inner.width as usize);
    let sep = Paragraph::new(sep_char).style(Style::default().fg(theme.separator_fg));

    // List items with fuzzy match character highlighting.
    // List::highlight_style overwrites Span styles, so we apply highlight_bg/fg directly to each Span.
    let selected_rank = list_state.selected().unwrap_or(0);
    let list_items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(rank, &i)| {
            let text = state.items[i].trim_end();
            let positions = state.match_indices.get(rank).map(|v| v.as_slice()).unwrap_or(&[]);
            let is_bell = text.contains('🔔');
            let (normal_style, match_style) = if rank == selected_rank {
                let fg = if is_bell { theme.bell_fg } else { theme.highlight_fg };
                (
                    Style::default().fg(fg).bg(theme.highlight_bg),
                    Style::default()
                        .fg(theme.match_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                let fg = if is_bell { theme.bell_fg } else { theme.item_fg };
                (
                    Style::default().fg(fg),
                    Style::default().fg(theme.match_fg).add_modifier(Modifier::BOLD),
                )
            };
            let line = highlight_spans(text, positions, normal_style, match_style);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(list_items).highlight_symbol("> ");

    // Status bar
    let status_text = format!("  {}/{}", state.filtered.len(), state.items.len());
    let status = Paragraph::new(status_text).style(Style::default().fg(theme.status_fg));

    let prompt_chunk = if layout == "reverse" {
        // reverse: list at top, prompt at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status
                Constraint::Min(1),    // list
                Constraint::Length(1), // separator
                Constraint::Length(1), // prompt
            ])
            .split(inner);

        frame.render_widget(status, chunks[0]);
        frame.render_stateful_widget(list, chunks[1], list_state);
        frame.render_widget(sep, chunks[2]);
        frame.render_widget(prompt, chunks[3]);
        chunks[3]
    } else {
        // default: prompt at top, list at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // prompt
                Constraint::Length(1), // separator
                Constraint::Min(1),    // list
                Constraint::Length(1), // status
            ])
            .split(inner);

        frame.render_widget(prompt, chunks[0]);
        frame.render_widget(sep, chunks[1]);
        frame.render_stateful_widget(list, chunks[2], list_state);
        frame.render_widget(status, chunks[3]);
        chunks[0]
    };

    // Render cursor at current query position
    let visual_col = state.query[..state.cursor].chars().count() as u16;
    frame.set_cursor_position((prompt_chunk.x + 2 + visual_col, prompt_chunk.y));
}
