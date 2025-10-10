# Ticket 3.1-Fix.1: `start_pipeline`이 실제 `Process` ID를 반환하도록 수정

## Goal

`StateManager::start_pipeline` 함수가 `PipelineEngine`을 실제로 구동하고, 생성된 `Process`를 내부 상태에 저장한 뒤, 해당 `Process`의 고유 `Uuid`를 반환하도록 수정합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/core/src/state/manager.rs`
- `pipeline-kit-rs/crates/core/src/engine/mod.rs`

## Reference Code

- **비동기 태스크 스폰**: `codex-rs/core/src/codex.rs`의 `Codex::spawn` 함수는 `tokio::spawn`을 사용하여 백그라운드 작업을 시작하고 핸들러를 관리하는 좋은 예시입니다. `StateManager`도 이와 유사하게 `PipelineEngine`을 스폰해야 합니다.

## Detailed Implementation Steps

1. **`Process` 구조체 확장**: `pk-protocol/src/process_models.rs`의 `Process` 구조체에 `tokio::task::JoinHandle`을 저장할 필드를 `#[serde(skip)]` 어트리뷰트와 함께 추가합니다 (상태 저장을 위해 직렬화에서 제외).

2. **`StateManager::start_pipeline` 수정**:
   - `Uuid::new_v4()`를 호출하여 새 프로세스 ID를 생성합니다.
   - `Process` 구조체 인스턴스를 생성하고 상태를 `Pending`으로 설정합니다.
   - `PipelineEngine::run`을 `tokio::spawn`으로 감싸 백그라운드 태스크를 시작하고, `JoinHandle`을 얻습니다.
   - 생성된 `Process`와 `JoinHandle`을 `self.processes` `HashMap`에 저장합니다.
   - 생성된 `Uuid`를 반환합니다.

## Acceptance Tests (TDD Process)

1. **RED**: `StateManager::start_pipeline`을 두 번 호출했을 때, 반환된 두 `Uuid`가 서로 다른지 확인하는 테스트를 작성합니다. 또한, 함수 호출 후 `StateManager`의 내부 프로세스 맵의 길이가 2가 되는지 확인합니다.

2. **GREEN**: `start_pipeline` 함수를 위 설명대로 구현하여 테스트를 통과시킵니다.

3. **REFACTOR**: `Process` 생성과 `PipelineEngine` 스폰 로직을 별도의 private 헬퍼 함수로 분리하여 `start_pipeline` 메서드를 간결하게 유지합니다.
