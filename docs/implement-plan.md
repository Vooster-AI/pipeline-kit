## `pipeline-kit` 개발 계획서

시니어 소프트웨어 엔지니어로서, `codex-cli`의 아키텍처를 기반으로 `pipeline-kit` 애플리케이션을 개발하기 위한 구체적인 계획을 제안합니다. 이 계획은 코딩 에이전트가 직접 실행할 수 있도록 각 단계를 상세한 프롬프트 형식의 **Ticket**으로 정의하고, 병렬 작업이 가능한 **Phase**로 그룹화했습니다. 모든 Ticket은 TDD(Test-Driven Development)의 RED/GREEN/REFACTOR 사이클을 따르도록 설계되었습니다.

---

### **Phase 1: 프로젝트 기반 및 핵심 데이터 모델 정의**

> **목표**: 프로젝트의 기본 구조를 설정하고, 애플리케이션 전반에서 사용될 핵심 데이터 타입을 정의하여 안정적인 개발 기반을 마련합니다. 이 Phase의 Ticket들은 서로 의존성이 없으므로 병렬로 진행할 수 있습니다.

---

#### **Ticket 1.1: 모노레포 및 Cargo 워크스페이스 구조 설정**

**Goal**: `pipeline-kit` 프로젝트의 전체 디렉터리 구조와 Rust Cargo 워크스페이스를 설정합니다. 이는 모든 향후 개발의 기반이 됩니다.

**Core Modules & Roles**:

- `pipeline-kit/`: 모노레포 최상위 루트.
- `pipeline-kit/pipeline-kit-cli/`: TypeScript 래퍼 디렉터리.
- `pipeline-kit/pipeline-kit-rs/`: Rust 워크스페이스 루트.
- `pipeline-kit/pipeline-kit-rs/crates/`: 모든 Rust 크레이트가 위치할 디렉터리.
- `pipeline-kit/pipeline-kit-rs/Cargo.toml`: 워크스페이스를 정의하는 최상위 Cargo manifest.

**Reference Code**:

- `codex-rs/` 디렉터리 구조 및 최상위 `codex-rs/Cargo.toml`의 `[workspace]` 설정을 참고하세요.
- 최상위 `pnpm-workspace.yaml` 파일을 참고하여 모노레포를 구성하세요.

**Guidelines & Conventions**:

- Rust 크레이트 이름은 `pk-` 접두사를 사용합니다. (예: `pk-core`, `pk-tui`)
- 초기에는 빈 `lib.rs` 또는 `main.rs` 파일을 포함하여 각 크레이트 디렉터리(`cli`, `core`, `tui`, `protocol`, `protocol-ts`)를 생성합니다.

**Acceptance Tests (TDD Process)**:

1.  **RED**: `cargo check --workspace` 명령어가 실패하는 상태로 시작합니다 (파일이 없으므로).
2.  **GREEN**: 위의 디렉터리 구조와 `Cargo.toml` 워크스페이스 파일을 생성하여 `cargo check --workspace` 명령어가 성공적으로 완료되도록 합니다.
3.  **REFACTOR**: 생성된 파일 구조가 명확하고, `Cargo.toml`에 각 멤버가 올바르게 명시되었는지 확인합니다.

---

#### **Ticket 1.2: 핵심 프로토콜 및 데이터 모델 정의 (`pk-protocol` 크레이트)**

