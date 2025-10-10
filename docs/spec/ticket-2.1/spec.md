# Ticket 2.1: 설정 파일 로더 구현 (`pk-core` 크레이트)

## Goal

`.pipeline-kit/` 디렉터리 내의 `config.toml`, `agents/*.md`, `pipelines/*.yaml` 파일을 읽고 파싱하여, Phase 1에서 정의한 Rust 구조체로 변환하는 `ConfigLoader` 모듈을 `pk-core`에 구현합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/core/src/config/loader.rs`: 설정 파일 로딩 및 파싱 로직 구현.
- `pipeline-kit-rs/crates/core/src/config/models.rs`: 파싱된 설정을 담을 `AppConfig` 구조체 정의.

## Interfaces

- `pub fn load_config(root_path: &Path) -> Result<AppConfig, ConfigError>`: 주어진 경로에서 모든 설정 파일을 읽어 하나의 `AppConfig` 객체로 합쳐 반환합니다.

## Reference Code

- `codex-rs/core/src/config.rs`의 `Config::load_with_cli_overrides` 함수와 TOML 파싱 로직을 참고하세요. `serde_yaml`과 `gray_matter` (Markdown Front Matter 파싱용) 크레이트를 추가로 활용해야 합니다.

## Guidelines & Conventions

- 에이전트 Markdown 파일은 `gray_matter`를 사용해 Front Matter(속성)와 content(시스템 프롬프트)를 분리하여 파싱합니다.
- 파일 I/O 에러, 파싱 에러 등을 상세히 다루는 `ConfigError` 열거형을 `thiserror`를 사용해 정의하세요.

## Acceptance Tests (TDD Process)

1.  **RED**: 임시 디렉터리에 `.pipeline-kit` 구조와 예제 설정 파일들을 생성하는 테스트를 작성합니다. `ConfigLoader::load_config`를 호출하고, 아직 구현되지 않았으므로 실패해야 합니다.
2.  **GREEN**: `serde_yaml`, `toml`, `gray_matter`를 사용하여 각 파일을 파싱하고 `AppConfig` 구조체를 채우는 로직을 구현하여 테스트를 통과시킵니다.
3.  **REFACTOR**: 오류 처리 로직을 개선하고, 설정 파일이 없는 경우 등 엣지 케이스에 대한 테스트를 보강합니다.
