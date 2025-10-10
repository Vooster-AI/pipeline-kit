# Ticket 3.1: 파이프라인 실행 엔진 및 상태 관리 구현 (`pk-core` 크레이트)

## Goal

`PipelineEngine`을 구현하여 `Pipeline` 정의에 따라 에이전트를 순차적으로 실행하고, `Process`의 상태를 관리합니다. 이 엔진은 비동기로 동작해야 합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/core/src/engine/mod.rs`: `PipelineEngine` 구현.
- `pipeline-kit-rs/crates/core/src/state/process.rs`: `Process` 상태 머신 로직 구현.
- `pipeline-kit-rs/crates/core/src/state/manager.rs`: 모든 활성 `Process`를 관리하는 `StateManager` 구현.

## Interfaces

- `PipelineEngine`:
  ```rust
  pub async fn run(&self, pipeline: &Pipeline, events_tx: Sender<Event>) -> Result<Process>;
  ```
- `StateManager`:
  ```rust
  pub fn new() -> Self;
  pub async fn start_pipeline(...) -> Uuid;
  pub async fn pause_process(&self, process_id: Uuid);
  // ... resume, kill 등
  ```

## Reference Code

- **Async Task Loop**: `codex-rs/core/src/codex.rs`의 `run_task` 함수는 모델과 상호작용하며 여러 "turn"을 실행하는 복잡한 비동기 루프의 좋은 예시입니다. `pipeline-kit`에서는 이 루프가 `process`의 `steps`를 순회하게 됩니다.
- **State Management**: `codex-rs/core/src/codex.rs`의 `struct Session`과 `struct State`가 `Mutex`를 사용하여 상태를 관리하는 방식을 참고하여 `StateManager`와 `Process`를 구현하세요.

## Guidelines & Conventions

- `PipelineEngine::run`은 `tokio::spawn`을 사용해 백그라운드 태스크에서 실행되어야 합니다.
- `Process`의 상태(`status`)가 변경될 때마다 `events_tx` 채널을 통해 `ProcessStatusUpdate` 이벤트를 전송해야 합니다.
- `HUMAN_REVIEW` 단계에 도달하면 `ProcessStatus`를 `HumanReview`로 변경하고 태스크 실행을 멈춥니다.

## Hints

-   `StateManager` 내부에 `HashMap<Uuid, Arc<Mutex<Process>>>`를 사용하여 각 프로세스의 상태를 스레드 안전하게 관리하세요.
-   `PipelineEngine::run`은 `tokio::spawn`을 사용해 백그라운드 태스크에서 실행되어야 합니다. 이 태스크는 `events_tx` 채널의 소유권을 가집니다.
-   `Process` 상태를 변경할 때마다 해당 `Process`에 연결된 `events_tx`를 통해 `ProcessStatusUpdate` 이벤트를 전송해야 합니다. `HUMAN_REVIEW`에 도달하면 상태를 변경하고 태스크 루프를 `tokio::sync::Notify` 등으로 대기 상태로 만드세요.

## Acceptance Tests (TDD Process)

1.  **RED**: `tests/pipeline_engine.rs`를 생성합니다. `MockAgent`를 사용하는 2단계 순차 파이프라인을 정의합니다. `PipelineEngine`을 실행하고 `tokio::mpsc` 채널을 통해 `ProcessStarted`, `ProcessStatusUpdate(Running)`, `ProcessStatusUpdate(HumanReview)` 이벤트가 순서대로 수신되는지 검증하는 테스트를 작성합니다.
2.  **GREEN**: `PipelineEngine`이 `process` 단계를 순회하며 `AgentManager`로부터 에이전트를 가져와 실행하고, 각 단계마다 적절한 `Event`를 보내는 로직을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: `StateManager`를 도입하여 `PipelineEngine`의 실행을 중앙에서 관리하도록 구조를 개선합니다. `start_pipeline`이 `Process`를 생성하고 `PipelineEngine` 태스크를 시작하도록 리팩터링합니다.