**Goal**: `core`와 `tui` 간의 통신 및 설정 파일 파싱에 사용될 핵심 데이터 구조를 `pk-protocol` 크레이트에 정의합니다. 이 크레이트는 의존성을 최소화하여 다른 부분과 독립적으로 컴파일될 수 있어야 합니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/protocol/src/lib.rs`: 모듈을 정의하는 메인 파일.
- `pipeline.rs`: `Pipeline`, `ProcessStep` 구조체 정의.
- `agent.rs`: `Agent` 구조체 및 관련 타입 정의.
- `process.rs`: `Process`, `ProcessStatus` 열거형 정의.
- `op.rs`, `event.rs`: TUI와 Core 간의 통신을 위한 `Op`, `Event` 열거형 정의.

**Interfaces**:

- 모든 구조체는 `Serialize`, `Deserialize`, `Debug`, `Clone`을 derive해야 합니다.
- TypeScript 연동을 위해 `ts_rs::TS`를 derive해야 합니다.
- `ProcessStatus`는 `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`를 사용하여 `HUMAN_REVIEW`와 같은 형식을 유지합니다.

**Reference Code**:

- **Op/Event 패턴**: `codex-rs/protocol/src/protocol.rs`의 `Op`와 `Event` 열거형 구조를 참고하세요.
- **데이터 구조체**: `codex-rs/protocol/src/config_types.rs` 와 `codex-rs/protocol/src/models.rs`의 `serde` 및 `ts_rs` 사용법을 참고하세요.

**Guidelines & Conventions**:

- YAML 파일의 `process` 리스트는 `Vec<ProcessStep>`으로 매핑됩니다. `ProcessStep`은 `String` 또는 `enum`으로 정의하여 `HUMAN_REVIEW` 같은 특수 명령어를 처리할 수 있도록 합니다.

**Acceptance Tests (TDD Process)**:

1.  **RED**: `pk-protocol` 크레이트에 테스트 파일을 만들고, 아직 존재하지 않는 `Pipeline`, `Agent` 등의 구조체를 `serde_json`으로 직렬화/역직렬화하려는 테스트 코드를 작성하여 컴파일 에러를 발생시킵니다.
2.  **GREEN**: `pipeline.rs`, `agent.rs` 등에 필요한 구조체와 열거형을 정의하고, `serde` 및 `ts_rs` derive를 추가하여 테스트가 통과하도록 합니다.
3.  **REFACTOR**: 각 구조체와 필드에 명확한 문서 주석(doc comments)을 추가하고, 모듈 구조를 논리적으로 정리합니다.

---

### **Phase 2: 설정 로딩 및 에이전트 추상화**

> **목표**: 설정 파일을 읽어오는 로직과, 다양한 AI 에이전트를 일관된 방식으로 다루기 위한 추상화 계층을 구현합니다. 두 Ticket은 `pk-core` 크레이트 내에서 다른 모듈을 수정하므로 병렬 진행이 가능합니다.

---

#### **Ticket 2.1: 설정 파일 로더 구현 (`pk-core` 크레이트)**

**Goal**: `.pipeline-kit/` 디렉터리 내의 `config.toml`, `agents/*.md`, `pipelines/*.yaml` 파일을 읽고 파싱하여, Phase 1에서 정의한 Rust 구조체로 변환하는 `ConfigLoader` 모듈을 `pk-core`에 구현합니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/core/src/config/loader.rs`: 설정 파일 로딩 및 파싱 로직 구현.
- `pipeline-kit-rs/crates/core/src/config/models.rs`: 파싱된 설정을 담을 `AppConfig` 구조체 정의.

**Interfaces**:

- `pub fn load_config(root_path: &Path) -> Result<AppConfig, ConfigError>`: 주어진 경로에서 모든 설정 파일을 읽어 하나의 `AppConfig` 객체로 합쳐 반환합니다.

**Reference Code**:

- `codex-rs/core/src/config.rs`의 `Config::load_with_cli_overrides` 함수와 TOML 파싱 로직을 참고하세요. `serde_yaml`과 `gray_matter` (Markdown Front Matter 파싱용) 크레이트를 추가로 활용해야 합니다.

**Guidelines & Conventions**:

- 에이전트 Markdown 파일은 `gray_matter`를 사용해 Front Matter(속성)와 content(시스템 프롬프트)를 분리하여 파싱합니다.
- 파일 I/O 에러, 파싱 에러 등을 상세히 다루는 `ConfigError` 열거형을 `thiserror`를 사용해 정의하세요.

**Acceptance Tests (TDD Process)**:

1.  **RED**: 임시 디렉터리에 `.pipeline-kit` 구조와 예제 설정 파일들을 생성하는 테스트를 작성합니다. `ConfigLoader::load_config`를 호출하고, 아직 구현되지 않았으므로 실패해야 합니다.
2.  **GREEN**: `serde_yaml`, `toml`, `gray_matter`를 사용하여 각 파일을 파싱하고 `AppConfig` 구조체를 채우는 로직을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: 오류 처리 로직을 개선하고, 설정 파일이 없는 경우 등 엣지 케이스에 대한 테스트를 보강합니다.

---

#### **Ticket 2.2: 에이전트 어댑터 패턴 및 관리자 구현 (`pk-core` 크레이트)**

**Goal**: 다양한 AI 코딩 서비스를 일관된 인터페이스로 다루기 위한 `Agent` 트레이트(Adapter Pattern)와 이를 관리하는 `AgentManager`를 `pk-core`에 구현합니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/core/src/agents/base.rs`: `Agent` 트레이트 정의.
- `pipeline-kit-rs/crates/core/src/agents/adapters/mock.rs`: 테스트를 위한 `MockAgent` 구현.
- `pipeline-kit-rs/crates/core/src/agents/manager.rs`: `AgentManager` 구현.

**Interfaces**:

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

**Reference Code**:

- **어댑터 패턴**: 제공된 `apps/api/app/services/cli/base.py`의 `BaseCLI`와 `adapters/` 구조에서 영감을 얻으세요. Rust에서는 이를 `trait`과 구현체로 표현합니다.

**Guidelines & Conventions**:

- 초기에는 `MockAgent`만 구현하여 `PipelineEngine` 개발을 용이하게 합니다. `MockAgent`는 미리 정해진 응답을 반환하여 예측 가능한 테스트를 가능하게 합니다.
- `AgentManager`는 `AppConfig`로부터 에이전트 설정을 받아 내부 `HashMap`에 `Arc<dyn Agent>` 형태로 저장합니다.

**Acceptance Tests (TDD Process)**:

1.  **RED**: `AgentManager`가 `MockAgent`를 반환하는지 확인하는 테스트를 작성합니다. `Agent` 트레이트가 없으므로 컴파일이 실패합니다.
2.  **GREEN**: `Agent` 트레이트, `MockAgent` 구현체, 그리고 `AgentManager`를 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: `Agent` 트레이트의 메서드 시그니처를 명확히 하고, `AgentManager`의 에러 처리(에이전트를 찾지 못한 경우)를 개선합니다.

---

### **Phase 3: 파이프라인 엔진 및 기본 CLI 기능 구현**

> **목표**: 파이프라인을 실제로 실행하고 상태를 관리하는 핵심 엔진을 구현하고, 이를 실행할 수 있는 기본적인 비대화형 CLI 명령어를 완성합니다.

---

#### **Ticket 3.1: 파이프라인 실행 엔진 및 상태 관리 구현 (`pk-core` 크레이트)**

**Goal**: `PipelineEngine`을 구현하여 `Pipeline` 정의에 따라 에이전트를 순차적으로 실행하고, `Process`의 상태를 관리합니다. 이 엔진은 비동기로 동작해야 합니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/core/src/engine/mod.rs`: `PipelineEngine` 구현.
- `pipeline-kit-rs/crates/core/src/state/process.rs`: `Process` 상태 머신 로직 구현.
- `pipeline-kit-rs/crates/core/src/state/manager.rs`: 모든 활성 `Process`를 관리하는 `StateManager` 구현.

**Interfaces**:

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

**Reference Code**:

- **비동기 작업 루프**: `codex-rs/core/src/codex.rs`의 `submission_loop`과 `run_task` 함수에서 비동기 채널과 `tokio::spawn`을 사용한 이벤트 기반 아키텍처를 참고하세요.

**Guidelines & Conventions**:

- `PipelineEngine::run`은 `tokio::spawn`을 사용해 백그라운드 태스크에서 실행되어야 합니다.
- `Process`의 상태(`status`)가 변경될 때마다 `events_tx` 채널을 통해 `ProcessStatusUpdate` 이벤트를 전송해야 합니다.
- `HUMAN_REVIEW` 단계에 도달하면 `ProcessStatus`를 `HumanReview`로 변경하고 태스크 실행을 멈춥니다.

**Acceptance Tests (TDD Process)**:

1.  **RED**: `MockAgent`를 사용하는 간단한 순차 파이프라인을 정의하고, `PipelineEngine`이 이를 실행하여 예상된 순서대로 `ProcessStatusUpdate` 이벤트를 발생시키는지, 그리고 `HUMAN_REVIEW`에서 멈추는지 검증하는 테스트를 작성합니다.
2.  **GREEN**: `PipelineEngine`과 `Process` 상태 머신을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: `Arc<Mutex<>>`를 사용하여 `Process` 상태를 스레드-안전하게 만들고, `StateManager`를 도입하여 여러 프로세스를 관리할 수 있는 기반을 마련합니다.

---

### **Phase 4: TUI 구현**

> **목표**: `ratatui`를 사용하여 모든 기능을 제어하고 모니터링할 수 있는 완전한 대화형 TUI를 구축합니다. 각 위젯은 독립적으로 개발 가능하므로 병렬 작업에 적합합니다.

---

#### **Ticket 4.1: TUI 애플리케이션 셸 및 이벤트 루프 구축 (`pk-tui` 크레이트)**

**Goal**: TUI의 기본 레이아웃, 메인 이벤트 루프, 그리고 `core`와의 통신 채널을 설정합니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/tui/src/app.rs`: `App` 구조체(TUI 상태 관리) 및 메인 루프.
- `pipeline-kit-rs/crates/tui/src/main.rs`: TUI 애플리케이션 시작점.

**Reference Code**:

- `codex-rs/tui/src/app.rs`의 `App::run` 메서드와 이벤트 처리 루프(`select!`)를 핵심 로직으로 참고하세요.
- `codex-rs/tui/src/tui.rs`에서 터미널 초기화 및 복원 로직을 가져옵니다.

**Acceptance Tests (TDD Process)**:

1.  **RED**: `App::run`을 호출하는 테스트를 작성합니다. `TestBackend`를 사용하여 UI가 렌더링되는지 확인하지만, 아직 위젯이 없으므로 빈 화면이어야 합니다.
2.  **GREEN**: `ratatui`를 사용하여 기본 레이아웃(대시보드, 상세, 입력창 영역)을 그리고, `crossterm` 이벤트 스트림을 처리하는 기본 루프를 구현합니다. `q` 키를 누르면 종료되는지 테스트합니다.
3.  **REFACTOR**: TUI 상태와 비즈니스 로직을 분리하고, 이벤트 처리를 별도 모듈(`event_handler.rs`)로 분리하는 것을 고려합니다.

---

#### **Ticket 4.2 - 4.4 (병렬 진행 가능)**

> 각 Ticket은 `tui` 크레이트 내 별도의 위젯 모듈을 구현합니다. 모든 위젯은 `app.rs`의 `App` 구조체에 저장된 상태를 읽어 렌더링하고, 사용자 입력에 따라 `Op` 이벤트를 발생시킵니다.

- **Ticket 4.2: 대시보드 위젯 구현**
  - **Goal**: `StateManager`가 관리하는 모든 `Process`의 ID, 이름, 상태, 현재 단계를 테이블 형태로 실시간 표시합니다.
  - **Reference**: `codex-rs/tui/src/chatwidget.rs`의 `render` 메서드에서 리스트 형태의 데이터를 렌더링하는 방식을 참고하세요.
- **Ticket 4.3: 프로세스 상세 뷰 위젯 구현**
  - **Goal**: 특정 `Process`를 선택했을 때, 해당 프로세스의 전체 로그와 상세 정보를 스크롤 가능한 뷰로 표시합니다.
  - **Reference**: `codex-rs/tui/src/pager_overlay.rs`의 `PagerView` 구현을 참고하여 스크롤 가능한 텍스트 뷰를 만드세요.
- **Ticket 4.4: 슬래시 커맨드 컴포저 위젯 구현**
  - **Goal**: `/start`, `/pause` 등 슬래시로 시작하는 명령어를 입력하고, 자동 완성 제안을 받을 수 있는 입력창을 구현합니다.
  - **Reference**: `codex-rs/tui/src/bottom_pane/chat_composer.rs`와 `command_popup.rs`는 이 기능의 완벽한 레퍼런스입니다.

---

### **Phase 5: 배포 준비**

> **목표**: `npm`을 통해 `pipeline-kit`을 배포하기 위한 모든 준비를 마칩니다.

---

#### **Ticket 5.1: TypeScript 래퍼 및 npm 패키징 (`pipeline-kit-cli` 디렉터리)**

**Goal**: `pipeline-kit-cli` 디렉터리를 설정하고, `npm install` 시 Rust 바이너리를 다운로드하여 실행할 수 있는 `pipeline-kit.js` 런처와 `package.json`을 작성합니다.

**Reference Code**:

- `codex-cli/` 디렉터리의 모든 파일을 거의 그대로 복제하고, 'codex'를 'pipeline-kit'으로 변경하는 작업이 주가 됩니다.
- `codex-cli/bin/codex.js`와 `codex-cli/scripts/install_native_deps.sh`를 주의 깊게 분석하세요.

**Acceptance Tests (TDD Process)**:

1.  **RED**: `pipeline-kit-cli` 디렉터리에서 `npm install`을 실행하고, `bin/pipeline-kit` 스크립트를 실행했을 때, "바이너리를 찾을 수 없음" 에러가 발생하는 것을 확인합니다.
2.  **GREEN**: `package.json`의 `postinstall` 스크립트와 `install_native_deps.sh`를 구현하여 (임시로 로컬 빌드된 바이너리를 복사하도록 하여) `pipeline-kit` 명령어가 성공적으로 Rust 프로세스를 실행하는지 확인합니다.
3.  **REFACTOR**: 스크립트의 하드코딩된 경로를 변수화하고, 오류 메시지를 사용자 친화적으로 개선합니다.

---

### **자체 점검 (Self-Correction)**

1.  **Phase 순서는 합리적인가?**
    - 예. `기반 구축 → 핵심 로직 구현 → UI 구현 → 배포 준비` 순서는 의존성에 기반한 합리적인 순서입니다.
2.  **같은 Phase의 Ticket 간 구현 충돌이 없는가?**
    - 예. Phase 1, 2, 4의 Ticket들은 각각 독립적인 크레이트나 모듈을 대상으로 하므로 병렬 작업 시 코드 충돌이 발생하지 않습니다.
3.  **각 Ticket의 프롬프트가 구현 가능한 수준으로 자세한가?**
    - 예. 각 Ticket은 목표, 수정할 파일/모듈, 따라야 할 인터페이스, 참고할 코드, 그리고 명확한 테스트 케이스를 포함하고 있어 코딩 에이전트가 이해하고 작업을 수행하기에 충분히 구체적입니다.
4.  **각 Ticket은 테스트 가능한가?**
    - 예. 모든 Ticket에 TDD 프로세스와 명확한 Acceptance Test가 명시되어 있습니다. 특히 Mock 객체(`MockAgent`)를 활용하여 의존성을 분리하고 단위 테스트를 용이하게 하는 전략을 포함했습니다.
