# Release Guide

## 📦 릴리즈 준비

### 1. 버전 결정
- Major: 호환성 없는 API 변경 (1.0.0 → 2.0.0)
- Minor: 하위 호환 기능 추가 (1.0.0 → 1.1.0)
- Patch: 버그 수정 (1.0.0 → 1.0.1)
- Prerelease: 알파/베타 (1.0.0-alpha, 1.0.0-beta)

### 2. 사전 체크리스트

#### 자동 체크 (권장)
```bash
# 모든 자동화된 체크를 한 번에 실행
./scripts/pre-release-check.sh
```

이 스크립트는 다음 항목들을 자동으로 검증합니다:
- ✅ `cargo test --workspace` 모두 통과
- ✅ `cargo clippy --all-targets` 경고 없음
- ✅ `cargo build --release` 성공
- ✅ TypeScript 테스트 통과

#### 수동 확인 필수
- [ ] CHANGELOG.md 업데이트
- [ ] README.md 버전 정보 확인

<details>
<summary>수동으로 체크하려면 (클릭하여 펼치기)</summary>

```bash
# Rust 테스트
cd pipeline-kit-rs && cargo test --workspace

# Clippy 검사
cd pipeline-kit-rs && cargo clippy --all-targets -- -D warnings

# Release 빌드
cd pipeline-kit-rs && cargo build --release

# TypeScript 테스트
cd pipeline-kit-cli && npm test
```
</details>

### 3. 버전 업데이트
```bash
# Cargo.toml 버전 업데이트
cd pipeline-kit-rs
# Edit Cargo.toml: version = "1.0.0"

# package.json 버전 업데이트 (동일한 버전)
cd ../pipeline-kit-cli
npm version 1.0.0 --no-git-tag-version
```

### 4. 변경사항 커밋
```bash
git add -A
git commit -m "chore: bump version to 1.0.0"
git push origin main
```

## 🚀 릴리즈 실행

### 방법 1: GitHub UI (권장)
1. GitHub 저장소 → Releases → "Draft a new release"
2. Tag 입력: `v1.0.0` (새 태그 생성)
3. Target: `main` 브랜치
4. Title: `Pipeline Kit v1.0.0`
5. Description: CHANGELOG 내용 복사
6. "Publish release" 클릭
7. 워크플로우 자동 시작 (Actions 탭에서 확인)

### 방법 2: CLI (자동화)
```bash
# 태그 생성
git tag -a v1.0.0 -m "Release 1.0.0"

# 태그 푸시 (워크플로우 자동 시작)
git push origin v1.0.0
```

## 🔄 워크플로우 진행 상황

### 1. Tag Validation (1분)
- Tag 형식 검증 (v*.*.*)
- Cargo.toml 버전 일치 확인
- ✅ 통과 → Build 시작
- ❌ 실패 → 버전 불일치 수정 필요

### 2. Build (10-20분)
6개 플랫폼 병렬 빌드:
- macOS x64 (Intel)
- macOS ARM64 (Apple Silicon)
- Linux x64
- Linux ARM64
- Windows x64
- Windows ARM64

각 플랫폼별:
1. Rust 바이너리 빌드
2. tar.gz 압축
3. sha256 체크섬 생성
4. 아티팩트 업로드

### 3. Release (2분)
- 모든 아티팩트 다운로드
- GitHub Release 생성
- 12개 파일 업로드 (6 × tar.gz + sha256)

## 📊 릴리즈 확인

### 1. GitHub Release 확인
```bash
# Release 페이지 확인
# https://github.com/{org}/pipeline-kit/releases/tag/v1.0.0

# 예상 Assets (12개):
# - pipeline-kit-macos-x64.tar.gz (+ .sha256)
# - pipeline-kit-macos-arm64.tar.gz (+ .sha256)
# - pipeline-kit-linux-x64.tar.gz (+ .sha256)
# - pipeline-kit-linux-arm64.tar.gz (+ .sha256)
# - pipeline-kit-windows-x64.tar.gz (+ .sha256)
# - pipeline-kit-windows-arm64.tar.gz (+ .sha256)
```

### 2. 바이너리 다운로드 테스트
```bash
# 현재 플랫폼용 바이너리 다운로드
gh release download v1.0.0 \
  --repo {org}/pipeline-kit \
  --pattern "pipeline-kit-$(uname -s | tr '[:upper:]' '[:lower:]')-*.tar.gz"

# 압축 해제
tar -xzf pipeline-kit-*.tar.gz

# 실행 테스트
./pipeline-kit/pipeline --version
```

