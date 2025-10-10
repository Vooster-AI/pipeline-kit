# Ticket 4.2-Fix: 대시보드 위젯을 TUI에 통합

## Goal

이미 구현된 `dashboard.rs`의 `render_dashboard` 함수를 `app.rs`의 메인 렌더링 루프에 통합하여, 실행 중인 파이프라인 목록이 화면에 정상적으로 표시되도록 합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/tui/src/app.rs`: TUI의 메인 상태 및 렌더링 로직.
- `pipeline-kit-rs/crates/tui/src/ui.rs`: `App` 상태를 받아 실제 위젯을 렌더링하는 함수.
- `pipeline-kit-rs/crates/tui/src/widgets/dashboard.rs`: 대시보드 테이블 위젯 구현체.

## Reference Code

- **위젯 렌더링**: `codex-rs/tui/src/chatwidget.rs`의 `impl WidgetRef for &ChatWidget` 블록은 복합 위젯이 하위 위젯들(bottom_pane 등)에 렌더링을 위임하는 방식을 보여줍니다. 이 패턴을 `ui.rs`에서 `dashboard.rs`의 함수를 호출하는 데 적용합니다.

## Detailed Implementation Steps

1. **`pk-tui/src/app.rs` 수정**: `App` 구조체에 대시보드 상태를 저장할 필드를 추가합니다.
   ```rust
   // In pipeline-kit-rs/crates/tui/src/app.rs
   pub struct App {
       // ... 기존 필드
       pub processes: Vec<pk_protocol::process_models::Process>,
       pub selected_process_index: Option<usize>,
   }
   ```

2. **`pk-tui/src/ui.rs` 수정**: `render` 함수에서 `Paragraph` 위젯을 사용하는 부분을 `render_dashboard` 함수 호출로 교체합니다.
   ```rust
   // In pipeline-kit-rs/crates/tui/src/ui.rs
   use crate::widgets::dashboard::render_dashboard;

   pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<B>) {
       // ... (레이아웃 정의)
       // let dashboard_area = ...;

       // 기존 Paragraph 호출을 삭제하고 아래 코드로 대체합니다.
       render_dashboard(
           frame,
           dashboard_area,
           &app.processes,
           app.selected_process_index,
       );
   }
   ```

3. `handle_key_event`에서 `Up` 및 `Down` 키 입력에 따라 `app.selected_process_index`를 업데이트하는 로직을 추가합니다.

## Acceptance Tests (TDD Process)

1. **RED**: `tests/ui_rendering.rs`를 생성합니다. `TestBackend`를 사용하여 `App`에 mock `Process` 데이터를 채운 후 렌더링합니다. 렌더링된 버퍼에 `Paragraph`의 기본 텍스트("Dashboard Area")가 포함되어 있고, 테이블 헤더(예: "ID", "Pipeline")가 없는 것을 `assert!`하여 테스트를 실패시킵니다.

2. **GREEN**: `ui.rs`의 `render` 함수를 수정하여 `render_dashboard`를 호출하도록 변경합니다. `App`에 mock 데이터를 채운 테스트가 이제 테이블 헤더와 데이터를 포함한 스냅샷과 일치하는지 확인하여 통과시킵니다. `insta::assert_snapshot!`을 사용하세요.

3. **REFACTOR**: `render_dashboard` 함수가 `App`의 상태를 직접 수정하지 않고, 읽기 전용 참조(`&`)만 받도록 하여 렌더링 로직과 상태 변경 로직을 분리합니다.
