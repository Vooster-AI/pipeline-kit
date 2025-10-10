# Ticket 5.1: TypeScript 래퍼 및 npm 패키징 (`pipeline-kit-cli` 디렉터리)

## Goal

`pipeline-kit-cli` 디렉터리를 설정하고, `npm install` 시 Rust 바이너리를 다운로드하여 실행할 수 있는 `pipeline-kit.js` 런처와 `package.json`을 작성합니다.

## Reference Code

- `codex-cli/` 디렉터리의 모든 파일을 거의 그대로 복제하고, 'codex'를 'pipeline-kit'으로 변경하는 작업이 주가 됩니다.
- `codex-cli/bin/codex.js`와 `codex-cli/scripts/install_native_deps.sh`를 주의 깊게 분석하세요.

## Acceptance Tests (TDD Process)

1.  **RED**: `pipeline-kit-cli` 디렉터리에서 `npm install`을 실행하고, `bin/pipeline-kit` 스크립트를 실행했을 때, "바이너리를 찾을 수 없음" 에러가 발생하는 것을 확인합니다.
2.  **GREEN**: `package.json`의 `postinstall` 스크립트와 `install_native_deps.sh`를 구현하여 (임시로 로컬 빌드된 바이너리를 복사하도록 하여) `pipeline-kit` 명령어가 성공적으로 Rust 프로세스를 실행하는지 확인합니다.
3.  **REFACTOR**: 스크립트의 하드코딩된 경로를 변수화하고, 오류 메시지를 사용자 친화적으로 개선합니다.
