# Ticket 5.1-Fix.2: TDD 테스트 추가

## Goal

`pipeline-kit-cli`에 `jest`나 `vitest` 같은 테스트 프레임워크를 추가하고, `pipeline-kit.js`가 올바른 플랫폼 바이너리 경로를 생성하는지 검증하는 단위 테스트를 작성합니다.

## Core Modules & Roles

- `pipeline-kit-cli/bin/pipeline-kit.js`: 플랫폼 감지 및 바이너리 실행 로직
- `pipeline-kit-cli/package.json`: 테스트 프레임워크 설정
- `pipeline-kit-cli/test/`: 테스트 파일 디렉터리

## Reference Code

- `codex-cli/test/`: TypeScript CLI 테스트 패턴 참고
- Jest/Vitest 문서: Node.js 모듈 테스트 작성법

## Detailed Implementation Steps

1. **테스트 프레임워크 설정**:
   - `package.json`에 `vitest` 또는 `jest` 추가
   - 테스트 스크립트 설정
   ```json
   {
     "scripts": {
       "test": "vitest run",
       "test:watch": "vitest"
     },
     "devDependencies": {
       "vitest": "^1.0.0"
     }
   }
   ```

2. **플랫폼 감지 테스트 작성** (`test/platform.test.js`):
   - `process.platform`과 `process.arch` 모킹
   - 각 플랫폼별 올바른 경로 생성 확인
   - 지원하지 않는 플랫폼 에러 처리 확인

3. **바이너리 경로 테스트 작성** (`test/binary-path.test.js`):
   - 생성된 경로가 존재하는지 확인 (개발 모드)
   - 올바른 실행 권한이 있는지 확인
   - 경로 문자열 형식 검증

4. **통합 테스트 작성** (`test/integration.test.js`):
   - `--version` 플래그로 바이너리 실행
   - `--help` 플래그로 도움말 출력 확인
   - 에러 핸들링 검증

## Acceptance Tests (TDD Process)

1. **RED**:
   - 위 테스트들을 작성
   - 테스트 실패 확인 (아직 구현되지 않은 기능)

2. **GREEN**:
   - 각 테스트가 통과하도록 코드 수정
   - 최소한의 구현으로 테스트 통과
   - 모든 플랫폼 조합 커버

3. **REFACTOR**:
   - 테스트 중복 제거
   - 헬퍼 함수 추출
   - 테스트 가독성 개선
   - 테스트 커버리지 리포트 추가
