# Ticket 4.4: 슬래시 커맨드 컴포저 위젯 구현

## Goal

`/start`, `/pause` 등 슬래시로 시작하는 명령어를 입력하고, 자동 완성 제안을 받을 수 있는 입력창을 구현합니다.

## Reference Code

- `codex-rs/tui/src/bottom_pane/chat_composer.rs`와 `command_popup.rs`는 이 기능의 완벽한 레퍼런스입니다.

## Guidelines & Conventions

- 위젯은 `app.rs`의 `App` 구조체에 저장된 상태를 읽어 렌더링합니다.
- 사용자 입력에 따라 `Op` 이벤트를 발생시킵니다.

## Hints

-   `codex-rs/tui/src/bottom_pane/chat_composer.rs`와 `command_popup.rs`는 슬래시 커맨드 자동 완성 팝업을 구현하는 데 완벽한 레퍼런스입니다. 이 로직을 그대로 가져와 명령어만 `pipeline-kit`에 맞게 수정하세요.
-   커맨드 컴포저는 `widgets/command_composer.rs` 파일에 구현하고, 다음과 같은 기능을 제공해야 합니다:
    1. 텍스트 입력창 (사용자가 명령어를 입력)
    2. 자동 완성 팝업 (사용자가 `/`를 입력하면 사용 가능한 명령어 목록 표시)
    3. 명령어 실행 (Enter를 누르면 `Op` 이벤트 발생)
-   사용 가능한 명령어 목록:
    ```rust
    const COMMANDS: &[(&str, &str)] = &[
        ("/start <pipeline>", "Start a new pipeline"),
        ("/pause <process_id>", "Pause a running process"),
        ("/resume <process_id>", "Resume a paused process"),
        ("/kill <process_id>", "Kill a process"),
        ("/list", "List all processes"),
    ];
    ```
-   자동 완성은 fuzzy matching을 사용하여 입력된 텍스트와 가장 유사한 명령어를 제안하세요. `codex-rs`의 구현을 참고하면 `nucleo` 크레이트나 간단한 문자열 매칭을 사용할 수 있습니다.
-   Tab 키로 제안된 명령어를 자동 완성하고, Enter 키로 명령어를 실행하세요.

## Acceptance Tests (TDD Process)

1.  **RED**: 커맨드 입력창 테스트를 작성합니다. `/`를 입력했을 때 자동 완성 팝업이 표시되는지 확인하지만, 아직 구현되지 않았으므로 실패합니다.
2.  **GREEN**: 슬래시 커맨드 입력 및 자동 완성 로직을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: 커맨드 파싱 및 제안 로직을 개선하고, 사용자 경험을 향상시킵니다 (예: 명령어 설명 표시, 하이라이팅).
