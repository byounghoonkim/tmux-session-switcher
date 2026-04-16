use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, PartialEq)]
pub(crate) enum Action {
    InsertChar(char),
    DeleteCharBackward,
    DeleteWordBackward,
    DeleteToStart,
    CursorLeft,
    CursorRight,
    CursorToStart,
    CursorToEnd,
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Confirm,
    Cancel,
    Noop,
}

pub(crate) fn key_to_action(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        // 취소
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Cancel,
        (KeyCode::Char('g'), KeyModifiers::CONTROL) => Action::Cancel,
        (KeyCode::Esc, _) => Action::Cancel,
        // 확인
        (KeyCode::Enter, _) => Action::Confirm,
        // 위로 이동
        (KeyCode::Up, _) => Action::MoveUp,
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => Action::MoveUp,
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => Action::MoveUp,
        (KeyCode::BackTab, _) => Action::MoveUp,
        // 아래로 이동
        (KeyCode::Down, _) => Action::MoveDown,
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => Action::MoveDown,
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => Action::MoveDown,
        (KeyCode::Tab, _) => Action::MoveDown,
        // 페이지
        (KeyCode::PageUp, _) => Action::PageUp,
        (KeyCode::PageDown, _) => Action::PageDown,
        // 커서 이동
        (KeyCode::Left, _) => Action::CursorLeft,
        (KeyCode::Char('b'), KeyModifiers::CONTROL) => Action::CursorLeft,
        (KeyCode::Right, _) => Action::CursorRight,
        (KeyCode::Char('f'), KeyModifiers::CONTROL) => Action::CursorRight,
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => Action::CursorToStart,
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => Action::CursorToEnd,
        // 삭제
        (KeyCode::Backspace, _) => Action::DeleteCharBackward,
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => Action::DeleteCharBackward,
        (KeyCode::Char('w'), KeyModifiers::CONTROL) => Action::DeleteWordBackward,
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => Action::DeleteToStart,
        // 문자 입력
        (KeyCode::Char(c), KeyModifiers::NONE) => Action::InsertChar(c),
        (KeyCode::Char(c), KeyModifiers::SHIFT) => Action::InsertChar(c),
        _ => Action::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_ctrl_c_is_cancel() {
        assert_eq!(key_to_action(key(KeyCode::Char('c'), KeyModifiers::CONTROL)), Action::Cancel);
    }

    #[test]
    fn test_ctrl_g_is_cancel() {
        assert_eq!(key_to_action(key(KeyCode::Char('g'), KeyModifiers::CONTROL)), Action::Cancel);
    }

    #[test]
    fn test_esc_is_cancel() {
        assert_eq!(key_to_action(key(KeyCode::Esc, KeyModifiers::NONE)), Action::Cancel);
    }

    #[test]
    fn test_enter_is_confirm() {
        assert_eq!(key_to_action(key(KeyCode::Enter, KeyModifiers::NONE)), Action::Confirm);
    }

    #[test]
    fn test_up_is_move_up() {
        assert_eq!(key_to_action(key(KeyCode::Up, KeyModifiers::NONE)), Action::MoveUp);
    }

    #[test]
    fn test_ctrl_k_is_move_up() {
        assert_eq!(key_to_action(key(KeyCode::Char('k'), KeyModifiers::CONTROL)), Action::MoveUp);
    }

    #[test]
    fn test_ctrl_p_is_move_up() {
        assert_eq!(key_to_action(key(KeyCode::Char('p'), KeyModifiers::CONTROL)), Action::MoveUp);
    }

    #[test]
    fn test_down_is_move_down() {
        assert_eq!(key_to_action(key(KeyCode::Down, KeyModifiers::NONE)), Action::MoveDown);
    }

    #[test]
    fn test_ctrl_j_is_move_down() {
        assert_eq!(key_to_action(key(KeyCode::Char('j'), KeyModifiers::CONTROL)), Action::MoveDown);
    }

    #[test]
    fn test_ctrl_n_is_move_down() {
        assert_eq!(key_to_action(key(KeyCode::Char('n'), KeyModifiers::CONTROL)), Action::MoveDown);
    }

    #[test]
    fn test_tab_is_move_down() {
        assert_eq!(key_to_action(key(KeyCode::Tab, KeyModifiers::NONE)), Action::MoveDown);
    }

    #[test]
    fn test_backtab_is_move_up() {
        assert_eq!(key_to_action(key(KeyCode::BackTab, KeyModifiers::NONE)), Action::MoveUp);
    }

    #[test]
    fn test_pageup_is_page_up() {
        assert_eq!(key_to_action(key(KeyCode::PageUp, KeyModifiers::NONE)), Action::PageUp);
    }

    #[test]
    fn test_pagedown_is_page_down() {
        assert_eq!(key_to_action(key(KeyCode::PageDown, KeyModifiers::NONE)), Action::PageDown);
    }

    #[test]
    fn test_ctrl_a_is_cursor_to_start() {
        assert_eq!(key_to_action(key(KeyCode::Char('a'), KeyModifiers::CONTROL)), Action::CursorToStart);
    }

    #[test]
    fn test_ctrl_e_is_cursor_to_end() {
        assert_eq!(key_to_action(key(KeyCode::Char('e'), KeyModifiers::CONTROL)), Action::CursorToEnd);
    }

    #[test]
    fn test_ctrl_b_is_cursor_left() {
        assert_eq!(key_to_action(key(KeyCode::Char('b'), KeyModifiers::CONTROL)), Action::CursorLeft);
    }

    #[test]
    fn test_left_is_cursor_left() {
        assert_eq!(key_to_action(key(KeyCode::Left, KeyModifiers::NONE)), Action::CursorLeft);
    }

    #[test]
    fn test_ctrl_f_is_cursor_right() {
        assert_eq!(key_to_action(key(KeyCode::Char('f'), KeyModifiers::CONTROL)), Action::CursorRight);
    }

    #[test]
    fn test_right_is_cursor_right() {
        assert_eq!(key_to_action(key(KeyCode::Right, KeyModifiers::NONE)), Action::CursorRight);
    }

    #[test]
    fn test_backspace_is_delete_char_backward() {
        assert_eq!(key_to_action(key(KeyCode::Backspace, KeyModifiers::NONE)), Action::DeleteCharBackward);
    }

    #[test]
    fn test_ctrl_h_is_delete_char_backward() {
        assert_eq!(key_to_action(key(KeyCode::Char('h'), KeyModifiers::CONTROL)), Action::DeleteCharBackward);
    }

    #[test]
    fn test_ctrl_w_is_delete_word_backward() {
        assert_eq!(key_to_action(key(KeyCode::Char('w'), KeyModifiers::CONTROL)), Action::DeleteWordBackward);
    }

    #[test]
    fn test_ctrl_u_is_delete_to_start() {
        assert_eq!(key_to_action(key(KeyCode::Char('u'), KeyModifiers::CONTROL)), Action::DeleteToStart);
    }

    #[test]
    fn test_printable_char_is_insert() {
        assert_eq!(key_to_action(key(KeyCode::Char('x'), KeyModifiers::NONE)), Action::InsertChar('x'));
    }

    #[test]
    fn test_shift_char_is_insert() {
        assert_eq!(key_to_action(key(KeyCode::Char('X'), KeyModifiers::SHIFT)), Action::InsertChar('X'));
    }
}
