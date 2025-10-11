# Ticket 9.2: 설치 스크립트 재작성 - Node.js 기반으로 전환

## Goal
플랫폼 의존적이고 외부 도구(`gh`)에 의존하는 `install_native_deps.sh` 셸 스크립트를 제거합니다. 모든 플랫폼에서 안정적으로 작동하는 순수 Node.js 기반의 설치 스크립트(`scripts/install.js`)로 대체하여 사용자 경험을 개선하고 유지보수성을 높입니다.

## Core Modules & Roles

-   `pipeline-kit-cli/scripts/install.js` (신규):
    -   모든 설치 로직을 담당합니다. 플랫폼 탐지, GitHub Release URL 구성, 바이너리 다운로드, 압축 해제를 수행합니다.
-   `pipeline-kit-cli/package.json` (수정):
    -   `"postinstall"` 스크립트를 `"node scripts/install.js"`로 변경합니다.
    -   `axios`, `tar` 등의 라이브러리를 `devDependencies`에 추가합니다.
-   `pipeline-kit-cli/scripts/install_native_deps.sh` (삭제):
    -   기존 셸 스크립트는 완전히 제거됩니다.

## Interfaces
이 작업은 주로 내부 스크립트를 변경하므로 외부 공개 인터페이스는 없습니다. `install.js`는 독립적으로 실행 가능한 스크립트가 됩니다.

## Guidelines & Conventions

-   플랫폼 및 아키텍처 탐지에는 `os.platform()`과 `os.arch()`를 사용합니다.
-   바이너리 다운로드에는 `axios` 또는 `node-fetch`를 사용합니다. `gh` CLI 의존성을 완전히 제거합니다.
-   `.tar.gz` 파일 압축 해제에는 `tar` npm 라이브러리를 사용합니다.
-   플랫폼 탐지 로직은 `lib/platform.js`에서 가져와 재사용하여 코드 중복을 방지합니다.

## Acceptance Tests (TDD Process)

### 1. RED:
-   `vitest`와 같은 테스트 프레임워크를 `pipeline-kit-cli`에 설정합니다.
-   `tests/install.test.js` 파일을 생성합니다.
-   `nock` 라이브러리를 사용하여 GitHub API 및 파일 다운로드 요청을 모킹(mocking)합니다.
-   `mock-fs` 라이브러리를 사용하여 가상 파일 시스템을 설정합니다.
-   `install.js` 스크립트를 실행했을 때, 특정 GitHub URL에서 파일을 다운로드하고, 지정된 `vendor` 디렉터리에 압축을 해제하며, 최종적으로 실행 파일이 올바른 위치에 존재하는지 검증하는 테스트를 작성합니다. 스크립트가 아직 없으므로 테스트는 실패합니다.

### 2. GREEN:
-   `install.js` 스크립트를 구현하여 RED 단계의 테스트를 통과시킵니다.
-   `package.json`의 `postinstall` 스크립트를 변경합니다.
-   로컬 환경에서 `npm install`을 실행하여 스크립트가 실제로 로컬 빌드 바이너리를 올바르게 복사하는지 수동으로 확인합니다.

### 3. REFACTOR:
-   다운로드 진행률 표시(progress bar) 기능을 추가하여 사용자 경험을 개선합니다.
-   오류 처리 로직을 강화합니다(예: 네트워크 오류, 압축 해제 실패 시 명확한 메시지 출력).
-   스크립트의 주요 로직(플랫폼 결정, URL 생성, 다운로드, 압축 해제)을 명확한 함수로 분리하여 가독성을 높입니다.
-   `install_native_deps.sh` 파일을 삭제합니다.

## Expected Outcomes

- 모든 플랫폼(Windows, macOS, Linux)에서 일관되게 작동하는 설치 프로세스를 확보합니다.
- `gh` CLI 의존성을 제거하여 사용자 설치 과정을 단순화합니다.
- 순수 JavaScript로 작성되어 유지보수와 디버깅이 용이합니다.
- 다운로드 진행률 표시로 사용자 경험이 개선됩니다.

## Related Files

- `pipeline-kit-cli/scripts/install.js` (신규)
- `pipeline-kit-cli/tests/install.test.js` (신규)
- `pipeline-kit-cli/package.json`
- `pipeline-kit-cli/lib/platform.js`
- `pipeline-kit-cli/scripts/install_native_deps.sh` (삭제 예정)

## Dependencies

다음 npm 패키지들을 `devDependencies`에 추가해야 합니다:
- `axios` 또는 `node-fetch`: HTTP 요청
- `tar`: 압축 해제
- `vitest`: 테스트 프레임워크
- `nock`: HTTP 모킹
- `mock-fs`: 파일 시스템 모킹
