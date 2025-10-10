# Pipeline-Kit 개선 필요 사항 보고서

**작성일**: 2025-10-11
**검증 대상**: Ticket 3.1, 4.1, 4.2, 4.3, 4.4, 5.1 (총 6개)
**검증 방법**: ticket-checker 에이전트를 통한 병렬 검증

---

## 📊 전체 요약

| 티켓 | 제목 | 판정 | 이슈 수 |
|------|------|------|---------|
| 3.1 | 파이프라인 실행 엔진 및 상태 관리 | ⚠️ 조건부 통과 | 3 |
| 4.1 | TUI 애플리케이션 셸 및 이벤트 루프 | ❌ FAIL | 1 |
| 4.2 | 대시보드 위젯 구현 | ❌ FAIL | 1 |
| 4.3 | 프로세스 상세 뷰 위젯 | ✅ PASS | 0 |
| 4.4 | 슬래시 커맨드 컴포저 위젯 | ✅ PASS | 0 |
| 5.1 | TypeScript 래퍼 및 npm 패키징 | ⚠️ 조건부 통과 | 3 |

**총 발견 이슈**: 8개
**치명적 이슈 (즉시 수정 필요)**: 2개
**중요 이슈**: 4개
**경미한 이슈**: 2개

---

## 🚨 치명적 이슈 (Critical - 즉시 수정 필요)

### 1. [Ticket 4.1] TUI 진입점 누락
**우선순위**: 🔴 Critical
**영향도**: 애플리케이션 실행 불가

**문제**:
- 스펙에서 명시한 `pipeline-kit-rs/crates/tui/src/main.rs` 파일이 존재하지 않음
- 모든 인프라는 완벽하게 구현되었으나 진입점이 없어 실행 불가능
- `App::run()` 함수를 호출할 방법이 없음

**위치**: `pipeline-kit-rs/crates/tui/src/main.rs` (미존재)

**해결 방법**:
```rust
// pipeline-kit-rs/crates/tui/src/main.rs
use anyhow::Result;
use pk_tui::App;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new().await?;
    app.run().await
}
```

**예상 소요 시간**: 30-60분 (main.rs 작성 + Cargo.toml 업데이트)

**상세 보고서**: `docs/spec/ticket-4.1/check.md`

---

### 2. [Ticket 4.2] 대시보드 위젯 미통합
**우선순위**: 🔴 Critical
**영향도**: 핵심 기능 미작동

**문제**:
- `widgets/dashboard.rs`에 Table 기반 대시보드 위젯이 완벽하게 구현됨
- 하지만 `app.rs`에서 이 위젯을 사용하지 않고 여전히 `Paragraph` 위젯 사용
- `render_dashboard()` 함수가 호출되지 않음
- IOI 점수 기준으로 0점 (함수 작성했지만 호출 안 함)

**위치**:
- 구현: `pipeline-kit-rs/crates/tui/src/widgets/dashboard.rs` ✅
- 미사용: `pipeline-kit-rs/crates/tui/src/app.rs:289-306`

**해결 방법**:
```rust
// app.rs에서
use crate::widgets::dashboard::render_dashboard;

// render() 함수 내부에서
let dashboard = render_dashboard(
    &self.processes,
    self.selected_process,
    dashboard_area,
);
f.render_widget(dashboard, dashboard_area);
```

**예상 소요 시간**: 1-2시간 (통합 + 테스트 + 스타일 조정)

**상세 보고서**: `docs/spec/ticket-4.2/check.md`

---

## ⚠️ 중요 이슈 (High Priority)

### 3. [Ticket 3.1] start_pipeline이 잘못된 UUID 반환
**우선순위**: 🟠 High
**영향도**: 프로세스 추적 불가

**문제**:
- `StateManager::start_pipeline()`이 실제 프로세스 ID 대신 placeholder UUID를 반환
- UI에서 프로세스를 추적, 조회, 제어할 수 없음
- TODO 주석으로 표시되어 있음

