# Ticket 4.3: 프로세스 상세 뷰 위젯 구현

## Goal

특정 `Process`를 선택했을 때, 해당 프로세스의 전체 로그와 상세 정보를 스크롤 가능한 뷰로 표시합니다.

## Reference Code

- `codex-rs/tui/src/pager_overlay.rs`의 `PagerView` 구현을 참고하여 스크롤 가능한 텍스트 뷰를 만드세요.

## Guidelines & Conventions

- 위젯은 `app.rs`의 `App` 구조체에 저장된 상태를 읽어 렌더링합니다.
- 사용자 입력에 따라 `Op` 이벤트를 발생시킵니다.

## Hints

-   `codex-rs/tui/src/pager_overlay.rs`의 `PagerView` 구현을 참고하여 스크롤 가능한 텍스트 뷰를 만드세요. 이 파일은 스크롤 위치를 관리하고 키보드 입력으로 스크롤하는 완전한 예제를 제공합니다.
-   상세 뷰 위젯은 `widgets/detail_view.rs` 파일에 구현하고, `Paragraph` 위젯과 `Scrollbar`를 사용하세요:
    ```rust
    use ratatui::widgets::{Paragraph, Block, Borders, Scrollbar, ScrollbarOrientation};

    pub struct DetailView {
        pub scroll_offset: usize,
    }

    impl DetailView {
        pub fn render(&self, frame: &mut Frame, area: Rect, process: &Process) {
            let log_text = process.logs.join("\n");
            let paragraph = Paragraph::new(log_text)
                .block(Block::default().borders(Borders::ALL).title("Process Details"))
                .scroll((self.scroll_offset as u16, 0));

            frame.render_widget(paragraph, area);
        }

        pub fn scroll_up(&mut self) {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }

        pub fn scroll_down(&mut self, max: usize) {
            self.scroll_offset = (self.scroll_offset + 1).min(max);
        }
    }
    ```
-   `j`/`k` 또는 PageUp/PageDown 키로 스크롤할 수 있도록 키 이벤트 핸들러를 추가하세요.

## Acceptance Tests (TDD Process)

1.  **RED**: 상세 뷰 위젯 렌더링 테스트를 작성합니다. `TestBackend`를 사용하여 프로세스 로그가 표시되는지 확인하지만, 아직 구현되지 않았으므로 실패합니다.
2.  **GREEN**: `Paragraph` 위젯을 사용하여 로그를 렌더링하고, 스크롤 기능을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: 스크롤 성능을 개선하고, UI를 더 사용자 친화적으로 만듭니다 (예: 스크롤바 추가, 현재 위치 표시).
