# Ticket 4.2: 대시보드 위젯 구현

## Goal

`StateManager`가 관리하는 모든 `Process`의 ID, 이름, 상태, 현재 단계를 테이블 형태로 실시간 표시합니다.

## Reference Code

- `codex-rs/tui/src/chatwidget.rs`의 `render` 메서드에서 리스트 형태의 데이터를 렌더링하는 방식을 참고하세요.

## Guidelines & Conventions

- 위젯은 `app.rs`의 `App` 구조체에 저장된 상태를 읽어 렌더링합니다.
- 사용자 입력에 따라 `Op` 이벤트를 발생시킵니다.

## Hints

-   `ratatui`의 `Table` 위젯을 사용하세요. `codex-rs/tui/src/resume_picker.rs`에서 `ratatui`를 사용해 리스트를 렌더링하는 `render_list` 함수가 좋은 참고 자료가 될 것입니다.
-   대시보드 위젯은 `widgets/dashboard.rs` 파일에 구현하고, 다음과 같은 구조로 만드세요:
    ```rust
    use ratatui::widgets::{Table, Row, Cell, Block, Borders};

    pub fn render_dashboard(frame: &mut Frame, area: Rect, processes: &[Process], selected: usize) {
        let rows = processes.iter().map(|p| {
            Row::new(vec![
                Cell::from(p.id.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:?}", p.status)),
                Cell::from(p.current_step.to_string()),
            ])
        });

        let table = Table::new(rows)
            .header(Row::new(vec!["ID", "Name", "Status", "Step"]))
            .block(Block::default().borders(Borders::ALL).title("Processes"))
            .highlight_style(Style::default().bg(Color::Blue));

        frame.render_stateful_widget(table, area, &mut TableState::default());
    }
    ```
-   키보드 방향키(↑/↓)로 프로세스를 선택할 수 있도록 `App::selected_index`를 업데이트하는 로직을 추가하세요.

## Acceptance Tests (TDD Process)

1.  **RED**: 대시보드 위젯 렌더링 테스트를 작성합니다. `TestBackend`를 사용하여 테이블이 렌더링되는지 확인하지만, 아직 구현되지 않았으므로 실패합니다.
2.  **GREEN**: `Table` 위젯을 사용하여 `Process` 리스트를 렌더링하는 로직을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: 위젯 코드를 정리하고, 스타일링과 색상을 개선하여 가독성을 높입니다.
