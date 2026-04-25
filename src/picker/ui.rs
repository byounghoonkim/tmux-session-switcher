use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::state::PickerState;
use super::theme::Theme;

const LAYOUT_REVERSE: &str = "reverse";

fn truncate_to_width(s: &str, max_width: usize) -> std::borrow::Cow<str> {
    use std::borrow::Cow;
    if UnicodeWidthStr::width(s) <= max_width {
        return Cow::Borrowed(s);
    }
    let target = max_width.saturating_sub(1); // reserve 1 col for "…"
    let mut width = 0usize;
    let mut byte_end = 0usize;
    for (byte_pos, ch) in s.char_indices() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
        if width + cw > target {
            break;
        }
        width += cw;
        byte_end = byte_pos + ch.len_utf8();
    }
    Cow::Owned(format!("{}…", &s[..byte_end]))
}

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

    // Available width for item text (inner width minus 2 for "> " highlight symbol)
    let item_width = (inner.width as usize).saturating_sub(2);

    // List items with fuzzy match character highlighting.
    // List::highlight_style overwrites Span styles, so we apply highlight_bg/fg directly to each Span.
    let selected_rank = list_state.selected().unwrap_or(0);
    let list_items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(rank, &i)| {
            let raw_text = state.items[i].trim_end();
            let text = truncate_to_width(raw_text, item_width);
            // Drop match positions beyond the visible (non-ellipsis) chars
            let truncated = matches!(text, std::borrow::Cow::Owned(_));
            let visible_chars = if truncated {
                text.chars().count().saturating_sub(1)
            } else {
                text.chars().count()
            };
            let raw_positions = state.match_indices.get(rank).map(|v| v.as_slice()).unwrap_or(&[]);
            let filtered_positions: Vec<u32> = raw_positions
                .iter()
                .copied()
                .filter(|&p| (p as usize) < visible_chars)
                .collect();
            let is_bell = raw_text.contains('🔔');
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
            let line = highlight_spans(&text, &filtered_positions, normal_style, match_style);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(list_items).highlight_symbol("> ");

    // Empty state: shown instead of the list when nothing matches
    let empty_msg = {
        let mut lines = vec![Line::from(Span::styled(
            "  No matches",
            Style::default().fg(theme.status_fg),
        ))];
        if !state.query.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  Press Enter to create '{}'", state.query),
                Style::default().fg(theme.status_fg),
            )));
        }
        Paragraph::new(lines)
    };

    // Status bar: [selected_pos/filtered_count] total_count total
    let pos = if state.filtered.is_empty() { 0 } else { state.selected + 1 };
    let status_text = format!(
        "  [{}/{}] {} total",
        pos,
        state.filtered.len(),
        state.items.len(),
    );
    let status = Paragraph::new(status_text).style(Style::default().fg(theme.status_fg));

    let prompt_chunk = if layout == LAYOUT_REVERSE {
        // reverse: list at top, prompt at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status
                Constraint::Min(1),    // list or empty msg
                Constraint::Length(1), // separator
                Constraint::Length(1), // prompt
            ])
            .split(inner);

        frame.render_widget(status, chunks[0]);
        if state.filtered.is_empty() {
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            frame.render_stateful_widget(list, chunks[1], list_state);
        }
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
                Constraint::Min(1),    // list or empty msg
                Constraint::Length(1), // status
            ])
            .split(inner);

        frame.render_widget(prompt, chunks[0]);
        frame.render_widget(sep, chunks[1]);
        if state.filtered.is_empty() {
            frame.render_widget(empty_msg, chunks[2]);
        } else {
            frame.render_stateful_widget(list, chunks[2], list_state);
        }
        frame.render_widget(status, chunks[3]);
        chunks[0]
    };

    // Wide-char-aware cursor position
    let visual_col = UnicodeWidthStr::width(&state.query[..state.cursor]) as u16;
    frame.set_cursor_position((prompt_chunk.x + 2 + visual_col, prompt_chunk.y));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string_unchanged() {
        let result = truncate_to_width("hello", 10);
        assert_eq!(result, "hello");
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)), "no truncation must return Cow::Borrowed");
    }

    #[test]
    fn test_truncate_exact_fit_unchanged() {
        let result = truncate_to_width("hello", 5);
        assert_eq!(result, "hello");
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)), "exact fit must return Cow::Borrowed");
    }

    #[test]
    fn test_truncate_ascii_adds_ellipsis() {
        // max=8, target=7: "hello w" fits (7 cols), "hello w…" = 8 display cols
        assert_eq!(truncate_to_width("hello world", 8), "hello w…");
    }

    #[test]
    fn test_truncate_wide_chars() {
        // "你好世界" — each CJK char is 2 cols wide (total 8 cols)
        // max=5, target=4: "你好" = 4 cols fits, next "世" would be 6 → "你好…"
        assert_eq!(truncate_to_width("你好世界", 5), "你好…");
    }

    #[test]
    fn test_truncate_empty_string_unchanged() {
        assert_eq!(truncate_to_width("", 5), "");
    }

    #[test]
    fn test_truncated_detection_three_byte_char_boundary() {
        // "a你" = 4 bytes (1 + 3). max_width=2 → target=1: 'a' fits (1 col), '你' (2 col wide) exceeds → "a…" = 4 bytes
        // old code: "a…".len() == 4 == "a你".len() → false negative (fails to detect truncation)
        // new code: Cow::Owned variant → correctly detects truncation
        let raw = "a你";
        let truncated_text = truncate_to_width(raw, 2);
        assert_eq!(truncated_text, "a…");
        assert!(matches!(truncated_text, std::borrow::Cow::Owned(_)), "truncation must return Cow::Owned");
        // byte lengths are equal, demonstrating why content comparison (not byte-length) is needed:
        assert_eq!(truncated_text.len(), raw.len(), "byte lengths are equal — old byte-length check would fail");
    }
}
