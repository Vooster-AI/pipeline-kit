# **Ticket 1.1: 모노레포 및 Cargo 워크스페이스 구조 설정**

**Goal**: `pipeline-kit` 프로젝트의 전체 디렉터리 구조와 Rust Cargo 워크스페이스를 설정합니다. 이는 모든 향후 개발의 기반이 되며, `codex-cli`의 성공적인 아키텍처를 그대로 재현하는 첫 단계입니다.

**Core Modules & Roles**:

- **`pipeline-kit/`**: 모노레포 최상위 루트 디렉터리.
- **`pipeline-kit/pipeline-kit-cli/`**: TypeScript/npm 래퍼 애플리케이션이 위치할 디렉터리.
- **`pipeline-kit/pipeline-kit-rs/`**: 모든 Rust 소스 코드를 포함하는 Cargo 워크스페이스 루트.
- **`pipeline-kit/pipeline-kit-rs/crates/`**: 개별 Rust 크레이트들이 위치할 디렉터리.
- **`pipeline-kit/pipeline-kit-rs/Cargo.toml`**: 워크스페이스 전체를 정의하는 최상위 Cargo manifest 파일.

**Reference Code**:

- `codex-rs/` 디렉터리 구조를 정확히 참고하여 하위 디렉터리들을 생성합니다.
- `codex-rs/Cargo.toml` 파일의 `[workspace]` 섹션을 참고하여 워크스페이스 멤버를 정의합니다.
- 최상위 `pnpm-workspace.yaml`과 `package.json` 파일을 참고하여 `pnpm` 모노레포 환경을 구성합니다.

**Guidelines & Conventions**:

- Rust 크레이트 이름은 `pk-` 접두사를 사용합니다. (예: `pk-core`, `pk-protocol`)
- 초기에는 빈 `lib.rs` (라이브러리 크레이트용) 또는 `main.rs` (바이너리 크레이트용) 파일을 포함하여 각 크레이트 디렉터리를 생성합니다.

**Detailed Implementation Steps**:

1.  **최상위 디렉터리 및 파일 생성**:

    - `pipeline-kit/` 디렉터리를 생성합니다.
    - `pipeline-kit/pnpm-workspace.yaml` 파일을 생성하고 다음 내용을 작성합니다:
      ```yaml
      packages:
        - "pipeline-kit-cli"
      ```
    - `pipeline-kit/package.json` 파일을 생성하고 최소한의 내용을 작성합니다 (추후 스크립트 추가):
      ```json
      {
        "name": "pipeline-kit-monorepo",
        "private": true,
        "scripts": {
          "check-rs": "cd pipeline-kit-rs && cargo check --workspace"
        }
      }
      ```

2.  **TypeScript 래퍼 디렉터리 생성**:

    - `pipeline-kit/pipeline-kit-cli/` 디렉터리를 생성합니다.
    - `pipeline-kit/pipeline-kit-cli/bin/` 디렉터리를 생성합니다.
    - `pipeline-kit/pipeline-kit-cli/bin/pipeline-kit.js` 파일을 생성하고 shebang만 추가합니다.
      ````javascript
      #!/usr/bin/env node
      ```    -   `pipeline-kit/pipeline-kit-cli/package.json` 파일을 생성하고 다음 내용을 작성합니다:
      ```json
      {
        "name": "pipeline-kit",
        "version": "0.0.1",
        "bin": {
          "pipeline": "bin/pipeline-kit.js"
        }
      }
      ````

3.  **Rust 워크스페이스 디렉터리 및 파일 생성**:

    - `pipeline-kit/pipeline-kit-rs/` 디렉터리를 생성합니다.
    - `pipeline-kit/pipeline-kit-rs/crates/` 디렉터리를 생성합니다.
    - **`pipeline-kit/pipeline-kit-rs/Cargo.toml`** 파일을 생성하고 다음 `[workspace]` 설정을 작성합니다.

      ```toml
      [workspace]
      members = [
          "crates/cli",
          "crates/core",
          "crates/protocol",
          "crates/protocol-ts",
          "crates/tui",
      ]
      resolver = "2" # Use the modern resolver

      [workspace.package]
      version = "0.1.0"
      edition = "2021" # Or "2024" if your toolchain supports it

      [workspace.lints.clippy]
      # Reference codex-rs/clippy.toml for useful lints
      unwrap_used = "deny"
      ```

    - 아래 5개의 크레이트 디렉터리를 `crates/` 내부에 생성합니다. 각 디렉터리는 `src/` 폴더와 `Cargo.toml` 파일을 포함해야 합니다.

      - **`crates/cli` (바이너리)**
        - `src/main.rs`: `fn main() {}`
        - `Cargo.toml`:

          ```toml
          [package]
          name = "pk-cli"
          version = { workspace = true }
          edition = { workspace = true }

          [[bin]]
          name = "pipeline"
          path = "src/main.rs"
          ```
      - **`crates/core` (라이브러리)**
        - `src/lib.rs`: (비어 있음)
        - `Cargo.toml`:
          ```toml
          [package]
          name = "pk-core"
          version = { workspace = true }
          edition = { workspace = true }
          ```
      - **`crates/protocol` (라이브러리)**
        - `src/lib.rs`: (비어 있음)
        - `Cargo.toml`:
          ```toml
          [package]
          name = "pk-protocol"
          version = { workspace = true }
          edition = { workspace = true }
          ```
      - **`crates/protocol-ts` (라이브러리)**
        - `src/lib.rs`: (비어 있음)
        - `Cargo.toml`:
          ```toml
          [package]
          name = "pk-protocol-ts"
          version = { workspace = true }
          edition = { workspace = true }
          ```
      - **`crates/tui` (라이브러리)**
        - `src/lib.rs`: (비어 있음)
        - `Cargo.toml`:
          ```toml
          [package]
          name = "pk-tui"
          version = { workspace = true }
          edition = { workspace = true }
          ```

**Acceptance Tests (TDD Process)**:

1.  **RED**:

    - `pipeline-kit` 디렉터리 생성 후, `pipeline-kit/pipeline-kit-rs/` 로 이동하여 `cargo check --workspace`를 실행합니다. `Cargo.toml`이 없으므로 실패합니다.
    - 최상위 `pipeline-kit` 디렉터리에서 `pnpm install`을 실행합니다. `pnpm-workspace.yaml` 파일이 없으므로 실패합니다.

2.  **GREEN**:

    - 위에 명시된 "Detailed Implementation Steps"를 모두 수행합니다.
    - `pipeline-kit/pipeline-kit-rs/` 디렉터리에서 `cargo check --workspace`를 실행하여 모든 크레이트가 워크스페이스의 멤버로 올바르게 인식되고 컴파일 경고 없이 성공하는지 확인합니다.
    - 최상위 `pipeline-kit` 디렉터리에서 `pnpm install`을 실행하여 `pipeline-kit-cli` 패키지가 워크스페이스의 일부로 설치되는지 확인합니다.

3.  **REFACTOR**:
    - 각 크레이트의 `Cargo.toml` 파일에 `description`과 `authors` 필드를 추가하여 명확성을 높입니다.
    - `pipeline-kit-rs/Cargo.toml`의 `[workspace.lints]` 설정을 `codex-rs/clippy.toml`과 유사하게 채워서 코드 품질 규칙의 기반을 마련합니다.
    - 불필요한 파일이나 디렉터리가 생성되지 않았는지 최종적으로 검토합니다.

---

이 명세는 코딩 에이전트가 `codex-cli` 프로젝트를 완벽하게 참조하여 `pipeline-kit`의 초기 구조를 설정하는 데 필요한 모든 정보를 담고 있습니다.
