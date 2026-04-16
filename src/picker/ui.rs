use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use super::state::PickerState;

/// layout = "default": 프롬프트 상단, 리스트 하단
/// layout = "reverse": 프롬프트 하단, 리스트 상단 (fzf --layout=reverse 동작)
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    _title: &str,
    _border: &str,
    layout: &str,
    list_state: &mut ListState,
) {
    let area = frame.area();
    let inner = Block::default().inner(area);

    // 프롬프트 영역: ">" + 쿼리
    let prompt_text = format!("> {}", state.query);
    let prompt = Paragraph::new(prompt_text);

    // 구분선
    let sep_char = "─".repeat(inner.width as usize);
    let sep = Paragraph::new(sep_char).style(Style::default().fg(Color::DarkGray));

    // 리스트 아이템
    let list_items: Vec<ListItem> = state
        .filtered
        .iter()
        .map(|&i| ListItem::new(Line::from(state.items[i].trim_end().to_string())))
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // 상태 표시줄
    let status_text = format!("  {}/{}", state.filtered.len(), state.items.len());
    let status = Paragraph::new(status_text).style(Style::default().fg(Color::DarkGray));

    let prompt_chunk;
    if layout == "reverse" {
        // reverse: 리스트 상단, 프롬프트 하단
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
        prompt_chunk = chunks[3];
    } else {
        // default: 프롬프트 상단, 리스트 하단
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
        prompt_chunk = chunks[0];
    }

    // Render cursor at current query position
    let visual_col = state.query[..state.cursor].chars().count() as u16;
    frame.set_cursor_position((prompt_chunk.x + 2 + visual_col, prompt_chunk.y));
}
