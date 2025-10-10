# Ticket 4.1-Fix: TUI 애플리케이션 진입점(`main.rs`) 구현

## Goal

`pk-tui` 크레이트를 실행 가능한 바이너리로 만들기 위해 `main.rs` 진입점을 생성합니다. 이 파일은 TUI 애플리케이션의 초기화와 실행을 담당합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/cli/src/main.rs`: `pipeline` 명령어가 아무 인수 없이 호출될 때 `pk-tui`의 `run_app` 함수를 호출하도록 수정합니다.
- `pipeline-kit-rs/crates/tui/src/lib.rs`: `run_app` 함수를 구현하여 `App`을 생성하고 실행합니다.
- `pipeline-kit-rs/crates/tui/Cargo.toml`: 바이너리 타겟을 정의합니다.

## Reference Code

- **TUI 진입점**: `codex-rs/tui/src/main.rs` 파일은 `codex-tui` 바이너리의 `main` 함수를 정의합니다. 이 구조를 그대로 따릅니다.
- **CLI-TUI 연결**: `codex-rs/cli/src/main.rs`의 `cli_main` 함수에서 서브커맨드가 없을 때 `codex_tui::run_main`을 호출하는 부분을 참고하세요.

## Detailed Implementation Steps

1. **`pk-tui/Cargo.toml` 수정**: `pk-tui`가 라이브러리이자 바이너리가 되도록 `[[bin]]` 섹션을 추가합니다.
   ```toml
   # In pipeline-kit-rs/crates/tui/Cargo.toml

   [package]
   name = "pk-tui"
   # ...

   [[bin]]
   name = "pk-tui"
   path = "src/main.rs"

   [lib]
   path = "src/lib.rs"
   ```

2. **`pk-tui/src/main.rs` 파일 생성**: 애플리케이션을 부트스트래핑하는 `main` 함수를 작성합니다.
   ```rust
   // In pipeline-kit-rs/crates/tui/src/main.rs
   use pk_tui::run_app;
   use color_eyre::Result;

   #[tokio::main]
   async fn main() -> Result<()> {
       // TUI를 초기화하고 App을 실행합니다.
       run_app().await
   }
   ```

3. **`pk-cli/src/main.rs` 수정**: `pipeline-kit-rs`의 메인 CLI가 `pipeline` 명령어를 처리하도록 수정합니다.
   ```rust
   // In pipeline-kit-rs/crates/cli/src/main.rs
   // ... (clap 파서 정의)

   // cli_main 함수 내부에서
   match cli.subcommand {
       None => { // 'pipeline' 단독 실행 시
           pk_tui::run_app().await?;
       }
       Some(Subcommand::Start { ... }) => {
           // ...
       }
   }
   ```

## Acceptance Tests (TDD Process)

1. **RED**: `pipeline-kit-rs/`에서 `cargo run --bin pk-tui`를 실행합니다. `main.rs` 파일이 없으므로 빌드가 실패합니다.

2. **GREEN**: 위의 "Detailed Implementation Steps"를 수행합니다. `cargo run --bin pk-tui`를 실행했을 때, TUI가 (비록 비어있을지라도) 실행되고 즉시 종료되는 것을 확인합니다.

3. **REFACTOR**: `run_app` 함수에 터미널 초기화 및 복원 로직이 잘 캡슐화되었는지 확인합니다. `codex-rs/tui/src/lib.rs`의 `run_main` 구조를 참고하여 에러 처리 및 터미널 복원 로직을 강화합니다.