**위치**: `pipeline-kit-rs/crates/core/src/state/manager.rs:69-100`

**현재 코드**:
```rust
pub async fn start_pipeline(&self, config: PipelineConfig) -> Result<Uuid> {
    // TODO: actually spawn and track it, return real process_id
    Ok(Uuid::new_v4())
}
```

**해결 방법**:
- `PipelineEngine`을 실제로 생성하여 실행
- 생성된 프로세스를 `self.processes`에 추가
- 실제 프로세스 ID 반환

**예상 소요 시간**: 2-3시간

**상세 보고서**: `docs/spec/ticket-3.1/check.md`

---

### 4. [Ticket 3.1] resume_process_by_id가 실행 태스크를 재시작하지 않음
**우선순위**: 🟠 High
**영향도**: HUMAN_REVIEW에서 재개 불가

**문제**:
- `resume_process_by_id()`가 상태만 변경하고 실제 실행 태스크를 재시작하지 않음
- HUMAN_REVIEW 상태에서 멈춘 파이프라인을 재개할 수 없음

**위치**: `pipeline-kit-rs/crates/core/src/state/manager.rs:143-154`

**해결 방법**:
- 실행 태스크를 새로 spawn하여 파이프라인 재시작
- Event::ProcessResumed 발생

**예상 소요 시간**: 2-3시간

**상세 보고서**: `docs/spec/ticket-3.1/check.md`

---

### 5. [Ticket 5.1] GitHub Release 다운로드 미구현
**우선순위**: 🟠 High
**영향도**: 프로덕션 배포 불가

**문제**:
- `install_native_deps.sh`가 로컬 빌드에서만 복사
- npm으로 배포 시 GitHub Release에서 바이너리를 다운로드해야 하는데 미구현
- TODO 주석으로 표시되어 있음

**위치**: `pipeline-kit-cli/scripts/install_native_deps.sh:13-17`

**현재 코드**:
```bash
# TODO: production should fetch prebuilt binaries from GitHub Releases
echo "Installing from local build (development mode)"
cp -R "$RUST_BIN" "$VENDOR_DIR/"
```

**해결 방법**:
- GitHub Release API를 통해 플랫폼별 바이너리 다운로드
- 체크섬 검증 추가
- 개발 모드와 프로덕션 모드 분리

**예상 소요 시간**: 3-4시간

**상세 보고서**: `docs/spec/ticket-5.1/check.md`

---

### 6. [Ticket 5.1] TDD 프로세스 미준수
**우선순위**: 🟠 High
**영향도**: 품질 보증 부족

**문제**:
- 스펙에서 RED/GREEN/REFACTOR 사이클을 명시했으나 미준수
- Acceptance test가 0개
- 테스트 커버리지 0%

**위치**: `pipeline-kit-cli/` 전체

**해결 방법**:
- npm install 테스트 추가
- 바이너리 실행 테스트 추가
- 플랫폼별 검증 테스트 추가

**예상 소요 시간**: 2-3시간

**상세 보고서**: `docs/spec/ticket-5.1/check.md`

---

## 🟡 경미한 이슈 (Medium Priority)

### 7. [Ticket 3.1] kill_process가 tokio 태스크를 취소하지 않음
**우선순위**: 🟡 Medium
**영향도**: 리소스 누수 가능성

**문제**:
- `kill_process()`가 상태만 변경하고 실제 tokio 태스크를 취소하지 않음
- 백그라운드에서 계속 실행되어 리소스 낭비

**위치**: `pipeline-kit-rs/crates/core/src/state/manager.rs:167-176`

**해결 방법**:
- `tokio::task::JoinHandle`을 저장
- `handle.abort()` 호출하여 태스크 종료

**예상 소요 시간**: 1-2시간

**상세 보고서**: `docs/spec/ticket-3.1/check.md`

---

### 8. [Ticket 5.1] 디렉터리 구조 불일치
**우선순위**: 🟡 Medium
**영향도**: 사용자 경험 저하

