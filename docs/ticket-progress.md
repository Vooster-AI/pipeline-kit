# Ticket Progress

## Phase 1: 프로젝트 기반 및 핵심 데이터 모델 정의

### Ticket 1.1: 모노레포 및 Cargo 워크스페이스 구조 설정
- [x] RED: `cargo check --workspace` 실패 확인
- [x] GREEN: 디렉터리 구조 및 `Cargo.toml` 워크스페이스 생성
- [x] REFACTOR: 파일 구조 및 멤버 명시 검증
- [x] **Ticket 1.1 완료**

### Ticket 1.2: 핵심 프로토콜 및 데이터 모델 정의 (`pk-protocol` 크레이트)
- [x] RED: 테스트 파일 작성 및 컴파일 에러 확인
- [x] GREEN: `Pipeline`, `Agent` 등 구조체 정의 및 derive 추가
- [x] REFACTOR: 문서 주석 추가 및 모듈 구조 정리
- [x] **Ticket 1.2 완료**

---

## Phase 2: 설정 로딩 및 에이전트 추상화

### Ticket 2.1: 설정 파일 로더 구현 (`pk-core` 크레이트)
- [x] RED: 임시 디렉터리 및 예제 설정 파일로 테스트 작성
- [x] GREEN: `serde_yaml`, `toml`, `gray_matter`로 파싱 로직 구현
- [x] REFACTOR: 오류 처리 개선 및 엣지 케이스 테스트 보강
- [x] **Ticket 2.1 완료**

### Ticket 2.2: 에이전트 어댑터 패턴 및 관리자 구현 (`pk-core` 크레이트)
- [x] RED: `AgentManager` 테스트 작성 및 컴파일 실패 확인
- [x] GREEN: `Agent` 트레이트, `MockAgent`, `AgentManager` 구현
- [x] REFACTOR: 트레이트 시그니처 명확화 및 에러 처리 개선
- [x] **Ticket 2.2 완료**

---

## Phase 3: 파이프라인 엔진 및 기본 CLI 기능 구현

### Ticket 3.1: 파이프라인 실행 엔진 및 상태 관리 구현 (`pk-core` 크레이트)
- [x] RED: 순차 파이프라인 테스트 작성 및 이벤트 검증
- [x] GREEN: `PipelineEngine` 및 `Process` 상태 머신 구현
- [x] REFACTOR: `Arc<Mutex<>>` 적용 및 `StateManager` 도입
- [x] **Ticket 3.1 완료**

---

## Phase 4: TUI 구현

### Ticket 4.1: TUI 애플리케이션 셸 및 이벤트 루프 구축 (`pk-tui` 크레이트)
- [x] RED: `App::run` 테스트 작성 및 빈 화면 확인
- [x] GREEN: `ratatui`로 기본 레이아웃 및 이벤트 루프 구현
- [x] REFACTOR: TUI 상태와 비즈니스 로직 분리
- [x] **Ticket 4.1 완료**

### Ticket 4.2: 대시보드 위젯 구현
- [ ] RED: 대시보드 위젯 테스트 작성
- [ ] GREEN: `Process` 테이블 형태 렌더링 구현
- [ ] REFACTOR: 위젯 코드 정리 및 최적화
- [ ] **Ticket 4.2 완료**

### Ticket 4.3: 프로세스 상세 뷰 위젯 구현
- [ ] RED: 상세 뷰 위젯 테스트 작성
- [ ] GREEN: 스크롤 가능한 로그 뷰 구현
- [ ] REFACTOR: 스크롤 성능 및 UI 개선
- [ ] **Ticket 4.3 완료**

### Ticket 4.4: 슬래시 커맨드 컴포저 위젯 구현
- [ ] RED: 커맨드 입력창 테스트 작성
- [ ] GREEN: 슬래시 커맨드 입력 및 자동 완성 구현
- [ ] REFACTOR: 커맨드 파싱 및 제안 로직 개선
- [ ] **Ticket 4.4 완료**

---

## Phase 5: 배포 준비

### Ticket 5.1: TypeScript 래퍼 및 npm 패키징 (`pipeline-kit-cli` 디렉터리)
- [ ] RED: `npm install` 및 바이너리 실행 실패 확인
- [ ] GREEN: `package.json` 및 `install_native_deps.sh` 구현
- [ ] REFACTOR: 경로 변수화 및 오류 메시지 개선
- [ ] **Ticket 5.1 완료**

---

## 전체 진행 상황

- **Phase 1**: 2/2 티켓 완료 ✅
- **Phase 2**: 2/2 티켓 완료 ✅
- **Phase 3**: 1/1 티켓 완료 ✅
- **Phase 4**: 1/4 티켓 완료
- **Phase 5**: 0/1 티켓 완료

**총 진행률**: 6/10 티켓 완료 (60%)