### 3. npm 설치 테스트
```bash
# GitHub Release 생성 완료 후
cd pipeline-kit-cli

# npm 배포 (수동)
npm publish --access public

# 설치 테스트
npm install -g pipeline-kit@1.0.0
pipeline-kit --version
```

## 🐛 문제 해결

### 버전 불일치 에러
```
❌ Tag 1.0.0 ≠ Cargo.toml 0.1.0
```

**해결**:
1. Cargo.toml 버전 수정
2. 커밋 & 푸시
3. 태그 삭제 후 재생성
```bash
git tag -d v1.0.0
git push --delete origin v1.0.0
git tag -a v1.0.0 -m "Release 1.0.0"
git push origin v1.0.0
```

### 빌드 실패
**Linux musl 에러**:
```bash
# musl-tools 설치 확인
sudo apt install -y musl-tools
```

**ARM64 빌드 실패**:
- GitHub Actions runner 제한 확인
- ubuntu-24.04-arm 사용 가능 여부 확인

### 체크섬 검증 실패
```bash
# 로컬에서 체크섬 확인
sha256sum pipeline-kit-*.tar.gz
cat pipeline-kit-*.tar.gz.sha256

# 재생성
sha256sum pipeline-kit-*.tar.gz > pipeline-kit-*.tar.gz.sha256
```

## 📝 릴리즈 후 작업

### 1. npm 배포
```bash
cd pipeline-kit-cli

# 버전 확인
npm version  # Should match git tag

# 배포
npm publish --access public

# 확인
npm view pipeline-kit version
```

### 2. 문서 업데이트
- [ ] README.md의 설치 명령어 버전 업데이트
- [ ] CHANGELOG.md에 릴리즈 날짜 추가
- [ ] 공식 문서 사이트 업데이트 (있는 경우)

### 3. 커뮤니케이션
- [ ] GitHub Discussions에 릴리즈 노트 게시
- [ ] Discord/Slack 채널에 공지
- [ ] Twitter/SNS 발표

### 4. 다음 개발 주기 시작
```bash
# 다음 버전으로 Cargo.toml 업데이트
# version = "1.1.0-dev"

git add Cargo.toml
git commit -m "chore: start 1.1.0 development cycle"
git push origin main
```

## 🔄 Hotfix 릴리즈

긴급 버그 수정 시:

```bash
# main에서 hotfix 브랜치 생성
git checkout -b hotfix/1.0.1

# 버그 수정
# ... fix code ...

# 버전 업데이트
# Cargo.toml: version = "1.0.1"

# 커밋
git commit -am "fix: critical bug in pipeline execution"

# main에 병합
git checkout main
git merge hotfix/1.0.1
git push origin main

# 릴리즈
git tag -a v1.0.1 -m "Hotfix 1.0.1"
git push origin v1.0.1

# hotfix 브랜치 삭제
git branch -d hotfix/1.0.1
```

## 📊 릴리즈 체크리스트

### 릴리즈 전
- [ ] 모든 테스트 통과
- [ ] Clippy 경고 없음
- [ ] CHANGELOG.md 업데이트
- [ ] 버전 번호 일치 (Cargo.toml, package.json)
- [ ] README.md 업데이트

### 릴리즈 중
- [ ] GitHub Release 생성됨
- [ ] 12개 Assets 모두 업로드됨
- [ ] 체크섬 파일 검증 통과

### 릴리즈 후
- [ ] npm 배포 완료
- [ ] 설치 테스트 성공
- [ ] 문서 업데이트
- [ ] 릴리즈 노트 게시

## 🔐 필수 Secrets

GitHub Repository Settings → Secrets and variables → Actions:

- `NPM_TOKEN`: npm publish용 (선택)
  ```bash
  # npm 토큰 생성
  npm login
  npm token create --read-only=false
  ```

## 🎯 자동화 팁

### 릴리즈 스크립트
```bash
#!/bin/bash
# scripts/release.sh

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Usage: ./release.sh 1.0.0"
  exit 1
fi

echo "🚀 Releasing v${VERSION}..."

# 1. 사전 체크 실행
echo "Running pre-release checks..."
./scripts/pre-release-check.sh

# 2. 버전 업데이트
sed -i "s/^version = .*/version = \"${VERSION}\"/" pipeline-kit-rs/Cargo.toml
npm --prefix pipeline-kit-cli version ${VERSION} --no-git-tag-version

# 3. 커밋
git add -A
git commit -m "chore: bump version to ${VERSION}"
git push origin main

# 4. 태그
git tag -a v${VERSION} -m "Release ${VERSION}"
git push origin v${VERSION}

echo "✅ Release v${VERSION} started. Check GitHub Actions."
```
