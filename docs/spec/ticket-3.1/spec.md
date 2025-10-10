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

- **비동기 작업 루프**: `codex-rs/core/src/codex.rs`의 `submission_loop`과 `run_task` 함수에서 비동기 채널과 `tokio::spawn`을 사용한 이벤트 기반 아키텍처를 참고하세요.

## Guidelines & Conventions

- `PipelineEngine::run`은 `tokio::spawn`을 사용해 백그라운드 태스크에서 실행되어야 합니다.
- `Process`의 상태(`status`)가 변경될 때마다 `events_tx` 채널을 통해 `ProcessStatusUpdate` 이벤트를 전송해야 합니다.
- `HUMAN_REVIEW` 단계에 도달하면 `ProcessStatus`를 `HumanReview`로 변경하고 태스크 실행을 멈춥니다.

## Acceptance Tests (TDD Process)

1.  **RED**: `MockAgent`를 사용하는 간단한 순차 파이프라인을 정의하고, `PipelineEngine`이 이를 실행하여 예상된 순서대로 `ProcessStatusUpdate` 이벤트를 발생시키는지, 그리고 `HUMAN_REVIEW`에서 멈추는지 검증하는 테스트를 작성합니다.
2.  **GREEN**: `PipelineEngine`과 `Process` 상태 머신을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: `Arc<Mutex<>>`를 사용하여 `Process` 상태를 스레드-안전하게 만들고, `StateManager`를 도입하여 여러 프로세스를 관리할 수 있는 기반을 마련합니다.
