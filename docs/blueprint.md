### 1. 모노레포 설계 (Monorepo Architecture)

`codex-cli`와 유사하게, **Rust의 강력한 성능과 안정성**을 핵심 로직에 사용하고, **TypeScript/npm의 광범위한 생태계와 배포 편의성**을 활용하는 구조를 채택합니다.

```
/pipeline-kit (monorepo root)
├── pipeline-kit-cli/      # TypeScript/npm Wrapper for distribution and execution
│   ├── bin/
│   │   └── pipeline-kit.js  # Platform-aware Rust binary launcher
│   ├── scripts/             # Build/release scripts
│   └── package.json         # npm package definition
│
├── pipeline-kit-rs/       # Rust Core Logic (Cargo Workspace)
│   ├── crates/
│   │   ├── cli/             # Main binary entry point (command parsing)
│   │   ├── core/            # Core business logic: pipeline engine, agent management
│   │   ├── tui/             # Terminal User Interface (ratatui)
│   │   ├── protocol/        # Data structures for internal communication (Op, Event)
│   │   └── protocol-ts/     # TypeScript type generation from Rust types
│   ├── Cargo.toml           # Workspace manifest
│   └── justfile             # Task runner for build, format, test
│
├── docs/                    # Project documentation
├── .github/                 # CI/CD workflows
└── package.json             # Root package.json for monorepo scripts (e.g., pnpm)
```

### 2. 각 서비스의 구조 및 주요 모듈화 설계

#### 2.1. `pipeline-kit-cli` (TypeScript Wrapper)

- **목표**: `npm i -g pipeline-kit`을 통한 손쉬운 설치와 실행.
- **`bin/pipeline-kit.js`**:
  - 사용자의 OS와 아키텍처를 확인합니다.
  - 플랫폼에 맞는 Rust 바이너리(`pipeline-kit-macos-aarch64`, `pipeline-kit-linux-x86_64` 등)의 경로를 결정합니다.
  - Node.js의 `child_process.spawn`을 사용해 해당 바이너리를 실행하며, 모든 CLI 인수와 `stdio`를 그대로 전달합니다.
  - 사용자로부터 오는 `SIGINT` (Ctrl-C) 같은 시그널을 자식 프로세스(Rust)에 전달하여 정상적인 종료를 유도합니다.

#### 2.2. `pipeline-kit-rs` (Rust Core Workspace)

##### **`crates/protocol`**

- **역할**: `tui`와 `core` 간의 통신 계약을 정의합니다. 외부 클라이언트(예: IDE 확장)와의 연동 기반이 됩니다.
- **주요 모듈**:
  - `op.rs`: UI에서 코어로 보내는 명령(Operation).
    ```rust
    pub enum Op {
        StartPipeline { name: String, reference_file: Option<PathBuf> },
        PauseProcess { process_id: Uuid },
        ResumeProcess { process_id: Uuid },
        KillProcess { process_id: Uuid },
        GetDashboardState,
        GetProcessDetail { process_id: Uuid },
        Shutdown,
    }
    ```
  - `event.rs`: 코어에서 UI로 보내는 상태 업데이트 및 결과.
    ```rust
    pub enum Event {
        ProcessStarted { process_id: Uuid, pipeline_name: String },
        ProcessStatusUpdate { process_id: Uuid, status: ProcessStatus, step: usize },
        ProcessLogChunk { process_id: Uuid, content: String },
        ProcessCompleted { process_id: Uuid },
        ProcessError { process_id: Uuid, error: String },
        DashboardStateUpdate(DashboardState),
        ProcessDetailUpdate(ProcessDetail),
    }
    ```

##### **`crates/core`**

