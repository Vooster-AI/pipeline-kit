# Ticket 5.1-Fix.3: 디렉터리 구조 정리

## Goal

`install_native_deps.sh`에서 Rust의 target triple(`aarch64-apple-darwin`)을 사용자 친화적인 이름(`macos-arm64`)으로 매핑하는 로직을 추가하여 `vendor/` 디렉터리 구조를 단순화합니다.

## Core Modules & Roles

- `pipeline-kit-cli/scripts/install_native_deps.sh`: 바이너리 설치 스크립트
- `pipeline-kit-cli/bin/pipeline-kit.js`: 플랫폼 감지 및 바이너리 경로 해석

## Reference Code

- `codex-cli/bin/codex.js`: 플랫폼 감지 로직 참고
  - `darwin-x64` → `macos-x64`
  - `darwin-arm64` → `macos-arm64`
  - `linux-x64` → `linux-x64`
  - `win32-x64` → `windows-x64`

## Detailed Implementation Steps

1. **플랫폼 매핑 함수 작성** (`install_native_deps.sh`):
   ```bash
   get_platform_name() {
       local os=$(uname -s)
       local arch=$(uname -m)

       case "$os-$arch" in
           Darwin-x86_64) echo "macos-x64" ;;
           Darwin-arm64) echo "macos-arm64" ;;
           Linux-x86_64) echo "linux-x64" ;;
           Linux-aarch64) echo "linux-arm64" ;;
           MINGW*-x86_64) echo "windows-x64" ;;
           *) echo "unsupported" ;;
       esac
   }
   ```

2. **`install_native_deps.sh` 수정**:
   - Rust target triple 대신 플랫폼 이름 사용
   - `vendor/<platform-name>/pipeline-kit` 구조로 변경
   - 중간 디렉터리 제거

3. **`pipeline-kit.js` 수정**:
   - 플랫폼 감지 로직을 매핑 함수와 일치시킴
   - 경로 생성 로직 업데이트
   ```javascript
   function getPlatformBinaryPath() {
       const platform = `${process.platform}-${process.arch}`;
       const platformMap = {
           'darwin-x64': 'macos-x64',
           'darwin-arm64': 'macos-arm64',
           'linux-x64': 'linux-x64',
           'linux-arm64': 'linux-arm64',
           'win32-x64': 'windows-x64'
       };
       return path.join(__dirname, '..', 'vendor', platformMap[platform], 'pipeline-kit');
   }
   ```

4. **문서 업데이트**:
   - README에 새로운 디렉터리 구조 명시
   - 개발자 가이드 업데이트

## Acceptance Tests (TDD Process)

1. **RED**:
   - 사용자 친화적 이름으로 바이너리 경로 조회 테스트 작성
   - 현재 구조(Rust triple)로는 테스트 실패

2. **GREEN**:
   - 위 구현 단계 수행
   - 모든 플랫폼에서 올바른 경로 생성 확인
   - 바이너리 실행 가능 확인

3. **REFACTOR**:
   - 플랫폼 매핑을 별도 모듈로 분리 고려
   - 에러 메시지에 지원되는 플랫폼 목록 표시
   - CI/CD 스크립트 업데이트
   - 기존 설치 마이그레이션 가이드 작성 (필요시)
