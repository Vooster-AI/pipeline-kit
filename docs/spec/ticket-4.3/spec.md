# Ticket 4.3: 프로세스 상세 뷰 위젯 구현

## Goal

특정 `Process`를 선택했을 때, 해당 프로세스의 전체 로그와 상세 정보를 스크롤 가능한 뷰로 표시합니다.

## Reference Code

- `codex-rs/tui/src/pager_overlay.rs`의 `PagerView` 구현을 참고하여 스크롤 가능한 텍스트 뷰를 만드세요.

## Guidelines & Conventions

- 위젯은 `app.rs`의 `App` 구조체에 저장된 상태를 읽어 렌더링합니다.
- 사용자 입력에 따라 `Op` 이벤트를 발생시킵니다.
