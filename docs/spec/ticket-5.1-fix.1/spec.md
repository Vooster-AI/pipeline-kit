# Ticket 5.1-Fix.1: GitHub Release에서 바이너리 다운로드 기능 구현

## Goal

`pipeline-kit-cli/scripts/install_native_deps.sh` 스크립트가 프로덕션 환경(`npm install` 시)에서 GitHub Release에 업로드된 플랫폼별 바이너리를 다운로드하고 압축을 해제하여 `vendor/` 디렉터리에 위치시키도록 수정합니다.

## Reference Code

- `codex-cli/scripts/install_native_deps.sh`: 이 스크립트는 `gh release download` 명령어를 사용하여 GitHub Actions 아티팩트를 다운로드하는 로직을 포함하고 있습니다. 이 로직을 GitHub Release 에셋을 다운로드하도록 수정해야 합니다.

## Detailed Implementation Steps

1. **스크립트 수정**: `install_native_deps.sh` 파일을 수정합니다.

2. 환경 변수(예: `NODE_ENV=production`)나 스크립트 인자를 사용하여 개발 모드와 프로덕션 모드를 구분하는 로직을 추가합니다.

3. 프로덕션 모드에서는 `gh release download <tag> -p 'pipeline-kit-*.tar.gz'` 명령어를 사용하여 최신 릴리즈에서 모든 플랫폼의 바이너리 아카이브를 다운로드합니다.

4. 다운로드된 각 `tar.gz` 파일의 압축을 해제하고, 내부 바이너리를 `vendor/<platform-name>/` 디렉터리로 이동시킵니다.

5. (선택) `sha256sum` 등을 사용하여 다운로드된 파일의 체크섬을 검증하는 단계를 추가합니다.

## Acceptance Tests (TDD Process)

1. **RED**: `pipeline-kit-cli`에서 로컬 테스트 스크립트를 작성합니다. 이 스크립트는 `install_native_deps.sh`를 프로덕션 모드로 실행하고, `vendor/` 디렉터리가 비어 있거나 로컬 파일만 복사되어 있는지 확인하여 테스트를 실패시킵니다.

2. **GREEN**: `gh` CLI와 `tar`를 사용하는 다운로드 및 압축 해제 로직을 구현합니다. 테스트에서는 mock GitHub Release 환경을 설정하거나, 실제 테스트용 릴리즈를 사용하여 스크립트가 올바른 파일을 다운로드하고 배치하는지 확인합니다.

3. **REFACTOR**: 플랫폼 이름(예: `macos-arm64`)을 결정하는 로직을 헬퍼 함수로 분리하고, 오류 발생 시 명확한 메시지를 출력하도록 개선합니다.
