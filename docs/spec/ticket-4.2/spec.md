# Ticket 4.2: 대시보드 위젯 구현

## Goal

`StateManager`가 관리하는 모든 `Process`의 ID, 이름, 상태, 현재 단계를 테이블 형태로 실시간 표시합니다.

## Reference Code

- `codex-rs/tui/src/chatwidget.rs`의 `render` 메서드에서 리스트 형태의 데이터를 렌더링하는 방식을 참고하세요.

## Guidelines & Conventions

- 위젯은 `app.rs`의 `App` 구조체에 저장된 상태를 읽어 렌더링합니다.
- 사용자 입력에 따라 `Op` 이벤트를 발생시킵니다.
