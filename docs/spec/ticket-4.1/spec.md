# Ticket 4.1: TUI 애플리케이션 셸 및 이벤트 루프 구축 (`pk-tui` 크레이트)

## Goal

TUI의 기본 레이아웃, 메인 이벤트 루프, 그리고 `core`와의 통신 채널을 설정합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/tui/src/app.rs`: `App` 구조체(TUI 상태 관리) 및 메인 루프.
- `pipeline-kit-rs/crates/tui/src/main.rs`: TUI 애플리케이션 시작점.

## Reference Code

- `codex-rs/tui/src/app.rs`의 `App::run` 메서드와 이벤트 처리 루프(`select!`)를 핵심 로직으로 참고하세요.
- `codex-rs/tui/src/tui.rs`에서 터미널 초기화 및 복원 로직을 가져옵니다.

## Acceptance Tests (TDD Process)

1.  **RED**: `App::run`을 호출하는 테스트를 작성합니다. `TestBackend`를 사용하여 UI가 렌더링되는지 확인하지만, 아직 위젯이 없으므로 빈 화면이어야 합니다.
2.  **GREEN**: `ratatui`를 사용하여 기본 레이아웃(대시보드, 상세, 입력창 영역)을 그리고, `crossterm` 이벤트 스트림을 처리하는 기본 루프를 구현합니다. `q` 키를 누르면 종료되는지 테스트합니다.
3.  **REFACTOR**: TUI 상태와 비즈니스 로직을 분리하고, 이벤트 처리를 별도 모듈(`event_handler.rs`)로 분리하는 것을 고려합니다.