- **역할**: 애플리케이션의 두뇌. 모든 비즈니스 로직을 처리합니다.
- **주요 모듈**:
  - **`config.rs`**: `.pipeline-kit/` 디렉터리의 `config.toml`, `pipelines/*.yaml`, `agents/*.md` 파일을 파싱하여 Rust 구조체로 변환하는 로직을 담당합니다. `serde`, `serde_yaml`, `toml` 크레이트를 활용합니다.
  - **`agent_manager.rs` (어댑터 패턴)**:
    - **`trait Agent`**: 모든 AI 에이전트가 구현해야 할 공통 인터페이스를 정의합니다.
    ```rust
    pub trait Agent {
        async fn execute(&self, context: &ExecutionContext) -> Result<AgentOutput>;
    }
    ```
    - **`adapters/` (모듈)**: 각 AI 서비스(OpenAI, Claude, Gemini 등)에 대한 구체적인 `Agent` 구현체(어댑터)를 포함합니다. 각 어댑터는 해당 서비스의 SDK나 API 호출 방식을 캡슐화하고 결과를 표준 `AgentOutput`으로 변환합니다.
    - **`AgentManager`**: `agents/*.md` 파일을 로드하여 `Agent` 인스턴스를 생성하고 관리하는 팩토리 역할을 합니다.
  - **`pipeline_engine.rs`**:
    - **`struct Process`**: 실행 중인 파이프라인의 상태(`process_id`, `status`, `current_step`, `logs` 등)를 관리하는 상태 머신입니다.
    - `PipelineEngine`: `Process`를 생성하고, `pipelines/*.yaml`에 정의된 단계를 순차적으로 또는 병렬로 실행합니다. `AgentManager`를 통해 필요한 에이전트를 호출하고, `HUMAN_REVIEW` 단계에서 실행을 멈춥니다.
  - **`state_manager.rs`**:
    - 모든 활성 `Process` 인스턴스를 `HashMap<Uuid, Arc<Mutex<Process>>>` 형태로 관리하여 동시 접근을 제어합니다.
    - `pause`, `resume`, `kill` 등의 명령을 받아 해당 `Process`의 상태를 변경합니다.
    - (선택) 활성 프로세스 상태를 파일에 주기적으로 저장하여 CLI 재시작 시 복구할 수 있는 기능을 제공합니다.

##### **`crates/tui`**

- **역할**: 대화형 터미널 UI를 구현합니다. `ratatui` 크레이트를 사용합니다.
- **주요 모듈**:
  - `app.rs`: TUI의 메인 애플리케이션 루프 및 상태 관리를 담당합니다. `core`와 채널을 통해 `Op`/`Event`를 주고받습니다.
  - `widgets/`:
    - `dashboard.rs`: 여러 프로세스의 상태를 요약하여 보여주는 위젯.
    - `process_detail.rs`: 단일 프로세스의 상세 로그와 정보를 보여주는 위젯.
    - `composer.rs`: 슬래시 커맨드 입력 및 제안을 처리하는 하단 입력창 위젯.
  - `event_handler.rs`: 키보드 입력, 리사이즈 등 터미널 이벤트를 처리하고 적절한 `Op`를 생성하여 `core`로 전송합니다.

##### **`crates/cli`**

- **역할**: Rust 바이너리의 진입점. `clap` 크레이트를 사용하여 커맨드라인 인수를 파싱합니다.
- **`main.rs`**:
  - `pipeline` 또는 인수 없음: `tui` 모듈의 `run_app()`을 호출합니다.
  - `pipeline start <name>`: `core`의 `PipelineEngine`을 직접 호출하고, `Event`를 `stdout`에 스트리밍합니다. (비대화형 모드)
  - `pipeline pause|resume|kill <id>`: `core`의 `StateManager`와 상호작용하는 간단한 클라이언트 로직을 수행합니다.

### 3. 합리적인 구현 계획 (Phased Implementation Plan)

이 프로젝트는 복잡도가 있으므로, 단계별로 기능을 완성해나가는 것이 안정적입니다.

#### **Phase 1: 코어 엔진 및 비대화형(Non-interactive) CLI 구축**

> **목표**: TUI 없이 커맨드라인에서 파이프라인을 실행하고 결과를 확인할 수 있는 기본 기능 완성.

