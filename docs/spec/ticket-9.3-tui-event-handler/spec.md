# Ticket 9.3: TUI 이벤트 핸들러 리팩토링 - 위젯에 책임 위임

## Goal
`app.rs`의 거대한 `handle_key_event` 메소드를 분리합니다. 각 대화형 위젯(`CommandComposer` 등)이 자신의 키 이벤트를 직접 처리하도록 책임을 위임하여 단일 책임 원칙을 준수하고, TUI 아키텍처의 확장성과 유지보수성을 향상시킵니다.

## Core Modules & Roles

-   `pipeline-kit-rs/crates/tui/src/app.rs` (수정):
    -   `handle_key_event`가 더 이상 위젯 내부의 상세 로직을 처리하지 않습니다.
    -   역할이 **이벤트 분배**로 축소됩니다. 현재 활성화된 위젯에 이벤트를 전달하고, 위젯이 처리하지 않은 경우에만 전역 이벤트를 처리합니다.
-   `pipeline-kit-rs/crates/tui/src/widgets/command_composer.rs` (수정):
    -   `handle_key_event(&mut self, key: KeyEvent) -> EventStatus`와 같은 새로운 메서드를 구현합니다.
    -   커서 이동, 텍스트 입력/삭제, 자동 완성 제안 선택 등 자신과 관련된 모든 키 입력을 직접 처리합니다.
-   기타 모든 대화형 위젯 (수정).
-   `pipeline-kit-rs/crates/tui/src/event.rs` (신규):
    -   `EventStatus` enum을 정의합니다.

## Interfaces

```rust
// In a new file: pipeline-kit-rs/crates/tui/src/event.rs

pub enum EventStatus {
    Consumed,      // 이벤트가 처리되었으며, 더 이상 전파할 필요 없음
    NotConsumed,   // 이벤트가 처리되지 않았으므로, 상위 컴포넌트가 처리해야 함
}

// In each interactive widget, e.g., command_composer.rs

impl CommandComposer {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> EventStatus;
}
```

## Guidelines & Conventions

-   `app.rs`의 이벤트 처리 순서는 **가장 구체적인(활성화된) 위젯에서 가장 일반적인(전역) 핸들러 순**으로 진행되어야 합니다.
-   예를 들어, 자동 완성 팝업이 활성화된 경우 `app.rs`는 키 이벤트를 먼저 팝업 위젯에 전달합니다. 팝업이 `EventStatus::Consumed`를 반환하면 이벤트 처리는 거기서 종료됩니다.
-   팝업이 `EventStatus::NotConsumed`를 반환하면(예: `Tab` 키가 아닌 일반 문자 입력), `app.rs`는 이어서 `CommandComposer` 위젯에 이벤트를 전달합니다.
-   이벤트가 모든 위젯에 의해 소비되지 않은 경우에만 `app.rs`는 전역 단축키(프로세스 선택, 종료 등)를 처리합니다.

## Acceptance Tests (TDD Process)

### 1. RED:
-   `widgets/command_composer.rs`의 테스트 모듈에 `handle_key_event`에 대한 테스트를 추가합니다.
-   예를 들어, "왼쪽 화살표 키(`KeyCode::Left`)" `KeyEvent`를 `handle_key_event`에 전달했을 때, `CommandComposer`의 내부 `cursor_pos` 상태가 1 감소하는지 검증하는 테스트를 작성합니다. `handle_key_event` 메서드가 아직 없으므로 컴파일에 실패합니다.

### 2. GREEN:
-   `EventStatus` 열거형을 정의합니다.
-   `CommandComposer`에 `handle_key_event` 메서드를 구현하여 텍스트 입력, 삭제, 커서 이동 로직을 `app.rs`에서 이전해옵니다. RED 단계의 테스트를 통과시킵니다.
-   `app.rs`의 `handle_key_event`를 수정하여 `CommandComposer`의 `handle_key_event`를 호출하고, `EventStatus::Consumed`일 경우 조기 반환하도록 로직을 변경합니다. 기존 `app.rs`의 관련 테스트들이 여전히 통과하는지 확인합니다.

### 3. REFACTOR:
-   `CommandComposer`의 이벤트 처리 로직을 더 작은 private 함수로 분리하여 가독성을 높입니다(예: `handle_char_input`, `handle_cursor_move`).
-   `app.rs`의 이벤트 분배 로직이 명확하고 확장 가능한지 검토합니다.
-   향후 다른 위젯(`DetailView`의 스크롤 등)에도 동일한 패턴을 적용할 수 있도록 구조를 일반화합니다.

## Expected Outcomes

- `app.rs`의 `handle_key_event` 메서드가 간결해지고 책임이 명확해집니다.
- 각 위젯이 자신의 상태와 이벤트를 독립적으로 관리하여 단일 책임 원칙을 준수합니다.
- 새로운 위젯이나 기능을 추가할 때 `app.rs`를 수정할 필요가 최소화됩니다.
- 위젯 단위의 테스트가 용이해져 테스트 커버리지가 향상됩니다.

## Related Files

- `pipeline-kit-rs/crates/tui/src/app.rs`
- `pipeline-kit-rs/crates/tui/src/event.rs` (신규)
- `pipeline-kit-rs/crates/tui/src/widgets/command_composer.rs`
- `pipeline-kit-rs/crates/tui/src/widgets/dashboard.rs`
- `pipeline-kit-rs/crates/tui/src/widgets/detail_view.rs`

## Architecture Pattern

이 리팩토링은 다음과 같은 아키텍처 패턴을 따릅니다:

1. **Chain of Responsibility**: 이벤트가 가장 구체적인 핸들러부터 일반적인 핸들러로 순차적으로 전파됩니다.
2. **Single Responsibility Principle**: 각 위젯이 자신의 이벤트만 처리하여 책임이 명확히 분리됩니다.
3. **Open/Closed Principle**: 새로운 위젯 추가 시 기존 코드 수정 없이 확장 가능한 구조입니다.
