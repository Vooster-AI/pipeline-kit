# **Ticket 1.2: 핵심 프로토콜 및 데이터 모델 정의 (`pk-protocol` 크레이트)**

**Goal**: `core`와 `tui` 간의 통신 및 설정 파일 파싱에 사용될 핵심 데이터 구조를 `pk-protocol` 크레이트에 정의합니다. 이 크레이트는 의존성을 최소화하여 다른 부분과 독립적으로 컴파일될 수 있어야 합니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/protocol/src/lib.rs`: 모든 공개 타입을 export하는 메인 파일.
- `pipeline_models.rs`: `Pipeline`, `MasterAgentConfig`, `ProcessStep` 구조체 및 열거형 정의.
- `agent_models.rs`: `Agent` 구조체 정의.
- `process_models.rs`: `Process`, `ProcessStatus` 열거형 정의.
- `config_models.rs`: `config.toml`에 대응하는 `GlobalConfig` 구조체 정의.
- `ipc.rs`: TUI와 Core 간의 통신을 위한 `Op`, `Event` 열거형 정의.

**Interfaces & Data Schemas**:

#### 1. `config_models.rs` (for `.pipeline-kit/config.toml`)```rust

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Represents global settings from config.toml. #[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct GlobalConfig { #[serde(default)]
pub git: bool,
}

````

#### 2. `agent_models.rs` (for `.pipeline-kit/agents/*.md`)
```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Represents an AI agent's configuration and system prompt.
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub model: String,
    #[serde(default)]
    pub color: String,

    /// The main content of the .md file, not part of the front matter.
    #[serde(skip)]
    pub system_prompt: String,
}
````

#### 3. `pipeline_models.rs` (for `.pipeline-kit/pipelines/*.yaml`)

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::collections::HashMap;

/// Represents a single step in a pipeline's process.
/// It can be either an agent name (String) or a special command.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, TS)]
#[serde(untagged)]
pub enum ProcessStep {
    Agent(String),
    #[serde(rename = "HUMAN_REVIEW")]
    HumanReview,
}

/// Defines the configuration for the master agent orchestrating the pipeline.
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "kebab-case")]
pub struct MasterAgentConfig {
    pub model: String,
    pub system_prompt: String,
    pub process: Vec<ProcessStep>,
}

/// Defines a full pipeline, including its agents and process flow.
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "kebab-case")]
pub struct Pipeline {
    pub name: String,
    #[serde(default)]
    pub required_reference_file: HashMap<u32, String>,
    #[serde(default)]
    pub output_file: HashMap<u32, String>,
    pub master: MasterAgentConfig,
    pub sub_agents: Vec<String>,
}
```

#### 4. `process_models.rs` (for runtime state)

````rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Represents the current lifecycle status of a running pipeline process.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessStatus {
    Pending,
    Running,
    Paused,
    HumanReview,
    Completed,
    Failed,
}

/// Represents the runtime state of a single pipeline execution.
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct Process {
    pub id: Uuid,
    pub pipeline_name: String,
    pub status: ProcessStatus,
    pub current_step: usize,
    pub logs: Vec<String>,
}```

#### 5. `ipc.rs` (for Inter-Process/Thread Communication)
```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::path::PathBuf;
use uuid::Uuid;
use crate::process_models::{Process, ProcessStatus}; // Reference other models in this crate

/// Operations sent from the UI (TUI) to the Core logic.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Op {
    StartPipeline {
        name: String,
        reference_file: Option<PathBuf>,
    },
    PauseProcess { process_id: Uuid },
    ResumeProcess { process_id: Uuid },
    KillProcess { process_id: Uuid },
    GetDashboardState,
    GetProcessDetail { process_id: Uuid },
    Shutdown,
}

/// Events sent from the Core logic to the UI (TUI).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Event {
    ProcessStarted {
        process_id: Uuid,
        pipeline_name: String,
    },
    ProcessStatusUpdate {
        process_id: Uuid,
        status: ProcessStatus,
        step_index: usize,
    },
    ProcessLogChunk {
        process_id: Uuid,
        content: String,
    },
    ProcessCompleted { process_id: Uuid },
    ProcessError {
        process_id: Uuid,
        error: String,
    },
    // ... other events like DashboardStateUpdate, ProcessDetailUpdate
}
````

**Reference Code**:

- **Op/Event Pattern**: `codex-rs/protocol/src/protocol.rs`의 `Op`와 `Event` 열거형 구조.
- **Data Structures**: `codex-rs/protocol/src/config_types.rs`와 `codex-rs/protocol/src/models.rs`의 `serde` 및 `ts_rs` 어트리뷰트 사용법.

**Hints**:

- `pk-protocol/Cargo.toml`에 다음 의존성을 추가하세요:
  ```toml
  [dependencies]
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  ts-rs = "8.0" # Use a recent version
  uuid = { version = "1.8", features = ["serde", "v4"] }
  ```
- `ProcessStep` 열거형에 `#[serde(untagged)]` 어트리뷰트를 사용하면 YAML 파서가 문자열 `"HUMAN_REVIEW"`와 에이전트 이름(일반 `String`)을 동일한 `process` 리스트 내에서 구분하여 파싱할 수 있습니다. 이는 유연한 설정을 위해 필수적입니다.
- `ipc.rs`의 `Op`와 `Event` 열거형에 `#[serde(tag = "type", content = "payload", rename_all = "camelCase")]` 어트리뷰트를 사용하세요. 이 패턴은 JSON 객체를 명확하게 구분해주어 TypeScript 클라이언트와의 통신에서 매우 유용합니다. `codex-rs/protocol/src/mcp_protocol.rs`의 `ClientRequest`가 이 패턴을 사용합니다.
- `lib.rs` 파일에서 모든 public 타입을 re-export하여 다른 크레이트에서 `pk_protocol::pipeline_models::Pipeline`과 같이 쉽게 접근할 수 있도록 하세요.
  ```rust
  // in pk-protocol/src/lib.rs
  pub mod agent_models;
  pub mod config_models;
  pub mod ipc;
  pub mod pipeline_models;
  pub mod process_models;
  ```

**Acceptance Tests (TDD Process)**:

1.  **RED**: `pk-protocol` 크레이트 내에 `tests/` 디렉터리와 `serialization.rs` 파일을 생성합니다. 이 파일에서, 제공된 `.pipeline-kit/pipelines/ux-improve.yaml` 파일의 내용을 문자열로 가져와 `serde_yaml::from_str::<Pipeline>`을 호출하는 테스트 코드를 작성합니다. `Pipeline` 구조체가 존재하지 않으므로 컴파일 에러가 발생합니다.
2.  **GREEN**: 위의 `Interfaces & Data Schemas` 섹션에 정의된 대로 모든 구조체와 열거형을 각각의 모듈 파일(`pipeline_models.rs` 등)에 구현합니다. `serde` 어트리뷰트를 올바르게 사용하여 YAML이 `Pipeline` 구조체로 성공적으로 역직렬화되도록 하여 테스트를 통과시킵니다.
3.  **REFACTOR**: 각 구조체와 필드에 명확한 문서 주석(`/// ...`)을 추가하여 코드의 의도를 명확히 합니다. `lib.rs`에서 모듈 구조를 논리적으로 정리하고, 필요한 타입들만 `pub use`로 노출시켜 깔끔한 crate API를 제공합니다.