**문제**:
- 스펙: `vendor/macos-x64/pipeline-kit`
- 실제: `vendor/aarch64-apple-darwin/pipeline-kit/pipeline`
- Rust target triple 대신 사용자 친화적 이름 권장

**위치**: `pipeline-kit-cli/scripts/install_native_deps.sh`

**해결 방법**:
- 플랫폼 이름을 `macos-arm64`, `linux-x64` 등으로 변경
- 또는 현재 구조를 문서화하여 명확히 함

**예상 소요 시간**: 1시간

**상세 보고서**: `docs/spec/ticket-5.1/check.md`

---

## 📋 작업 우선순위 제안

### Phase 1: 즉시 수정 (1-2일)
1. **Ticket 4.1** - TUI main.rs 추가 (0.5일)
2. **Ticket 4.2** - 대시보드 위젯 통합 (0.5일)

→ **목표**: 애플리케이션을 실행 가능하고 기본 기능 작동하게 만들기

### Phase 2: 핵심 기능 완성 (2-3일)
3. **Ticket 3.1 이슈 #3** - start_pipeline UUID 수정 (0.5일)
4. **Ticket 3.1 이슈 #4** - resume 로직 수정 (0.5일)
5. **Ticket 5.1 이슈 #5** - GitHub Release 다운로드 (1일)

→ **목표**: 모든 핵심 기능이 정상 작동하도록 수정

### Phase 3: 품질 개선 (1-2일)
6. **Ticket 5.1 이슈 #6** - TDD 테스트 추가 (0.5일)
7. **Ticket 3.1 이슈 #7** - kill 로직 수정 (0.5일)
8. **Ticket 5.1 이슈 #8** - 디렉터리 구조 정리 (0.5일)

→ **목표**: 코드 품질 향상 및 리소스 관리 개선

**총 예상 소요 시간**: 5-7일

---

## 📊 테스트 현황

### 통과한 테스트
- ✅ Ticket 1.1-1.2: 기본 구조 (100% 통과)
- ✅ Ticket 2.1-2.2: 설정 로딩 및 에이전트 (100% 통과)
- ✅ Ticket 3.1: 파이프라인 엔진 (97개 테스트 통과)
- ✅ Ticket 4.1: TUI 인프라 (43개 테스트 통과)
- ✅ Ticket 4.2: 대시보드 위젯 (3개 테스트 통과)
- ✅ Ticket 4.3: 상세 뷰 위젯 (11개 테스트 통과)
- ✅ Ticket 4.4: 커맨드 컴포저 (20개 테스트 통과)

**전체 테스트**: 174개 통과 / 0개 실패 (100%)

### 주의사항
- 모든 **유닛 테스트**는 통과했지만 **통합/실행 레벨**에서 이슈 발견
- 특히 Ticket 4.1, 4.2는 개별 컴포넌트는 완벽하나 통합이 누락됨

---

## 🎯 최종 권고사항

### 즉시 조치
1. **Ticket 4.1, 4.2를 최우선으로 수정** - 애플리케이션이 현재 실행 불가능
2. Phase 1 완료 후 전체 통합 테스트 실시

### 중기 조치
3. Ticket 3.1의 StateManager 로직 완성
4. Ticket 5.1의 프로덕션 배포 준비

### 장기 개선
5. E2E 테스트 추가
6. 문서화 보완
7. 성능 최적화

---

## 📁 관련 문서

- **전체 티켓 진행 상황**: `docs/ticket-progress.md`
- **개별 검증 보고서**:
  - `docs/spec/ticket-3.1/check.md`
  - `docs/spec/ticket-4.1/check.md`
  - `docs/spec/ticket-4.2/check.md`
  - `docs/spec/ticket-4.3/check.md`
  - `docs/spec/ticket-4.4/check.md`
  - `docs/spec/ticket-5.1/check.md`

---

**검증 완료일**: 2025-10-11
**다음 검증 예정일**: Phase 1 수정 완료 후
