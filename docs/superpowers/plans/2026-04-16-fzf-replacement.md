# fzf 교체 (Native Rust TUI Picker) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 외부 `fzf` 바이너리 의존성을 제거하고, `ratatui` + `nucleo-matcher` 기반 네이티브 Rust TUI 피커로 교체한다.

**Architecture:** outer process가 아이템을 temp file에 JSON 직렬화한 뒤 `tmux display-popup -EE "tss internal-picker <items_path> <result_path>"` 를 실행하고, inner process가 ratatui TUI를 실행 후 결과를 result file에 기록한다. 기존 `SelectItemReturn` 인터페이스를 유지하여 `main.rs` 호출부 변경 없이 교체한다.

**Tech Stack:** ratatui 0.29, crossterm 0.28, nucleo-matcher 0.3, tempfile 3, serde_json (already present)

---

## 파일 구조

**신규 생성:**
- `src/picker/mod.rs` — PickerConfig, PickerResult, run() 진입점
- `src/picker/filter.rs` — nucleo-matcher 퍼지 필터 래퍼
- `src/picker/state.rs` — PickerState (쿼리, 커서, 필터 결과)
- `src/picker/input.rs` — crossterm KeyEvent → Action 매핑
- `src/picker/ui.rs` — ratatui 렌더링 로직

**수정:**
- `Cargo.toml` — 의존성 추가
- `src/args.rs` — InternalPicker 서브커맨드 추가
- `src/fzf.rs` — select_item 교체, invoke_picker 추가
- `src/main.rs` — InternalPicker 분기 처리, remove_favorite_interactive 교체

---

## Task 1: 의존성 추가

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Cargo.toml에 의존성 추가**

`[dependencies]` 섹션에 아래 항목 추가:

```toml
ratatui = "0.29"
crossterm = "0.28"
nucleo-matcher = "0.3"
tempfile = "3"
```

`serde_json`은 이미 있으므로 추가 불필요.

- [ ] **Step 2: 빌드 확인**

```bash
cargo build
```

Expected: 새 의존성 다운로드 후 컴파일 성공

- [ ] **Step 3: 커밋**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add ratatui, crossterm, nucleo-matcher, tempfile deps"
```

---

## Task 2: src/picker/filter.rs — 퍼지 필터

**Files:**
- Create: `src/picker/filter.rs`

- [ ] **Step 1: 실패하는 테스트 작성**

`src/picker/filter.rs` 파일 생성:

```rust
use nucleo_matcher::{Config, Matcher, Utf32Str};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};

pub(crate) struct FuzzyFilter {
    matcher: Matcher,
}

impl FuzzyFilter {
    pub(crate) fn new() -> Self {
        todo!()
    }

