# Bell Notification Display Design

**Date:** 2026-04-18  
**Status:** Implemented

## Overview

tmux picker 리스트에서 `window_bell_flag`가 설정된 windows를 시각적으로 구분해 표시한다.
🔔 아이콘과 `bell_fg` 색상을 조합해 알림이 있는 창을 한눈에 파악할 수 있게 한다.

---

## 1. 데이터 레이어

### `src/tmux/window.rs`

- `Window` 구조체에 `bell: bool` 필드 추가
- `Display` impl: `bell == true`면 기존 `🟢` / `♥️` 패턴과 동일하게 ` 🔔` 문자열 추가
- `SortPriority` impl: bell window는 priority **1.0** (active=0.0, marked=2.0, others=3.0)
  — 알림 있는 창이 리스트 상단에 표시됨

### `src/tmux/mod.rs`

- `get_running_windows`의 tmux format 문자열에 `#{window_bell_flag}|` 추가
- 파싱 regex에 캡처 그룹 하나 추가 (`captures[6]`)
- `Window` 생성 시 `bell: &captures[6] == "1"` 설정

---

## 2. 테마 레이어

### `src/picker/theme.rs`

- `Theme` 구조체에 `bell_fg: Color` 필드 추가
- 내장 테마별 기본값:
  - **default**: `Color::Yellow`
  - **tokyo-night**: `Color::Rgb(255, 158, 100)` (주황)
  - **solarized-dark**: `Color::Yellow`

### `src/config.rs`

- `Config` 구조체에 `bell_fg: Option<String>` 추가
- `config.toml`에서 hex 색상 오버라이드 가능 (기존 `highlight_fg` 등과 동일한 파싱 방식)

---

## 3. UI 레이어

### `src/picker/ui.rs`

- `render` 함수의 list item 생성 루프에서, 텍스트에 `🔔`가 포함된 경우:
  - **비선택 row**: `normal_style`의 fg를 `theme.bell_fg`로 교체
  - **선택 row**: `highlight_bg` 유지, fg를 `theme.bell_fg`로 교체
- match highlight (`match_fg`, bold)는 bell 상태와 독립적으로 유지

---

## 변경 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/tmux/window.rs` | `bell` 필드 추가, Display/SortPriority 업데이트 |
| `src/tmux/mod.rs` | tmux format + regex 파싱 확장 |
| `src/picker/theme.rs` | `bell_fg` 필드 추가, 테마별 기본값 설정 |
| `src/config.rs` | `bell_fg` config 옵션 추가 |
| `src/picker/ui.rs` | bell row 색상 렌더링 로직 추가 |

---

## 비고

- `window_activity_flag`, `window_silence_flag`는 이번 범위에서 제외
- bell 상태는 tmux가 직접 관리 (리셋은 tmux 내부 동작에 따름)