1.  **프로젝트 구조 설정**: `cargo workspace`와 `pnpm workspace`를 포함한 모노레포 구조를 생성합니다.
2.  **프로토콜 정의 (`protocol` crate)**: `Op`, `Event` 및 `Agent`, `Pipeline`의 Rust 구조체를 정의합니다.
3.  **설정 로딩 (`core` crate)**: `.pipeline-kit/` 디렉터리의 YAML, TOML, Markdown 파일을 파싱하여 `Config` 객체를 만드는 기능을 구현합니다.
4.  **에이전트 어댑터 구현 (`core` crate)**: `Agent` 트레이트를 정의하고, 먼저 하나의 AI 서비스(예: OpenAI)에 대한 어댑터를 구현합니다. 처음에는 Mock Agent를 만들어 테스트 용이성을 확보하는 것도 좋은 방법입니다.
5.  **파이프라인 엔진 구현 (`core` crate)**: 단일 파이프라인을 순차적으로 실행하는 기본 엔진을 만듭니다. `HUMAN_REVIEW`에서 멈추는 기능까지 구현합니다.
6.  **기본 CLI (`cli` crate)**: `pipeline start <name>` 명령어를 구현하여 파이프라인 엔진을 실행하고, 발생하는 `Event`를 `stdout`에 텍스트로 출력합니다.

#### **Phase 2: TUI 및 대화형 기능 구현**

> **목표**: 사용자가 파이프라인을 시각적으로 모니터링하고 상호작용할 수 있는 TUI 완성.

1.  **TUI 기본 구조 (`tui` crate)**: `ratatui`를 사용하여 기본 화면 레이아웃과 이벤트 루프를 설정합니다.
2.  **Core ↔ TUI 연동**: `tokio::mpsc` 채널을 사용하여 `cli`에서 `core`와 `tui`를 각각 별도의 스레드(또는 태스크)로 실행하고, `Op`/`Event` 프로토콜로 통신하도록 연결합니다.
3.  **대시보드 위젯**: 실행 중인 모든 프로세스의 상태를 실시간으로 보여주는 대시보드 UI를 구현합니다.
4.  **상세 뷰 위젯**: 특정 프로세스를 선택했을 때 상세 로그와 정보를 보여주는 UI를 구현합니다.
5.  **컴포저 및 슬래시 커맨드**: 하단 입력창에서 `/start`, `/pause` 등의 명령어를 입력하고 자동 완성 제안을 받을 수 있는 기능을 구현합니다.

#### **Phase 3: 프로세스 관리 및 고급 기능**

> **목표**: 여러 파이프라인을 동시에 안정적으로 관리하고 제어하는 기능 완성.

1.  **동시 프로세스 관리 (`core` crate)**: `StateManager`를 구현하여 여러 파이프라인 프로세스를 동시에 관리할 수 있도록 개선합니다. `Arc<Mutex<>>`를 활용하여 스레드 안전성을 확보합니다.
2.  **프로세스 제어 구현**: `pause`, `resume`, `kill` 로직을 `PipelineEngine`과 `StateManager`에 구현하고, TUI의 슬래시 커맨드와 연동합니다.
3.  **상태 복구**: CLI가 재시작되어도 진행 중이던 `PAUSE`, `HUMAN_REVIEW` 상태의 프로세스를 복구할 수 있도록 상태 지속성(Persistence) 기능을 구현합니다.
4.  **참조 파일 처리**: `pipeline start` 명령어 실행 시, `required-reference-file`이 누락된 경우 TUI를 통해 사용자에게 파일 입력을 요청하는 로직을 추가합니다.

#### **Phase 4: 배포 및 패키징**

> **목표**: `npm`을 통해 누구나 쉽게 설치하고 사용할 수 있도록 패키징.

1.  **TypeScript 래퍼 (`pipeline-kit-cli`)**: `codex-cli`를 참고하여 플랫폼별 바이너리를 실행하는 `pipeline-kit.js` 런처를 작성합니다.
2.  **빌드 자동화 (GitHub Actions)**: `rust-ci.yml`과 `rust-release.yml`을 수정하여 다양한 플랫폼(macOS, Linux, Windows 등)용 Rust 바이너리를 자동으로 빌드하고 압축하여 GitHub Release에 아티팩트로 업로드하는 워크플로우를 구성합니다.
3.  **npm 패키지 설정**: `pipeline-kit-cli/package.json`의 `postinstall` 스크립트를 사용하여 `npm install` 시점에 GitHub Release에서 적절한 바이너리를 다운로드하도록 설정합니다.
4.  **배포**: `npm publish`를 통해 패키지를 npm 레지스트리에 배포합니다.
