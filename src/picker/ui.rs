use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use super::state::PickerState;
use super::theme::Theme;

/// 텍스트를 매칭 위치에 따라 일반/하이라이트 Span으로 분리
fn highlight_spans<'a>(text: &str, positions: &[u32], normal: Style, highlight: Style) -> Line<'a> {
    if positions.is_empty() {
        return Line::from(Span::styled(text.to_string(), normal));
    }
    let matched: std::collections::HashSet<u32> = positions.iter().copied().collect();
    let mut spans = Vec::new();
    let mut current_highlighted = false;
    let mut current_text = String::new();

    for (i, ch) in text.chars().enumerate() {
        let is_matched = matched.contains(&(i as u32));
        if is_matched != current_highlighted && !current_text.is_empty() {
            let style = if current_highlighted { highlight } else { normal };
            spans.push(Span::styled(current_text.clone(), style));
            current_text.clear();
        }
        current_highlighted = is_matched;
        current_text.push(ch);
    }
    if !current_text.is_empty() {
        let style = if current_highlighted { highlight } else { normal };
        spans.push(Span::styled(current_text, style));
    }
    Line::from(spans)
}

/// layout = "default": 프롬프트 상단, 리스트 하단
/// layout = "reverse": 프롬프트 하단, 리스트 상단 (fzf --layout=reverse 동작)
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    _title: &str,
    _border: &str,
    layout: &str,
    theme: &Theme,
    list_state: &mut ListState,
) {
    let area = frame.area();
    let inner = Block::default().inner(area);

    // 프롬프트 영역: ">" + 쿼리
    let prompt_text = format!("> {}", state.query);
    let prompt = Paragraph::new(prompt_text).style(Style::default().fg(theme.prompt_fg));

    // 구분선
    let sep_char = "─".repeat(inner.width as usize);
    let sep = Paragraph::new(sep_char).style(Style::default().fg(theme.separator_fg));

    // 리스트 아이템 (매칭 글자 하이라이팅)
    let normal_style = Style::default().fg(theme.item_fg);
    let match_style = Style::default().fg(theme.match_fg).add_modifier(Modifier::BOLD);
    let list_items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(rank, &i)| {
            let text = state.items[i].trim_end();
            let positions = state.match_indices.get(rank).map(|v| v.as_slice()).unwrap_or(&[]);
            let line = highlight_spans(text, positions, normal_style, match_style);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(theme.highlight_bg)
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // 상태 표시줄
    let status_text = format!("  {}/{}", state.filtered.len(), state.items.len());
    let status = Paragraph::new(status_text).style(Style::default().fg(theme.status_fg));

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
