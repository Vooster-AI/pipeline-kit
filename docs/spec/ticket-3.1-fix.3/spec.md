# Ticket 3.1-Fix.3: `kill_process` 태스크 취소 로직

## Goal

`StateManager`가 `Process`와 함께 저장한 `JoinHandle`에 대해 `.abort()`를 호출하여 tokio 태스크를 즉시 종료하고 리소스를 정리하도록 수정합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/core/src/state/manager.rs`: 프로세스 상태 관리 및 종료 로직
- `pipeline-kit-rs/crates/core/src/engine/mod.rs`: 파이프라인 엔진 실행 및 정리 로직

## Reference Code

- `tokio::task::JoinHandle` 문서: `.abort()` 메서드 사용법
- `tokio::task::JoinError` 문서: 취소된 태스크 에러 처리
- `codex-rs/core/src/codex.rs`: 백그라운드 태스크 관리 및 종료 패턴

## Detailed Implementation Steps

1. **`Process` 구조체 확장** (이미 Ticket 3.1-Fix.1에서 수행):
   - `tokio::task::JoinHandle` 필드 확인
   - `#[serde(skip)]` 어트리뷰트 확인

2. **`StateManager::kill_process` 수정**:
   - 프로세스 상태를 `Killed`로 변경
   - 저장된 `JoinHandle`에 `.abort()` 호출
   - `Event::ProcessKilled` 발생
   - 프로세스를 내부 맵에서 제거 (또는 완료된 프로세스로 이동)

3. **`PipelineEngine` 정리 로직**:
   - 취소 시그널 감지 시 리소스 정리
   - 진행 중이던 작업 안전하게 중단
   - 임시 파일 등 정리

## Acceptance Tests (TDD Process)

1. **RED**:
   - 실행 중인 프로세스 생성
   - `kill_process` 호출
   - 백그라운드 태스크가 여전히 실행 중인지 확인 (테스트 실패)

2. **GREEN**:
   - 위 구현 단계 수행
   - `kill_process` 호출 후 태스크가 즉시 종료되는지 확인
   - `Event::ProcessKilled` 수신 확인
   - 메모리 누수가 없는지 확인

3. **REFACTOR**:
   - 리소스 정리 로직을 별도 함수로 분리
   - 에러 케이스 처리 (이미 종료된 프로세스 등)
   - 정리 타임아웃 처리 추가
   - 로깅 및 모니터링 개선
