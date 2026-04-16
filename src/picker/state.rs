pub(crate) struct PickerState {
    pub query: String,
    pub cursor: usize,
    pub selected: usize,
    pub filtered: Vec<usize>,
    pub match_indices: Vec<Vec<u32>>,
    pub items: Vec<String>,
}

impl PickerState {
    pub(crate) fn new(items: Vec<String>) -> Self {
        let filtered = (0..items.len()).collect();
        let match_indices = (0..items.len()).map(|_| vec![]).collect();
        Self {
            query: String::new(),
            cursor: 0,
            selected: 0,
            filtered,
            match_indices,
            items,
        }
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        self.query.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub(crate) fn delete_char_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = self.query[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.query.drain(prev..self.cursor);
        self.cursor = prev;
    }

    /// Delete the word (plus trailing space) before the cursor. Matches fzf Ctrl-W behavior.
    pub(crate) fn delete_word_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let new_cursor = self.query[..self.cursor].rfind(' ').unwrap_or(0);
        self.query.drain(new_cursor..self.cursor);
        self.cursor = new_cursor;
    }

    pub(crate) fn delete_to_start(&mut self) {
        self.query.drain(..self.cursor);
        self.cursor = 0;
    }

    pub(crate) fn cursor_left(&mut self) {
        if self.cursor > 0 {
            let prev = self.query[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.cursor = prev;
        }
    }

    pub(crate) fn cursor_right(&mut self) {
        if self.cursor < self.query.len() {
            let c = self.query[self.cursor..].chars().next().unwrap();
            self.cursor += c.len_utf8();
        }
    }

    pub(crate) fn cursor_to_start(&mut self) {
        self.cursor = 0;
    }

    pub(crate) fn cursor_to_end(&mut self) {
        self.cursor = self.query.len();
    }

    pub(crate) fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub(crate) fn move_down(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub(crate) fn page_up(&mut self, page_size: usize) {
        self.selected = self.selected.saturating_sub(page_size);
    }

    pub(crate) fn page_down(&mut self, page_size: usize) {
        let max = self.filtered.len().saturating_sub(1);
        self.selected = (self.selected + page_size).min(max);
    }

    pub(crate) fn update_filter(&mut self, filtered: Vec<usize>) {
        self.match_indices = (0..filtered.len()).map(|_| vec![]).collect();
        self.filtered = filtered;
        let max = self.filtered.len().saturating_sub(1);
        if self.selected > max {
            self.selected = max;
        }
        // When filtered is empty, selected is 0 but selected_item_index returns None.
    }

    pub(crate) fn update_filter_full(&mut self, results: Vec<(usize, Vec<u32>)>) {
        let max = results.len().saturating_sub(1);
        self.filtered = results.iter().map(|(i, _)| *i).collect();
        self.match_indices = results.into_iter().map(|(_, m)| m).collect();
        if self.selected > max {
            self.selected = max;
        }
    }

    pub(crate) fn selected_item_index(&self) -> Option<usize> {
        self.filtered.get(self.selected).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(items: &[&str]) -> PickerState {
        PickerState::new(items.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn test_new_state() {
        let s = make(&["a", "b", "c"]);
        assert_eq!(s.query, "");
        assert_eq!(s.cursor, 0);
        assert_eq!(s.selected, 0);
        assert_eq!(s.filtered, vec![0, 1, 2]);
    }

    #[test]
    fn test_insert_char() {
        let mut s = make(&[]);
        s.insert_char('a');
        s.insert_char('b');
        assert_eq!(s.query, "ab");
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn test_delete_char_backward() {
        let mut s = make(&[]);
        s.insert_char('a');
        s.insert_char('b');
        s.delete_char_backward();
        assert_eq!(s.query, "a");
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn test_delete_char_backward_at_start_is_noop() {
        let mut s = make(&[]);
        s.delete_char_backward();
        assert_eq!(s.query, "");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_delete_word_backward() {
        let mut s = make(&[]);
        for c in "foo bar".chars() { s.insert_char(c); }
        s.delete_word_backward();
        assert_eq!(s.query, "foo");
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn test_delete_to_start() {
        let mut s = make(&[]);
        for c in "hello".chars() { s.insert_char(c); }
        s.delete_to_start();
        assert_eq!(s.query, "");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut s = make(&[]);
        for c in "ab".chars() { s.insert_char(c); }
        s.cursor_left();
        assert_eq!(s.cursor, 1);
        s.cursor_right();
        assert_eq!(s.cursor, 2);
        s.cursor_to_start();
        assert_eq!(s.cursor, 0);
        s.cursor_to_end();
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn test_cursor_left_at_start_is_noop() {
        let mut s = make(&[]);
        s.cursor_left();
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_move_up_down() {
        let mut s = make(&["a", "b", "c"]);
        assert_eq!(s.selected, 0);
        s.move_down();
        assert_eq!(s.selected, 1);
        s.move_down();
        assert_eq!(s.selected, 2);
        s.move_down();
        assert_eq!(s.selected, 2);
        s.move_up();
        assert_eq!(s.selected, 1);
        s.move_up();
        s.move_up();
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn test_page_up_down() {
        let items: Vec<&str> = vec!["x"; 20];
        let mut s = make(&items);
        s.selected = 10;
        s.page_up(5);
        assert_eq!(s.selected, 5);
        s.page_down(8);
        assert_eq!(s.selected, 13);
        s.page_down(100);
        assert_eq!(s.selected, 19);
        s.page_up(100);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn test_update_filter_clamps_selected() {
        let mut s = make(&["a", "b", "c"]);
        s.selected = 2;
        s.update_filter(vec![0]);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn test_selected_item_index() {
        let mut s = make(&["a", "b", "c"]);
        s.update_filter(vec![2, 0]);
        assert_eq!(s.selected_item_index(), Some(2));
        s.move_down();
        assert_eq!(s.selected_item_index(), Some(0));
    }

    #[test]
    fn test_selected_item_index_empty_filter() {
        let mut s = make(&["a", "b"]);
        s.update_filter(vec![]);
        assert_eq!(s.selected_item_index(), None);
    }
}
