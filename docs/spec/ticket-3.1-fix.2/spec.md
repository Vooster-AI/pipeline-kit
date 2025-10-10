# Ticket 3.1-Fix.2: `resume_process_by_id` 로직 수정

## Goal

`StateManager`에서 `PAUSED` 또는 `HUMAN_REVIEW` 상태의 프로세스를 찾아, 저장된 `PipelineEngine` 태스크에 `tokio::sync::Notify`를 통해 재개 신호를 보내도록 수정합니다.

## Core Modules & Roles

- `pipeline-kit-rs/crates/core/src/state/manager.rs`: 프로세스 상태 관리 및 재개 로직
- `pipeline-kit-rs/crates/core/src/engine/mod.rs`: 파이프라인 엔진 실행 로직

## Reference Code

- `tokio::sync::Notify` 문서: 태스크 간 신호 전달을 위한 비동기 동기화 프리미티브
- `codex-rs/core/src/codex.rs`: 비동기 태스크 관리 패턴 참고

## Detailed Implementation Steps

1. **`Process` 구조체 확장**:
   - `tokio::sync::Arc<Notify>`를 저장할 필드 추가
   - `#[serde(skip)]` 어트리뷰트 적용

2. **`PipelineEngine` 수정**:
   - `HUMAN_REVIEW` 상태에서 `Notify::notified().await`로 재개 대기
   - 재개 신호 수신 시 다음 단계로 진행

3. **`StateManager::resume_process_by_id` 수정**:
   - 프로세스 상태를 `Running`으로 변경
   - 저장된 `Notify`에 `notify_one()` 호출
   - `Event::ProcessResumed` 발생

## Acceptance Tests (TDD Process)

1. **RED**:
   - `HUMAN_REVIEW` 상태의 프로세스 생성
   - `resume_process_by_id` 호출
   - 프로세스가 실제로 재개되지 않는 것 확인 (테스트 실패)

2. **GREEN**:
   - 위 구현 단계 수행
   - 재개 후 파이프라인이 다음 단계를 실행하는지 확인
   - `Event::ProcessResumed` 수신 확인

3. **REFACTOR**:
   - 재개 로직을 명확히 문서화
   - 에러 케이스 처리 (이미 실행 중인 프로세스 등)
   - 타임아웃 처리 추가 고려
