# Favorite Commands Design

**Date:** 2026-04-15  
**Status:** Approved

## Overview

`tss favorite` 서브커맨드를 추가하여 config.toml의 즐겨찾기 항목을 CLI에서 직접 추가/제거/조회할 수 있게 한다. 기존 fzf 스위처 동작은 변경 없이 유지된다.

## Commands

### `tss favorite add`

현재 활성 tmux 윈도우를 즐겨찾기에 추가한다.

- 인자 없이 실행하면 `tmux display-message -p "#{session_name}|#{window_index}|#{window_name}|#{pane_current_path}"` 로 현재 윈도우 정보를 자동 감지
- 아래 인자를 제공하면 해당 값 사용 (인자 없는 필드는 자동 감지 값 사용)
  - `--name <name>`: 윈도우 이름
  - `--session-name <session>`: 세션 이름
  - `--index <index>`: 윈도우 인덱스
  - `--path <path>`: 작업 디렉토리 경로
- config.toml을 읽어 `favorites` 배열에 항목 추가 후 저장
- 동일한 `name`이 이미 존재하면 에러 메시지 출력 후 종료

### `tss favorite remove`

즐겨찾기 항목을 제거한다.

- `--name <name>` 없이 실행하면 fzf로 현재 즐겨찾기 목록을 표시하고 선택한 항목 제거
- `--name <name>` 제공 시 해당 이름의 항목을 직접 제거
- 즐겨찾기가 없거나 해당 이름을 찾지 못하면 에러 메시지 출력 후 종료

### `tss favorite list`

즐겨찾기 목록을 출력한다.

- config.toml의 `favorites` 배열을 순서대로 출력
- 즐겨찾기가 없으면 "No favorites found." 메시지 출력

## Args Structure

```
Args
├── --config, --title, --border, --layout  (기존 플래그)
└── [subcommand] Commands::Favorite(FavoriteArgs)
    └── FavoriteCommands
        ├── Add { name, session_name, index, path }  (모두 Optional)
        ├── Remove { name }                           (Optional)
        └── List
```

서브커맨드 없이 실행 시 기존 fzf 스위처 동작.

## Data Flow

```
tss favorite add
  → tmux display-message (인자 없을 때)
  → Config::new(path) 로 config.toml 읽기
  → favorites에 Favorite 항목 추가
  → Config::save(path) 로 config.toml 저장

tss favorite remove (--name 없음)
  → Config::new(path) 로 config.toml 읽기
  → fzf로 즐겨찾기 목록 표시
  → 선택된 항목 제거
  → Config::save(path) 로 config.toml 저장

tss favorite remove --name foo
  → Config::new(path) 로 config.toml 읽기
  → name으로 항목 찾아 제거
  → Config::save(path) 로 config.toml 저장

tss favorite list
  → Config::new(path) 로 config.toml 읽기
  → 목록 출력
```

## Config Changes

`Config` 구조체에 `save(path: &str)` 메서드 추가:

```rust
pub fn save(&self, config_file: &str) {
    let contents = toml::to_string_pretty(self).expect("Failed to serialize config");
    fs::write(config_file, contents).expect("Failed to write config file");
}
```

`Config`와 `Favorite` 모두 `serde::Serialize` derive 추가 필요 (현재는 `Deserialize`만 있음).

## Files to Change

- `src/args.rs` — `Commands`, `FavoriteArgs`, `FavoriteCommands` 추가
- `src/config.rs` — `Serialize` derive 추가, `save()` 메서드 추가
- `src/tmux/favorite.rs` — `Serialize` derive 추가
- `src/main.rs` — 서브커맨드 분기 로직 추가
- `src/tmux/mod.rs` — `get_current_window()` 함수 추가 (현재 활성 윈도우 감지)

## Error Handling

- config.toml이 없을 경우: `add` 시 새로 생성, `remove`/`list` 시 "No favorites found." 출력
- 중복 이름으로 `add` 시: "Favorite 'foo' already exists." 출력 후 `exit(1)`
- `remove --name foo`에서 해당 이름 없을 시: "Favorite 'foo' not found." 출력 후 `exit(1)`
- `remove` (fzf)에서 선택 취소 시: 아무것도 하지 않고 종료
