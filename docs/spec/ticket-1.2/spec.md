# Ticket 1.2: 핵심 프로토콜 및 데이터 모델 정의 (`pk-protocol` 크레이트)

## Goal

`core`와 `tui` 간의 통신 및 설정 파일 파싱에 사용될 핵심 데이터 구조를 `pk-protocol` 크레이트에 정의합니다. 이 크레이트는 의존성을 최소화하여 다른 부분과 독립적으로 컴파일될 수 있어야 합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/protocol/src/lib.rs`: 모듈을 정의하는 메인 파일.
- `pipeline.rs`: `Pipeline`, `ProcessStep` 구조체 정의.
- `agent.rs`: `Agent` 구조체 및 관련 타입 정의.
- `process.rs`: `Process`, `ProcessStatus` 열거형 정의.
- `op.rs`, `event.rs`: TUI와 Core 간의 통신을 위한 `Op`, `Event` 열거형 정의.

## Interfaces

- 모든 구조체는 `Serialize`, `Deserialize`, `Debug`, `Clone`을 derive해야 합니다.
- TypeScript 연동을 위해 `ts_rs::TS`를 derive해야 합니다.
- `ProcessStatus`는 `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`를 사용하여 `HUMAN_REVIEW`와 같은 형식을 유지합니다.

## Reference Code

- **Op/Event 패턴**: `codex-rs/protocol/src/protocol.rs`의 `Op`와 `Event` 열거형 구조를 참고하세요.
- **데이터 구조체**: `codex-rs/protocol/src/config_types.rs` 와 `codex-rs/protocol/src/models.rs`의 `serde` 및 `ts_rs` 사용법을 참고하세요.

## Guidelines & Conventions

- YAML 파일의 `process` 리스트는 `Vec<ProcessStep>`으로 매핑됩니다. `ProcessStep`은 `String` 또는 `enum`으로 정의하여 `HUMAN_REVIEW` 같은 특수 명령어를 처리할 수 있도록 합니다.

## Acceptance Tests (TDD Process)

1.  **RED**: `pk-protocol` 크레이트에 테스트 파일을 만들고, 아직 존재하지 않는 `Pipeline`, `Agent` 등의 구조체를 `serde_json`으로 직렬화/역직렬화하려는 테스트 코드를 작성하여 컴파일 에러를 발생시킵니다.
2.  **GREEN**: `pipeline.rs`, `agent.rs` 등에 필요한 구조체와 열거형을 정의하고, `serde` 및 `ts_rs` derive를 추가하여 테스트가 통과하도록 합니다.
3.  **REFACTOR**: 각 구조체와 필드에 명확한 문서 주석(doc comments)을 추가하고, 모듈 구조를 논리적으로 정리합니다.
