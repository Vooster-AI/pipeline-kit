# Ticket 2.2: 에이전트 어댑터 패턴 및 관리자 구현 (`pk-core` 크레이트)

## Goal

다양한 AI 코딩 서비스를 일관된 인터페이스로 다루기 위한 `Agent` 트레이트(Adapter Pattern)와 이를 관리하는 `AgentManager`를 `pk-core`에 구현합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/core/src/agents/base.rs`: `Agent` 트레이트 정의.
- `pipeline-kit-rs/crates/core/src/agents/adapters/mock.rs`: 테스트를 위한 `MockAgent` 구현.
- `pipeline-kit-rs/crates/core/src/agents/manager.rs`: `AgentManager` 구현.

## Interfaces

- `trait Agent`:
  ```rust
  pub trait Agent: Send + Sync {
      async fn execute(&self, context: &ExecutionContext) -> Result<AgentOutput, AgentError>;
  }
  ```
- `struct AgentManager`:
  ```rust
  pub struct AgentManager { ... }
  impl AgentManager {
      pub fn new(agents_config: &[protocol::agent::Agent]) -> Self;
      pub fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>>;
  }
  ```

## Reference Code

- **어댑터 패턴**: 제공된 `apps/api/app/services/cli/base.py`의 `BaseCLI`와 `adapters/` 구조에서 영감을 얻으세요. Rust에서는 이를 `trait`과 구현체로 표현합니다.

## Guidelines & Conventions

- 초기에는 `MockAgent`만 구현하여 `PipelineEngine` 개발을 용이하게 합니다. `MockAgent`는 미리 정해진 응답을 반환하여 예측 가능한 테스트를 가능하게 합니다.
- `AgentManager`는 `AppConfig`로부터 에이전트 설정을 받아 내부 `HashMap`에 `Arc<dyn Agent>` 형태로 저장합니다.

## Acceptance Tests (TDD Process)

1.  **RED**: `AgentManager`가 `MockAgent`를 반환하는지 확인하는 테스트를 작성합니다. `Agent` 트레이트가 없으므로 컴파일이 실패합니다.
2.  **GREEN**: `Agent` 트레이트, `MockAgent` 구현체, 그리고 `AgentManager`를 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: `Agent` 트레이트의 메서드 시그니처를 명확히 하고, `AgentManager`의 에러 처리(에이전트를 찾지 못한 경우)를 개선합니다.
