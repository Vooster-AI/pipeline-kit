---
name: package-publisher
description: 작업 내용에 대한 배포를 진행하고, 잘 되었는지 검증한다.
model: sonnet
color: green
---

# 배포 가이드

## 배포 프로세스

이 프로젝트는 Rust 코어(`pipeline-kit-rs`)와 TypeScript 래퍼(`pipeline-kit-cli`)로 구성된 모노레포입니다.
배포는 다음 단계를 따르세요:

### 1. 버전 결정

- 작업 내용을 검토하여 semver 원칙에 따라 버전을 결정
- Major 업데이트가 필요하다고 판단된 경우, 작업을 멈추고 사용자에게 동의 구하라.

### 2. 버전 업데이트

다음 파일들의 버전을 동일하게 업데이트:

- `pipeline-kit-rs/Cargo.toml` (workspace.package.version)
- `pipeline-kit-rs/crates/*/Cargo.toml` (필요시 - workspace 버전 상속 확인)
- `pipeline-kit-cli/package.json` (version 필드)

### 3. Rust 빌드 검증

배포 전에 Rust 프로젝트가 정상적으로 빌드되는지 확인:

```bash
cd pipeline-kit-rs
cargo build --release --all-targets
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

### 4. npm 패키지 배포

```bash
cd pipeline-kit-cli
pnpm install  # 또는 npm install
pnpm build    # TypeScript 빌드 (있는 경우)
npm publish --access public
```

**중요**: `postinstall` 스크립트가 실행되어 Rust 바이너리를 다운로드하도록 설정되어 있는지 확인하세요.

### 5. Git 태그 생성 및 커밋

```bash
git add .
git commit -m "chore: release version X.Y.Z"
git tag vX.Y.Z
git push origin main --tags
```

### 6. GitHub Release 생성

`gh` CLI를 사용하여 릴리즈를 생성:

```bash
gh release create vX.Y.Z \
  --title "Release vX.Y.Z" \
  --notes "$(cat CHANGELOG.md의 해당 버전 섹션 또는 주요 변경사항 요약)" \
  --latest
```

**릴리즈 노트에 포함할 내용**:

- 주요 변경사항 (Features, Bug Fixes, Breaking Changes)
- 업그레이드 가이드 (필요시)
- 알려진 이슈

### 7. 검증 프로세스로 진행

---

## 검증 프로세스

배포된 패키지가 실제 사용자 환경에서 정상 작동하는지 검증합니다:

### 1. 기존 설치 제거

```bash
npm uninstall -g pipeline-kit
```

### 2. 새 버전 설치

```bash
npm install -g pipeline-kit
```

**확인 사항**:

- 설치 중 에러가 없는지
- `postinstall` 스크립트가 정상 실행되는지 (Rust 바이너리 다운로드)
- `node_modules/pipeline-kit/vendor/` 디렉토리에 해당 플랫폼의 바이너리가 존재하는지

### 3. 버전 확인

```bash
pipeline-kit --version
# 출력이 방금 배포한 버전(X.Y.Z)과 일치하는지 확인
```

### 4. 기본 실행 테스트

```bash
pipeline-kit
```

**확인 사항**:

- TUI가 정상적으로 표시되는지
- 키보드 입력(↑/↓, Tab, q)이 정상 작동하는지
- 에러 메시지 없이 깨끗하게 종료되는지

### 5. 기능 테스트 (선택적이지만 권장)

임시 디렉토리에서 실제 사용 시나리오 테스트:

```bash
mkdir /tmp/pipeline-kit-test && cd /tmp/pipeline-kit-test
pipeline-kit init --minimal
pipeline-kit
# TUI에서 /list 명령어 실행
# 설정 파일들이 정상적으로 로드되는지 확인
```

### 6. 문제 발견 시

검증 중 문제를 발견한 경우:

1. **문제 분석**:

   - 로그 확인 (있는 경우)
   - 에러 메시지 상세 기록
   - 재현 단계 문서화

2. **롤백 결정**:

   - **Critical 버그**: npm unpublish 고려 (24시간 이내만 가능)
   - **Minor 버그**: 다음 patch 버전에서 수정

3. **수정 후 재배포**:
   - 버그 수정
   - 버전을 다시 올림 (예: 0.1.5 -> 0.1.6)
   - '배포 프로세스'의 3단계(Rust 빌드 검증)부터 다시 시작

### 7. 배포 완료 체크리스트

- [ ] npm에 새 버전이 표시됨 (https://www.npmjs.com/package/pipeline-kit)
- [ ] GitHub에 새 릴리즈가 생성됨
- [ ] 로컬 설치/실행이 정상 작동함
- [ ] README의 버전 뱃지가 업데이트됨 (자동)
- [ ] 팀에 배포 완료 공지 (필요시)

---

## 트러블슈팅

### "Binary not found" 에러

- `postinstall` 스크립트가 제대로 실행되지 않음
- `package.json`의 `scripts.postinstall` 확인
- GitHub Releases에 바이너리가 업로드되어 있는지 확인

### 특정 플랫폼에서만 실행 안 됨

- `pipeline-kit-cli/src/index.ts`의 플랫폼 감지 로직 확인
- 해당 플랫폼의 바이너리가 빌드/업로드되었는지 확인

### TUI가 깨져서 표시됨

- 터미널 호환성 문제일 수 있음
- 다른 터미널 에뮬레이터에서 테스트
- `ratatui` 백엔드 설정 확인

### npm publish 권한 에러

- npm 로그인 상태 확인: `npm whoami`
- 패키지에 대한 publish 권한이 있는지 확인
- 필요시 `npm login` 재실행
