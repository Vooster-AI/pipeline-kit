# Ticket 9.1: 에이전트 어댑터 리팩토링 - `CliExecutor` 도입

## Goal
모든 CLI 기반 에이전트 어댑터에 중복된 서브프로세스 실행 및 통신 로직을 제거합니다. `CliExecutor`라는 공통 모듈을 도입하여, 새로운 에이전트를 추가할 때 오직 명령어 구성과 결과 변환 로직만 작성하도록 구조를 개선합니다.

## Core Modules & Roles

-   `pipeline-kit-rs/crates/core/src/agents/cli_executor.rs` (신규):
    -   서브프로세스 실행, stdio 파이핑, 출력 스트림(JSON Lines, NDJSON) 파싱 등 모든 공통 로직을 담당합니다.
-   `pipeline-kit-rs/crates/core/src/agents/adapters/*.rs` (수정):
    -   `claude_adapter.rs`, `cursor_adapter.rs`, `gemini_adapter.rs` 등 모든 CLI 기반 어댑터.
    -   기존의 서브프로세스 코드를 제거하고 `CliExecutor`를 사용하도록 수정합니다.
    -   역할이 **명령어 및 인자 구성**과 **결과(`serde_json::Value`)를 `AgentEvent`로 변환**하는 것으로 축소됩니다.

## Interfaces

```rust
// In pipeline-kit-rs/crates/core/src/agents/cli_executor.rs

use tokio_stream::Stream;
use std::pin::Pin;

pub struct CliExecutor;

impl CliExecutor {
    /// CLI 명령어를 실행하고, stdout을 JSON Lines/NDJSON 스트림으로 파싱하여 반환합니다.
    pub fn execute(
        command: String,
        args: Vec<String>,
        working_dir: String,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value, AgentError>> + Send>>;
}
```

## Guidelines & Conventions

-   `CliExecutor::execute`는 `tokio::process::Command`를 사용하여 비동기적으로 서브프로세스를 실행해야 합니다.
-   `stdout`은 `tokio::io::BufReader`의 `lines()`를 사용하여 비동기적으로 한 줄씩 읽어야 합니다.
-   읽어온 각 라인은 `serde_json::from_str`를 사용해 `serde_json::Value`로 파싱합니다. 파싱 실패 시 `AgentError::StreamParseError`를 반환해야 합니다.
-   각 어댑터는 `CliExecutor::execute`가 반환한 `Stream`을 `map` 또는 `and_then`으로 변환하여 `serde_json::Value`를 `AgentEvent`로 매핑합니다.

## Acceptance Tests (TDD Process)

### 1. RED:
-   `tests/cli_executor.rs` 파일을 생성합니다.
-   표준 출력으로 미리 정의된 JSON Lines를 출력하는 간단한 Mock CLI 실행 파일(Python 또는 Rust로 작성)을 만듭니다.
-   `CliExecutor::execute`를 호출하여 이 Mock CLI를 실행하고, 반환된 스트림이 예상된 `serde_json::Value` 객체들을 순서대로 방출하는지 검증하는 테스트를 작성합니다. `CliExecutor`가 아직 존재하지 않으므로 테스트는 컴파일에 실패합니다.

### 2. GREEN:
-   `cli_executor.rs` 모듈과 `CliExecutor::execute` 함수를 구현하여 RED 단계의 테스트를 통과시킵니다.
-   `claude_adapter.rs`를 `CliExecutor`를 사용하도록 리팩토링합니다. `ClaudeAdapter`의 단위 테스트를 (필요하다면) 수정하여, 이제 Mock `serde_json::Value` 스트림을 입력받아 `AgentEvent`를 올바르게 변환하는지만 검증하도록 변경합니다.

### 3. REFACTOR:
-   `CliExecutor`가 다양한 출력 형식(예: 일반 텍스트 스트림)도 처리할 수 있도록 확장성을 고려하여 구조를 개선합니다.
-   다른 모든 CLI 어댑터(`CursorAdapter`, `GeminiAdapter` 등)를 `CliExecutor`를 사용하도록 순차적으로 리팩토링하고, 각 어댑터의 테스트를 수정합니다.
-   중복 코드가 완전히 제거되었는지 확인합니다.

## Expected Outcomes

- 모든 CLI 기반 어댑터에서 서브프로세스 실행 코드 중복이 완전히 제거됩니다.
- 새로운 CLI 기반 에이전트를 추가할 때 명령어 구성과 결과 변환 로직만 작성하면 됩니다.
- 코드 유지보수성과 테스트 용이성이 크게 향상됩니다.

## Related Files

- `pipeline-kit-rs/crates/core/src/agents/cli_executor.rs` (신규)
- `pipeline-kit-rs/crates/core/src/agents/adapters/claude_adapter.rs`
- `pipeline-kit-rs/crates/core/src/agents/adapters/cursor_adapter.rs`
- `pipeline-kit-rs/crates/core/src/agents/adapters/gemini_adapter.rs`
- `pipeline-kit-rs/crates/core/tests/cli_executor.rs` (신규)
