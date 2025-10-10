# **Ticket 2.1: 설정 파일 로더 구현 (`pk-core` 크레이트)**

**Goal**: `.pipeline-kit/` 디렉터리 내의 `config.toml`, `agents/*.md`, `pipelines/*.yaml` 파일을 읽고 파싱하여, Phase 1에서 정의한 Rust 데이터 구조체로 변환하는 `ConfigLoader` 모듈을 `pk-core`에 구현합니다. 이 모듈은 애플리케이션의 모든 동작을 결정하는 설정을 로드하는 핵심적인 역할을 합니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/core/src/config/loader.rs`: 설정 파일 로딩 및 파싱 로직의 구현 파일입니다.
- `pipeline-kit-rs/crates/core/src/config/models.rs`: 파싱된 모든 설정을 담을 `AppConfig` 구조체를 정의합니다. 이 구조체는 `pk-protocol`의 모델들을 사용합니다.
- `pipeline-kit-rs/crates/core/src/config/error.rs`: 설정 로딩 과정에서 발생할 수 있는 오류들을 `thiserror`를 사용하여 구체적으로 정의합니다.

**Interfaces & Data Schemas**:

#### 1. `config.toml` 스키마

- **파일 위치**: `.pipeline-kit/config.toml`
- **역할**: 전역 설정을 정의합니다.
- **Rust 구조체 (`pk-protocol/src/config_models.rs`)**:

  ```rust
  // in pk-protocol/src/config_models.rs (or a new file)
  use serde::{Deserialize, Serialize};
  use ts_rs::TS;

  #[derive(Serialize, Deserialize, Debug, Clone, TS)]
  pub struct GlobalConfig {
      #[serde(default)]
      pub git: bool,
      // 향후 추가될 전역 설정 (예: 기본 모델, 로깅 레벨 등)
  }
  ```

#### 2. Agent Definition (`.md`) 스키마

- **파일 위치**: `.pipeline-kit/agents/*.md`
- **역할**: 각 AI 에이전트의 속성(이름, 모델 등)과 시스템 프롬프트를 정의합니다.
- **형식**: YAML Front Matter + Markdown 본문
- **Rust 구조체 (`pk-protocol/src/agent_models.rs`)**:

  ```rust
  // in pk-protocol/src/agent_models.rs
  use serde::{Deserialize, Serialize};
  use ts_rs::TS;

  #[derive(Serialize, Deserialize, Debug, Clone, TS)]
  pub struct Agent {
      pub name: String,
      pub description: String,
      pub model: String,
      #[serde(default)]
      pub color: String, // TUI에서 사용할 색상 등 UI 힌트

      #[serde(skip)] // Front Matter가 아닌 본문에서 채워짐
      pub system_prompt: String,
  }
  ```

#### 3. Pipeline Definition (`.yaml`) 스키마

- **파일 위치**: `.pipeline-kit/pipelines/*.yaml`
- **역할**: 여러 에이전트를 조합하여 하나의 작업 흐름(파이프라인)을 정의합니다.
- **Rust 구조체 (`pk-protocol/src/pipeline_models.rs`)**:

  ```rust
  // in pk-protocol/src/pipeline_models.rs
  use serde::{Deserialize, Serialize};
  use ts_rs::TS;
  use std::collections::HashMap;

  #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, TS)]
  #[serde(untagged)] // 문자열 또는 특수 객체를 허용
  pub enum ProcessStep {
      Agent(String), // 에이전트 이름을 나타내는 문자열
      #[serde(rename = "HUMAN_REVIEW")]
      HumanReview,
  }

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

  #[derive(Serialize, Deserialize, Debug, Clone, TS)]
  #[serde(rename_all = "kebab-case")]
  pub struct MasterAgentConfig {
      pub model: String,
      pub system_prompt: String,
      pub process: Vec<ProcessStep>,
  }
  ```

#### 4. 최종 `AppConfig` 모델

- **`config/models.rs`**: 위 모든 설정을 하나로 통합한 구조체입니다.

  ```rust
  use pk_protocol::{agent_models::Agent, pipeline_models::Pipeline, config_models::GlobalConfig};

  pub struct AppConfig {
      pub global: GlobalConfig,
      pub agents: Vec<Agent>,
      pub pipelines: Vec<Pipeline>,
  }
  ```

**Reference Code**:

- `codex-rs/core/src/config.rs`: `Config::load_with_cli_overrides` 함수와 TOML 파싱 로직을 참고하세요.
- `codex-rs/core/src/project_doc.rs`: `read_project_docs` 함수에서 디렉터리를 순회하며 특정 이름의 파일을 찾는 로직을 참고할 수 있습니다.

**Guidelines & Conventions**:

- `pk-core/Cargo.toml`에 `serde`, `serde_yaml`, `toml`, `gray_matter`, `thiserror`, `walkdir` 크레이트를 의존성으로 추가하세요.
- `loader.rs`에서 `walkdir`를 사용하여 `agents/`와 `pipelines/` 디렉터리를 탐색하고, 확장자에 따라 적절한 파서를 호출하는 로직을 구현하세요.
- `Agent` 구조체 파싱 시 `gray_matter`를 사용하여 Front Matter는 `Agent`의 필드로, 본문(content)은 `system_prompt` 필드로 매핑하세요.

  ```rust
  // Example using gray_matter in loader.rs
  use gray_matter::{Matter, engine::YAML};
  use pk_protocol::agent_models::Agent;

  // ...
  let matter = Matter::<YAML>::new();
  let result = matter.parse(markdown_content);

  let mut agent: Agent = result.data.unwrap().deserialize()?;
  agent.system_prompt = result.content;
  // ...
  ```

**Acceptance Tests (TDD Process)**:

1.  **RED**: `tests/config_loader.rs` 파일을 생성합니다. `tempfile::tempdir`를 사용하여 임시 디렉터리에 Ticket 설명에 있는 예제와 동일한 `.pipeline-kit` 구조와 설정 파일들을 생성하는 테스트를 작성합니다. `load_config` 함수를 호출하고, 아직 구현되지 않았으므로 실패해야 합니다.
2.  **GREEN**: `serde_yaml`로 `.yaml` 파일을, `toml`로 `.toml` 파일을, `gray_matter`로 `.md` 파일을 파싱하여 `AppConfig` 구조체를 올바르게 채우는 로직을 구현합니다. 모든 필드가 예상대로 채워졌는지 `assert_eq!`로 검증하여 테스트를 통과시킵니다.
3.  **REFACTOR**: 오류 처리를 개선합니다. 파일이 없거나, YAML/TOML/Markdown 형식이 잘못된 경우 `ConfigError` 열거형을 통해 명확한 오류를 반환하도록 리팩터링하고, 이에 대한 테스트 케이스를 추가합니다. 예를 들어, `pipelines/` 디렉터리가 없는 경우 `Ok(AppConfig)`를 반환하되 `pipelines` 필드는 비어 있어야 합니다.