    pub(crate) fn filter(&mut self, query: &str, items: &[String]) -> Vec<usize> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query_returns_all_in_order() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let result = f.filter("", &items);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_fuzzy_match_includes_matching_item() {
        let mut f = FuzzyFilter::new();
        let items = vec!["editor".to_string(), "terminal".to_string(), "server".to_string()];
        let result = f.filter("ed", &items);
        assert!(result.contains(&0), "editor should match 'ed'");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string()];
        let result = f.filter("zzz", &items);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_with_empty_items() {
        let mut f = FuzzyFilter::new();
        let result = f.filter("abc", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_higher_score_appears_first() {
        let mut f = FuzzyFilter::new();
        // "ed" should match "editor" with higher score than "embedded"
        let items = vec!["embedded".to_string(), "editor".to_string()];
        let result = f.filter("edi", &items);
        // "editor" (index 1) should score higher than "embedded" (index 0) for "edi"
        assert_eq!(result[0], 1, "editor should rank higher for 'edi'");
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test picker::filter
```

Expected: FAIL (`todo!()` panic)

- [ ] **Step 3: 구현**

`src/picker/filter.rs`의 `todo!()` 부분을 교체:

```rust
use nucleo_matcher::{Config, Matcher, Utf32Str};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};

pub(crate) struct FuzzyFilter {
    matcher: Matcher,
}

impl FuzzyFilter {
    pub(crate) fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// query가 비어있으면 전체 인덱스를 순서대로 반환.
    /// 아닌 경우 매칭되는 인덱스를 점수 내림차순으로 반환.
    pub(crate) fn filter(&mut self, query: &str, items: &[String]) -> Vec<usize> {
        if query.is_empty() {
            return (0..items.len()).collect();
        }
        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut buf = Vec::new();
        let mut scored: Vec<(usize, u32)> = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let score = pattern.score(Utf32Str::new(item, &mut buf), &mut self.matcher)?;
                Some((i, score))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(i, _)| i).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query_returns_all_in_order() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let result = f.filter("", &items);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_fuzzy_match_includes_matching_item() {
        let mut f = FuzzyFilter::new();
        let items = vec!["editor".to_string(), "terminal".to_string(), "server".to_string()];
        let result = f.filter("ed", &items);
        assert!(result.contains(&0), "editor should match 'ed'");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string()];
        let result = f.filter("zzz", &items);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_with_empty_items() {
        let mut f = FuzzyFilter::new();
        let result = f.filter("abc", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_higher_score_appears_first() {
        let mut f = FuzzyFilter::new();
        let items = vec!["embedded".to_string(), "editor".to_string()];
        let result = f.filter("edi", &items);
        assert_eq!(result[0], 1, "editor should rank higher for 'edi'");
    }
}
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cargo test picker::filter
```

Expected: 5 tests PASS

- [ ] **Step 5: 커밋**

```bash
git add src/picker/filter.rs
git commit -m "feat: add fuzzy filter using nucleo-matcher"
```

---

## Task 3: src/picker/state.rs — PickerState

**Files:**
- Create: `src/picker/state.rs`

- [ ] **Step 1: 실패하는 테스트 작성**

`src/picker/state.rs` 파일 생성:

```rust
pub(crate) struct PickerState {
    pub query: String,
    pub cursor: usize,
    pub selected: usize,
    pub filtered: Vec<usize>,
    pub items: Vec<String>,
}

impl PickerState {
    pub(crate) fn new(items: Vec<String>) -> Self { todo!() }
    pub(crate) fn insert_char(&mut self, _c: char) { todo!() }
    pub(crate) fn delete_char_backward(&mut self) { todo!() }
    pub(crate) fn delete_word_backward(&mut self) { todo!() }
    pub(crate) fn delete_to_start(&mut self) { todo!() }
    pub(crate) fn cursor_left(&mut self) { todo!() }
    pub(crate) fn cursor_right(&mut self) { todo!() }
    pub(crate) fn cursor_to_start(&mut self) { todo!() }
    pub(crate) fn cursor_to_end(&mut self) { todo!() }
    pub(crate) fn move_up(&mut self) { todo!() }
    pub(crate) fn move_down(&mut self) { todo!() }
    pub(crate) fn page_up(&mut self, _page_size: usize) { todo!() }
    pub(crate) fn page_down(&mut self, _page_size: usize) { todo!() }
    pub(crate) fn update_filter(&mut self, _filtered: Vec<usize>) { todo!() }
    pub(crate) fn selected_item_index(&self) -> Option<usize> { todo!() }
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
        s.move_down(); // 경계: 더 이상 내려갈 수 없음
        assert_eq!(s.selected, 2);
        s.move_up();
        assert_eq!(s.selected, 1);
        s.move_up();
        s.move_up(); // 경계
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn test_page_up_down() {
        let mut s = make(&["a"; 20]);
        s.selected = 10;
        s.page_up(5);
        assert_eq!(s.selected, 5);
        s.page_down(8);
        assert_eq!(s.selected, 13);
        s.page_down(100); // 경계
        assert_eq!(s.selected, 19);
        s.page_up(100); // 경계
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn test_update_filter_clamps_selected() {
        let mut s = make(&["a", "b", "c"]);
        s.selected = 2;
        s.update_filter(vec![0]); // 1개 결과
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
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test picker::state
```

Expected: FAIL (`todo!()` panic)

- [ ] **Step 3: 구현**

`src/picker/state.rs` 전체를 아래로 교체:

```rust
pub(crate) struct PickerState {
    pub query: String,
    pub cursor: usize,       // query 내 바이트 인덱스
    pub selected: usize,     // filtered 내 인덱스
    pub filtered: Vec<usize>,// items 내 인덱스 목록
    pub items: Vec<String>,
}

impl PickerState {
    pub(crate) fn new(items: Vec<String>) -> Self {
        let filtered = (0..items.len()).collect();
        Self {
            query: String::new(),
            cursor: 0,
            selected: 0,
            filtered,
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

    /// 커서 앞의 단어(공백 포함) 삭제. fzf의 Ctrl-W 동작과 동일.
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
        self.filtered = filtered;
        let max = self.filtered.len().saturating_sub(1);
        if self.selected > max {
            self.selected = max;
        }
        // filtered가 비어있을 때 selected는 0이지만 selected_item_index는 None 반환
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
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cargo test picker::state
```

Expected: 12 tests PASS

- [ ] **Step 5: 커밋**

```bash
git add src/picker/state.rs
git commit -m "feat: add PickerState for query and list navigation"
```

---

## Task 4: src/picker/input.rs — 키 입력 → Action 매핑

**Files:**
- Create: `src/picker/input.rs`

- [ ] **Step 1: 실패하는 테스트 작성**

`src/picker/input.rs` 파일 생성:

```rust
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

pub(crate) fn key_to_action(_key: KeyEvent) -> Action {
    todo!()
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
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test picker::input
```

Expected: FAIL (`todo!()` panic)

- [ ] **Step 3: 구현**

`key_to_action` 함수를 구현:

```rust
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
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cargo test picker::input
```

Expected: 26 tests PASS

- [ ] **Step 5: 커밋**

```bash
git add src/picker/input.rs
git commit -m "feat: add fzf-compatible key binding mapping"
```

---

## Task 5: src/picker/ui.rs — ratatui 렌더링

**Files:**
- Create: `src/picker/ui.rs`

> UI 렌더링은 실제 터미널이 없으면 테스트가 어렵다. 컴파일 통과를 확인한다.

- [ ] **Step 1: ui.rs 작성**

`src/picker/ui.rs` 파일 생성:

```rust
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use super::state::PickerState;

fn border_type_from_str(border: &str) -> BorderType {
    match border {
        "sharp" => BorderType::Plain,
        "bold" => BorderType::Thick,
        "double" => BorderType::Double,
        _ => BorderType::Rounded,
    }
}

/// layout = "default": 프롬프트 상단, 리스트 하단
/// layout = "reverse": 프롬프트 하단, 리스트 상단 (fzf --layout=reverse 동작)
pub(crate) fn render(
    frame: &mut Frame,
    state: &PickerState,
    title: &str,
    border: &str,
    layout: &str,
    list_state: &mut ListState,
) {
    let area = frame.area();

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type_from_str(border))
        .title(format!(" {} ", title));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

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
    }
}
```

- [ ] **Step 2: 컴파일 확인**

```bash
cargo check
```

Expected: 컴파일 성공 (ui.rs는 아직 mod.rs에서 선언되지 않았으므로 dead_code 경고는 무시)

---

## Task 6: src/picker/mod.rs — 이벤트 루프

**Files:**
- Create: `src/picker/mod.rs`

- [ ] **Step 1: mod.rs 작성**

`src/picker/mod.rs` 파일 생성:

```rust
mod filter;
mod input;
mod state;
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

#[derive(Serialize, Deserialize)]
pub(crate) struct PickerConfig {
    pub items: Vec<String>,
    pub title: String,
    pub border: String,
    pub layout: String,
}

pub(crate) enum PickerResult {
    Selected(usize),   // items 내 인덱스
    New(String),       // 매칭 없는 쿼리 → 새 창 이름
    Cancelled,
}

/// tmux display-popup 내에서 실행되는 TUI 피커.
/// PickerConfig를 받아 사용자 선택 결과를 반환한다.
pub(crate) fn run(config: PickerConfig) -> PickerResult {
    // 패닉 시 터미널 복원을 위한 훅
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
    // config.items를 이동시키기 전에 나머지 필드를 먼저 추출
    let PickerConfig { items, title, border, layout } = config;
    let mut state = PickerState::new(items);
    let mut filter = FuzzyFilter::new();
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal
            .draw(|f| ui::render(f, &state, &title, &border, &layout, &mut list_state))
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
    let filtered = filter.filter(&state.query, &state.items);
    state.update_filter(filtered);
    list_state.select(Some(state.selected));
}
```

- [ ] **Step 2: main.rs에 mod picker; 선언 추가**

`src/main.rs` 상단의 `mod` 선언 목록에 추가:

```rust
mod picker;
```

- [ ] **Step 3: 컴파일 확인**

```bash
cargo check
```

Expected: 컴파일 성공

- [ ] **Step 4: 커밋**

```bash
git add src/picker/
git commit -m "feat: add native ratatui TUI picker (inner process)"
```

---

## Task 7: src/args.rs — InternalPicker 서브커맨드

**Files:**
- Modify: `src/args.rs`

- [ ] **Step 1: 실패하는 테스트 작성**

`src/args.rs`의 `#[cfg(test)]` 블록 안에 테스트 추가:

```rust
#[test]
fn test_internal_picker_subcommand() {
    let args = Args::try_parse_from([
        "tss", "internal-picker", "/tmp/items.json", "/tmp/result.txt",
    ])
    .unwrap();
    match args.command {
        Some(Commands::InternalPicker { items_path, result_path }) => {
            assert_eq!(items_path, "/tmp/items.json");
            assert_eq!(result_path, "/tmp/result.txt");
        }
        _ => panic!("Expected InternalPicker"),
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cargo test args::tests::test_internal_picker_subcommand
```

Expected: FAIL (InternalPicker variant 없음)

- [ ] **Step 3: Commands enum에 InternalPicker 추가**

`src/args.rs`의 `Commands` enum에 variant 추가:

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage favorites
    Favorite(FavoriteArgs),
    /// Internal: run TUI picker inside tmux display-popup (not for direct use)
    #[command(hide = true)]
    InternalPicker {
        items_path: String,
        result_path: String,
    },
}
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cargo test args::tests
```

Expected: 전체 args 테스트 PASS

- [ ] **Step 5: 커밋**

```bash
git add src/args.rs
git commit -m "feat: add internal-picker subcommand to args"
```

---

## Task 8: src/fzf.rs — select_item 교체

**Files:**
- Modify: `src/fzf.rs`

> `invoke_picker`는 실제 tmux 세션이 필요하므로 단위 테스트 없음. 컴파일 + 기존 단위 테스트 통과를 확인한다.

- [ ] **Step 1: fzf.rs 전체 교체**

`src/fzf.rs`를 아래 내용으로 완전히 교체:

```rust
use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;
use std::fmt::Display;
use std::io::Write;
use std::process::Command;

use super::tmux::SortPriority;
use crate::picker::PickerConfig;

fn get_terminal_width() -> u16 {
    if let Some((terminal_size::Width(width), _)) = terminal_size::terminal_size() {
        std::cmp::min(width, 80)
    } else {
        80
    }
}

pub(crate) fn sort_by_priority<T: SortPriority + ?Sized>(items: &mut [Box<T>]) {
    items.sort_by(|a, b| {
        if a.sort_priority() > b.sort_priority() {
            return Greater;
        } else if a.sort_priority() < b.sort_priority() {
            return Less;
        }
        std::cmp::Ordering::Equal
    });
}

pub enum SelectItemReturn<'a, T> {
    None,
    Item(&'a T),
    NewWindowTitle(String),
}

pub(crate) enum PickerOutput {
    Cancelled,
    Selected(usize),
    New(String),
}

/// 아이템 문자열 목록을 받아 tmux display-popup으로 TUI 피커를 실행하고 결과를 반환한다.
/// tmux 세션 외부에서 호출하면 panic한다.
pub(crate) fn invoke_picker(
    item_strings: &[String],
    title: &str,
    border: &str,
    layout: &str,
) -> PickerOutput {
    let config = PickerConfig {
        items: item_strings.to_vec(),
        title: title.to_string(),
        border: border.to_string(),
        layout: layout.to_string(),
    };

    // 아이템을 temp file에 직렬화
    let mut items_file = tempfile::NamedTempFile::new().expect("Failed to create items temp file");
    serde_json::to_writer(&items_file, &config).expect("Failed to serialize picker config");
    items_file.flush().expect("Failed to flush items temp file");
    let items_path = items_file.path().to_string_lossy().to_string();

    // 결과를 받을 temp file 생성 (inner process가 여기에 씀)
    let result_file = tempfile::NamedTempFile::new().expect("Failed to create result temp file");
    let result_path = result_file.path().to_string_lossy().to_string();

    // 현재 실행 파일 경로로 display-popup 실행
    let exe = std::env::current_exe().expect("Failed to get current executable path");
    let height = std::cmp::min(item_strings.len() + 6, 40);
    let width = get_terminal_width();
    let popup_cmd = format!(
        "{} internal-picker {} {}",
        exe.to_string_lossy(),
        items_path,
        result_path,
    );

    Command::new("tmux")
        .args([
            "display-popup",
            "-EE",
            "-w",
            &width.to_string(),
            "-h",
            &height.to_string(),
            &popup_cmd,
        ])
        .status()
        .expect("Failed to run tmux display-popup");

    // result file 읽기 (items_file, result_file이 아직 살아있으므로 안전)
    let raw = std::fs::read_to_string(result_file.path()).unwrap_or_default();
    let raw = raw.trim();

    if raw.is_empty() {
        return PickerOutput::Cancelled;
    }
    if let Some(title) = raw.strip_prefix("new:") {
        return PickerOutput::New(title.to_string());
    }
    if let Ok(idx) = raw.parse::<usize>() {
        return PickerOutput::Selected(idx);
    }
    PickerOutput::Cancelled
}

pub(crate) fn select_item<'a, T: Display + ?Sized>(
    items: &'a [Box<T>],
    title: &str,
    border: &str,
    layout: &str,
) -> SelectItemReturn<'a, Box<T>> {
    let item_strings: Vec<String> = items.iter().map(|w| w.to_string()).collect();

    match invoke_picker(&item_strings, title, border, layout) {
        PickerOutput::Cancelled => SelectItemReturn::None,
        PickerOutput::Selected(idx) => {
            if let Some(item) = items.get(idx) {
                SelectItemReturn::Item(item)
            } else {
                SelectItemReturn::None
            }
        }
        PickerOutput::New(title) => SelectItemReturn::NewWindowTitle(title),
    }
}
```

- [ ] **Step 2: 컴파일 확인**

```bash
cargo check
```

Expected: 컴파일 성공

- [ ] **Step 3: 전체 테스트 통과 확인**

```bash
cargo test
```

Expected: 기존 단위 테스트 전부 PASS (fzf.rs에는 단위 테스트 없음)

- [ ] **Step 4: 커밋**

```bash
git add src/fzf.rs
git commit -m "feat: replace fzf shell call with native ratatui picker subprocess"
```

---

## Task 9: src/main.rs — InternalPicker 분기 + remove_favorite_interactive 교체

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: main()의 커맨드 처리 블록을 단일 match로 교체**

`src/main.rs`의 `fn main()` 안의 기존 `if let Some(Commands::Favorite(fa)) = args.command { ... }` 블록 전체를 아래로 교체한다.

기존:
```rust
    if let Some(Commands::Favorite(fa)) = args.command {
        match fa.command {
            FavoriteCommands::List => {
                handle_list(&config_path);
                return;
            }
            FavoriteCommands::Add { name, session_name, index, path } => {
                handle_add(&config_path, name, session_name, index, path);
                return;
            }
            FavoriteCommands::Remove { name } => {
                handle_remove(&config_path, name);
                return;
            }
        }
    }
```

교체 후:
```rust
    if let Some(cmd) = args.command {
        match cmd {
            Commands::Favorite(fa) => match fa.command {
                FavoriteCommands::List => handle_list(&config_path),
                FavoriteCommands::Add { name, session_name, index, path } => {
                    handle_add(&config_path, name, session_name, index, path);
                }
                FavoriteCommands::Remove { name } => handle_remove(&config_path, name),
            },
            Commands::InternalPicker { items_path, result_path } => {
                let json = std::fs::read_to_string(&items_path)
                    .expect("Failed to read items file");
                let picker_config: picker::PickerConfig = serde_json::from_str(&json)
                    .expect("Failed to parse picker config");

                let result = picker::run(picker_config);

                match result {
                    picker::PickerResult::Selected(idx) => {
                        std::fs::write(&result_path, idx.to_string())
                            .expect("Failed to write result");
                    }
                    picker::PickerResult::New(title) => {
                        std::fs::write(&result_path, format!("new:{}", title))
                            .expect("Failed to write result");
                    }
                    picker::PickerResult::Cancelled => {
                        // 결과 파일 미작성 → outer process가 Cancelled로 처리
                    }
                }
            }
        }
        return;
    }
```

- [ ] **Step 2: remove_favorite_interactive 교체**

`src/main.rs`의 `remove_favorite_interactive` 함수 전체를 교체:

```rust
fn remove_favorite_interactive(config_path: &str) {
    let config = Config::new(config_path);
    let favorites = match config.favorites {
        Some(ref f) if !f.is_empty() => f.clone(),
        _ => {
            println!("No favorites found.");
            return;
        }
    };

    let item_strings: Vec<String> = favorites.iter().map(|f| f.to_string()).collect();

    match fzf::invoke_picker(&item_strings, "Remove Favorite", "rounded", "default") {
        fzf::PickerOutput::Selected(idx) => {
            if let Some(fav) = favorites.get(idx) {
                remove_favorite_by_name(config_path, &fav.name);
            }
        }
        fzf::PickerOutput::Cancelled | fzf::PickerOutput::New(_) => {}
    }
}
```

- [ ] **Step 3: 컴파일 및 전체 테스트 통과 확인**

```bash
cargo test
```

Expected: 전체 단위 테스트 PASS

- [ ] **Step 4: 커밋**

```bash
git add src/main.rs
git commit -m "feat: wire InternalPicker dispatch and replace remove_favorite_interactive"
```

---

## Task 10: 최종 확인 및 정리

**Files:**
- No new files

- [ ] **Step 1: 전체 테스트 실행**

```bash
cargo test
```

Expected: 모든 테스트 PASS

- [ ] **Step 2: 빌드 릴리즈 확인**

```bash
cargo build --release
```

Expected: 경고 없이 빌드 성공

- [ ] **Step 3: 수동 테스트 (tmux 세션 내에서)**

tmux 세션 내에서 실행:
```bash
./target/release/tss
```

- 피커가 tmux popup으로 표시되는지 확인
- 문자 타이핑 시 퍼지 필터링이 동작하는지 확인
- Ctrl-J/K, ↑/↓, Tab 으로 항목 이동 확인
- Enter로 선택, Esc로 취소 확인
- 매칭 없는 쿼리 입력 후 Enter로 새 창 생성 확인

- [ ] **Step 4: 커밋**

테스트 통과 확인 후 최종 커밋:
```bash
git add -p  # 변경사항 있을 경우
git commit -m "chore: verify fzf replacement integration"
```
