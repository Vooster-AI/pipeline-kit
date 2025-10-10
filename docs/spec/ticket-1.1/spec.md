# Ticket 1.1: 모노레포 및 Cargo 워크스페이스 구조 설정

## Goal

`pipeline-kit` 프로젝트의 전체 디렉터리 구조와 Rust Cargo 워크스페이스를 설정합니다. 이는 모든 향후 개발의 기반이 됩니다.

## Core Modules & Roles

- `pipeline-kit/`: 모노레포 최상위 루트.
- `pipeline-kit/pipeline-kit-cli/`: TypeScript 래퍼 디렉터리.
- `pipeline-kit/pipeline-kit-rs/`: Rust 워크스페이스 루트.
- `pipeline-kit/pipeline-kit-rs/crates/`: 모든 Rust 크레이트가 위치할 디렉터리.
- `pipeline-kit/pipeline-kit-rs/Cargo.toml`: 워크스페이스를 정의하는 최상위 Cargo manifest.

## Reference Code

- `codex-rs/` 디렉터리 구조 및 최상위 `codex-rs/Cargo.toml`의 `[workspace]` 설정을 참고하세요.
- 최상위 `pnpm-workspace.yaml` 파일을 참고하여 모노레포를 구성하세요.

## Guidelines & Conventions

- Rust 크레이트 이름은 `pk-` 접두사를 사용합니다. (예: `pk-core`, `pk-tui`)
- 초기에는 빈 `lib.rs` 또는 `main.rs` 파일을 포함하여 각 크레이트 디렉터리(`cli`, `core`, `tui`, `protocol`, `protocol-ts`)를 생성합니다.

## Acceptance Tests (TDD Process)

1.  **RED**: `cargo check --workspace` 명령어가 실패하는 상태로 시작합니다 (파일이 없으므로).
2.  **GREEN**: 위의 디렉터리 구조와 `Cargo.toml` 워크스페이스 파일을 생성하여 `cargo check --workspace` 명령어가 성공적으로 완료되도록 합니다.
3.  **REFACTOR**: 생성된 파일 구조가 명확하고, `Cargo.toml`에 각 멤버가 올바르게 명시되었는지 확인합니다.
