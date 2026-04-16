# fzf 교체 (Native Rust TUI Picker) 설계 문서

**날짜:** 2026-04-16  
**작성자:** Brainstorming 세션

---

## 목표

외부 `fzf` 바이너리 의존성을 제거하고, `ratatui` + `nucleo-matcher` 기반의 네이티브 Rust TUI 피커로 교체한다. 기존 동작(퍼지 필터링, tmux popup 오버레이, 새 창 생성)을 완전히 유지한다.

---

## Section 1: 아키텍처

**Two-stage picker 구조:**

- **outer process** (`tss`): 아이템 목록을 temp file에 JSON으로 직렬화한 뒤 `tmux display-popup -EE "tss --internal-picker <items_path> <result_path>"` 를 실행하고 결과 파일을 읽는다.
- **inner process** (`tss --internal-picker`): ratatui TUI를 실행하고, 선택 결과를 result temp file에 기록한 후 종료한다.

popup은 `display-popup -EE` 옵션으로 blocking 실행되며, outer process는 popup이 닫힌 후 result file을 읽어 `SelectItemReturn`으로 변환한다.

**기존 인터페이스 유지:**
- `SelectItemReturn::None` — 취소
- `SelectItemReturn::Item(item)` — 아이템 선택
- `SelectItemReturn::NewWindowTitle(title)` — 매칭 없는 쿼리로 새 창 생성

---

## Section 2: TUI 레이아웃 & 키 바인딩

**레이아웃:**
```
┌─────────────────────────────────┐
│ > _                             │  ← 검색 프롬프트
├─────────────────────────────────┤
│ > main:1 - editor ⭐ ~/work     │  ← 선택된 항목 (하이라이트)
│   main:2 - terminal             │
│   work:1 - server               │
│   ...                           │
├─────────────────────────────────┤
│  3/10                           │  ← 매칭 수 / 전체 수
└─────────────────────────────────┘
```

**키 바인딩 (fzf 호환):**

| 키 | 동작 |
|----|------|
| 문자 타이핑 | 퍼지 필터 업데이트 |
| `↑` / `Ctrl-K` / `Ctrl-P` | 위로 이동 |
| `↓` / `Ctrl-J` / `Ctrl-N` | 아래로 이동 |
| `Tab` | 아래로 이동 |
| `Shift-Tab` | 위로 이동 |
| `Enter` | 현재 항목 선택 |
| `Esc` / `Ctrl-C` / `Ctrl-G` | 취소 (result file 미생성) |
| `Ctrl-U` | 검색어 전체 삭제 |
| `Ctrl-W` | 단어 단위 삭제 (역방향) |
| `Backspace` / `Ctrl-H` | 문자 하나 삭제 |
| `Ctrl-A` | 커서를 맨 앞으로 |
| `Ctrl-E` | 커서를 맨 뒤로 |
| `Ctrl-B` / `←` | 커서 한 칸 왼쪽 |
| `Ctrl-F` / `→` | 커서 한 칸 오른쪽 |
| `PgUp` | 페이지 위로 |
| `PgDn` | 페이지 아래로 |

> `Ctrl-B`/`Ctrl-F`는 검색 프롬프트에서 커서 이동으로 동작한다.

---

## Section 3: 데이터 플로우

```
[outer process: tss]
  1. 아이템 목록 생성 (favorites + windows + previous)
  2. 아이템을 temp file에 JSON으로 직렬화
  3. result temp file 경로 생성 (빈 파일)
  4. tmux display-popup -EE "tss --internal-picker <items_path> <result_path>" 실행
     (blocking — popup 닫힐 때까지 대기)
  5. result_path 파일 읽기:
     - 파일 없음 / 빈 파일 → SelectItemReturn::None
     - "new:<title>"      → SelectItemReturn::NewWindowTitle(title)
     - "<index>"          → SelectItemReturn::Item(ws[index])
  6. temp file 자동 정리 (tempfile crate의 NamedTempFile Drop)

[inner process: tss --internal-picker <items_path> <result_path>]
  1. items_path JSON 읽어서 아이템 목록 복원
  2. crossterm raw mode 진입, ratatui TUI 시작
  3. 이벤트 루프:
     - 키 입력 → 검색어 업데이트 → nucleo로 퍼지 필터링 → 목록 재렌더링
  4. Enter:
     - 필터 결과 있음 → result_path에 "<index>" 쓰고 종료
     - 필터 결과 없고 검색어 있음 → result_path에 "new:<query>" 쓰고 종료
  5. Esc / Ctrl-C / Ctrl-G → result_path에 아무것도 쓰지 않고 종료
```

**엣지 케이스:**
- popup 강제 종료 시 result_path 없음 → `None` 처리
- temp file은 outer process `Drop` 시 자동 삭제 (`tempfile` crate)
- inner process 패닉 시 raw mode 복원을 위해 `panic hook`에서 crossterm 복원 처리

---

## Section 4: 파일 구조 & 의존성

**변경될 파일:**
```
src/
├── main.rs          — --internal-picker 플래그 분기 처리 추가
├── args.rs          — InternalPicker 인수 추가 (items_path, result_path)
├── fzf.rs           — 완전 교체: 외부 fzf 호출 → tmux display-popup + temp file
├── picker/          — 신규 모듈
│   ├── mod.rs       — InternalPicker 진입점, 이벤트 루프
│   ├── ui.rs        — ratatui 렌더링 로직 (레이아웃, 하이라이트)
│   ├── input.rs     — crossterm 키 입력 → 액션 매핑
│   └── filter.rs    — nucleo-matcher 퍼지 필터 래퍼
└── tmux/
    └── ...          — 변경 없음
```

**추가될 의존성 (`Cargo.toml`):**
```toml
ratatui = "0.29"
crossterm = "0.28"
nucleo-matcher = "0.3"
tempfile = "3"
serde_json = "1"
```

**`remove_favorite_interactive` 변경:**
- 현재 `src/main.rs`에서 fzf 바이너리를 직접 호출하는 코드를 내부 picker로 교체한다.

---

## 비고

- `SelectItemReturn` 타입과 `main.rs` 호출부는 변경 없이 유지된다.
- 터미널 너비 감지 (`--tmux` 옵션 대응)는 `crossterm::terminal::size()`로 처리한다.
- 타이틀(`--title`), 테두리(`--border`), 레이아웃(`--layout`) 옵션은 내부 picker에도 전달되어 동일하게 렌더링된다.
