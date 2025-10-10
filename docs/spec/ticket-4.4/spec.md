# Ticket 4.4: 슬래시 커맨드 컴포저 위젯 구현

## Goal

`/start`, `/pause` 등 슬래시로 시작하는 명령어를 입력하고, 자동 완성 제안을 받을 수 있는 입력창을 구현합니다.

## Reference Code

- `codex-rs/tui/src/bottom_pane/chat_composer.rs`와 `command_popup.rs`는 이 기능의 완벽한 레퍼런스입니다.

## Guidelines & Conventions

- 위젯은 `app.rs`의 `App` 구조체에 저장된 상태를 읽어 렌더링합니다.
- 사용자 입력에 따라 `Op` 이벤트를 발생시킵니다.
