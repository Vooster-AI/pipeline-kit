# Ticket 2.2 Enhanced Implementation Plan: Real Agent Adapters

**문서 버전**: v2.0
**작성일**: 2025-10-11
**기반**: Python 레퍼런스 코드 분석 결과

---

## 목차

1. [개요](#1-개요)
2. [Python 레퍼런스 코드 분석](#2-python-레퍼런스-코드-분석)
3. [아키텍처 설계 개선](#3-아키텍처-설계-개선)
4. [ClaudeAdapter 상세 설계](#4-claudeadapter-상세-설계)
5. [CursorAdapter 상세 설계](#5-cursoradapter-상세-설계)
6. [GeminiAdapter 상세 설계](#6-geminiadapter-상세-설계)
7. [AgentFactory 설계](#7-agentfactory-설계)
8. [ExecutionContext 확장](#8-executioncontext-확장)
9. [세션 관리 전략](#9-세션-관리-전략)
10. [구현 우선순위 및 단계](#10-구현-우선순위-및-단계)
11. [테스트 전략](#11-테스트-전략)
12. [파일 구조](#12-파일-구조)
13. [의존성 및 환경 변수](#13-의존성-및-환경-변수)

---

## 1. 개요

Ticket 2.2의 MockAgent 구현을 기반으로, 실제 AI 에이전트(Claude, Cursor, Gemini)와 통신하는 어댑터를 구현합니다. Python 레퍼런스 코드를 분석하여 검증된 패턴을 Rust로 이식합니다.

### 핵심 원칙

1. **검증된 패턴 사용**: Python 레퍼런스의 통신 방식을 그대로 따름
2. **안정성 우선**: subprocess 기반 CLI 실행으로 안정성 확보
3. **TDD 유지**: 각 어댑터마다 단위 테스트 우선 작성
4. **점진적 구현**: Claude → Cursor → Gemini 순으로 구현

---

## 2. Python 레퍼런스 코드 분석

### 2.1 ClaudeCodeCLI 분석 결과

#### 통신 방식
- **방법**: Python SDK (`ClaudeSDKClient`) 사용
- **실체**: Claude Code CLI를 subprocess로 실행하는 래퍼
- **장점**: SDK가 세션 관리, 재시도 로직 등 처리

#### check_availability
```python
# "claude -h" 명령으로 CLI 설치 확인
result = await asyncio.create_subprocess_shell("claude -h", ...)
if result.returncode != 0:
    return {"available": False, ...}
```

#### execute_with_streaming 핵심 로직
```python
# 1. 시스템 프롬프트를 임시 settings.json에 저장
temp_settings = tempfile.NamedTemporaryFile(...)
json.dump({"customSystemPrompt": full_system_prompt}, temp_settings)

# 2. 도구 필터링 (is_initial_prompt에 따라)
if is_initial_prompt:
    disallowed_tools = ["TodoWrite"]  # 첫 프롬프트에서는 TodoWrite 제외
else:
    allowed_tools = ["Read", "Write", ..., "TodoWrite"]

# 3. SDK 옵션 설정
options = ClaudeCodeOptions(
    allowed_tools=allowed_tools,
    disallowed_tools=disallowed_tools,
    permission_mode="bypassPermissions",
    model=cli_model,
    continue_conversation=True,
    settings=session_settings_path,
    resumeSessionId=existing_session_id,  # 세션 재개
)

# 4. 메시지 스트리밍
async with ClaudeSDKClient(options=options) as client:
    await client.query(instruction)
    async for message_obj in client.receive_messages():
        # 메시지 타입별 처리
        if isinstance(message_obj, SystemMessage):
            # session_id 추출 및 저장
            claude_session_id = message_obj.session_id
        elif isinstance(message_obj, AssistantMessage):
            # content blocks 처리 (TextBlock, ToolUseBlock)
        elif isinstance(message_obj, ResultMessage):
            # 세션 완료
```

#### 메시지 타입별 처리

| 타입 | 역할 | UI 표시 | AgentEvent 변환 |
|------|------|---------|----------------|
| `SystemMessage` | 세션 초기화, session_id 제공 | ❌ (hidden_from_ui) | - |
| `AssistantMessage` | AI 응답 (텍스트 + 도구 호출) | ✅ | `MessageChunk`, `ToolCall` |
| `UserMessage` | 도구 결과 | ❌ | - |
| `ResultMessage` | 세션 완료 (cost, duration 등) | ❌ (hidden_from_ui) | `Completed` |

#### 세션 관리
```python
# project_id → session_id 매핑 저장
session_mapping: Dict[str, str] = {}

async def get_session_id(self, project_id: str) -> Optional[str]:
    return self.session_mapping.get(project_id)

async def set_session_id(self, project_id: str, session_id: str):
    self.session_mapping[project_id] = session_id
```

---

### 2.2 CursorAgentCLI 분석 결과

#### 통신 방식
- **방법**: `cursor-agent` CLI를 subprocess로 실행
- **출력 형식**: NDJSON (Newline-Delimited JSON)
- **명령 예시**: `cursor-agent --force -p "instruction" --output-format stream-json -m gpt-5`

#### check_availability
```python
result = await asyncio.create_subprocess_shell("cursor-agent -h", ...)
if result.returncode != 0:
    return {"available": False, ...}
```

#### execute_with_streaming 핵심 로직
```python
# 1. AGENTS.md 파일 생성 (시스템 프롬프트)
await self._ensure_agent_md(project_path)

# 2. 명령 구성
cmd = [
    "cursor-agent",
    "--force",
    "-p", instruction,
    "--output-format", "stream-json",
    "-m", cli_model,
]
if stored_session_id:
    cmd.extend(["--resume", stored_session_id])

# 3. 프로세스 실행 및 NDJSON 파싱
process = await asyncio.create_subprocess_exec(*cmd, stdout=PIPE, ...)
async for line in process.stdout:
    event = json.loads(line_str)  # NDJSON 파싱

    # 이벤트 타입별 처리
    if event["type"] == "system":
        # 초기화 (hidden_from_ui)
    elif event["type"] == "assistant":
        # 텍스트 델타
    elif event["type"] == "tool_call":
        if event["subtype"] == "started":
            # 도구 호출 시작
        elif event["subtype"] == "completed":
            # 도구 결과
    elif event["type"] == "result":
        # session_id 추출 및 저장
        session_id = event.get("session_id")
        await self.set_session_id(project_id, session_id)
        break  # 스트림 종료
```

#### AGENTS.md 생성
```python
async def _ensure_agent_md(self, project_path: str):
    """프로젝트 루트에 AGENTS.md 파일 생성 (시스템 프롬프트)"""
    agent_md_path = os.path.join(project_repo_path, "AGENTS.md")
    if not os.path.exists(agent_md_path):
        with open(system_prompt_path, "r") as f:
            system_prompt_content = f.read()
        with open(agent_md_path, "w") as f:
            f.write(system_prompt_content)
```

#### NDJSON 이벤트 타입

| type | subtype | 설명 | UI 표시 |
|------|---------|------|---------|
| `system` | - | 초기화 | ❌ |
| `user` | - | 에코백 (suppress) | ❌ |
| `assistant` | - | 텍스트 델타 | ✅ |
| `tool_call` | `started` | 도구 호출 시작 | ✅ |
| `tool_call` | `completed` | 도구 결과 | ❌ |
| `result` | - | 세션 완료 | ❌ |

---

### 2.3 GeminiCLI 분석 결과

Python 레퍼런스에는 GeminiCLI가 있지만, spec.md의 설명에서 유추:
- **통신 방식**: JSON-RPC via stdio
- **특징**: 양방향 통신 (stdin으로 요청, stdout으로 응답)

---

## 3. 아키텍처 설계 개선

### 3.1 기존 설계 문제점

1. **SDK 의존성**: Rust용 공식 Claude SDK 없음
2. **복잡도**: SDK 대신 CLI subprocess 방식이 더 안정적
3. **세션 관리 누락**: Python 레퍼런스의 session_mapping 패턴 미반영

### 3.2 개선된 설계 방향

1. **CLI Subprocess 전략**: 모든 어댑터가 CLI를 subprocess로 실행
2. **통일된 세션 관리**: `SessionManager` trait 도입
3. **ExecutionContext 확장**: `is_initial_prompt`, `project_path` 등 추가

---

## 4. ClaudeAdapter 상세 설계

### 4.1 구조

```rust
// crates/core/src/agents/adapters/claude_adapter.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tempfile::NamedTempFile;

pub struct ClaudeAdapter {
    name: String,
    model: String,
    system_prompt: String,
    /// project_id → session_id 매핑 (Arc<Mutex<>> for thread safety)
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
}

impl ClaudeAdapter {
    pub fn new(name: String, model: String, system_prompt: String) -> Result<Self> {
        Ok(Self {
            name,
            model,
            system_prompt,
            session_mapping: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// project_id에서 project_path 추출
    /// 예: "/path/to/project" -> "project"
    fn extract_project_id(project_path: &str) -> String {
        std::path::Path::new(project_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(project_path)
            .to_string()
    }

    /// 임시 settings.json 파일 생성
    fn create_settings_file(&self) -> Result<NamedTempFile, AgentError> {
        let settings = serde_json::json!({
            "customSystemPrompt": self.system_prompt
        });

        let mut temp_file = NamedTempFile::new()
            .map_err(|e| AgentError::ExecutionError(e.to_string()))?;

        serde_json::to_writer(&mut temp_file, &settings)
            .map_err(|e| AgentError::ExecutionError(e.to_string()))?;

        Ok(temp_file)
    }

    /// claude CLI 명령 구성
    fn build_command(
        &self,
        context: &ExecutionContext,
        settings_path: &str,
        session_id: Option<&str>,
    ) -> Command {
        let mut cmd = Command::new("claude");

        // 기본 옵션
        cmd.args(&[
            "--settings", settings_path,
            "--model", &self.model,
            "--permission-mode", "bypassPermissions",
            "--continue-conversation",
        ]);

        // 도구 필터링
        if context.is_initial_prompt {
            // 첫 프롬프트: TodoWrite 제외
            cmd.args(&[
                "--allowed-tools", "Read,Write,Edit,MultiEdit,Bash,Glob,Grep,LS,WebFetch,WebSearch",
                "--disallowed-tools", "TodoWrite",
            ]);
        } else {
            // 후속 프롬프트: TodoWrite 포함
            cmd.args(&[
                "--allowed-tools", "Read,Write,Edit,MultiEdit,Bash,Glob,Grep,LS,WebFetch,WebSearch,TodoWrite",
            ]);
        }

        // 세션 재개
        if let Some(sid) = session_id {
            cmd.args(&["--resume-session-id", sid]);
        }

        // 프롬프트
        cmd.args(&["--prompt", &context.instruction]);

        cmd
    }
}

#[async_trait]
impl Agent for ClaudeAdapter {
    async fn check_availability(&self) -> bool {
        // "claude -h" 명령으로 설치 확인
        match Command::new("claude")
            .arg("-h")
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // 1. settings.json 파일 생성
        let settings_file = self.create_settings_file()?;
        let settings_path = settings_file.path().to_str()
            .ok_or_else(|| AgentError::ExecutionError("Invalid settings path".to_string()))?;

        // 2. 세션 ID 조회
        let project_id = Self::extract_project_id(&context.project_path);
        let session_id = {
            let mapping = self.session_mapping.lock().unwrap();
            mapping.get(&project_id).cloned()
        };

        // 3. 명령 실행
        let mut cmd = self.build_command(context, settings_path, session_id.as_deref());
        cmd.current_dir(&context.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| AgentError::ExecutionError(format!("Failed to spawn claude: {}", e)))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdout".to_string()))?;

        // 4. JSON Lines 파싱
        let reader = BufReader::new(stdout);
        let lines_stream = tokio_stream::wrappers::LinesStream::new(reader.lines());

        // 5. 각 줄을 AgentEvent로 변환
        let session_mapping = self.session_mapping.clone();
        let project_id_clone = project_id.clone();

        let events_stream = lines_stream.filter_map(move |line_result| {
            let session_mapping = session_mapping.clone();
            let project_id = project_id_clone.clone();

            async move {
                match line_result {
                    Ok(line) => {
                        // JSON 파싱
                        match serde_json::from_str::<ClaudeMessage>(&line) {
                            Ok(msg) => {
                                convert_claude_message(msg, session_mapping, project_id).await
                            }
                            Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
                        }
                    }
                    Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
                }
            }
        });

        Ok(Box::pin(events_stream))
    }
}

/// Claude CLI 출력 메시지 구조
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeMessage {
    #[serde(rename = "system")]
    System {
        session_id: Option<String>,
        model: Option<String>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        content: Vec<ContentBlock>,
    },
    #[serde(rename = "user")]
    User {
        content: String,
    },
    #[serde(rename = "result")]
    Result {
        session_id: Option<String>,
        duration_ms: u64,
        total_cost_usd: f64,
        num_turns: u32,
        is_error: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

/// ClaudeMessage를 AgentEvent로 변환
async fn convert_claude_message(
    msg: ClaudeMessage,
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
    project_id: String,
) -> Option<Result<AgentEvent, AgentError>> {
    match msg {
        ClaudeMessage::System { session_id, .. } => {
            // session_id 저장
            if let Some(sid) = session_id {
                let mut mapping = session_mapping.lock().unwrap();
                mapping.insert(project_id, sid);
            }
            // SystemMessage는 UI에 표시하지 않음
            None
        }
        ClaudeMessage::Assistant { content } => {
            for block in content {
                match block {
                    ContentBlock::Text { text } => {
                        return Some(Ok(AgentEvent::MessageChunk(text)));
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        let tool_json = serde_json::to_string(&serde_json::json!({
                            "name": name,
                            "input": input
                        })).unwrap();
                        return Some(Ok(AgentEvent::ToolCall(tool_json)));
                    }
                    ContentBlock::ToolResult { .. } => {
                        // 도구 결과는 UI에 표시하지 않음
                    }
                }
            }
            None
        }
        ClaudeMessage::User { .. } => {
            // UserMessage는 UI에 표시하지 않음
            None
        }
        ClaudeMessage::Result { .. } => {
            // 세션 완료
            Some(Ok(AgentEvent::Completed))
        }
    }
}
```

### 4.2 필요 의존성

```toml
# Cargo.toml
[dependencies]
tempfile = "3.8"
serde_json = "1.0"
tokio-stream = "0.1"
```

### 4.3 환경 변수

- **필수**: 없음 (claude CLI가 로그인 정보 자체 관리)
- **선택**: `ANTHROPIC_API_KEY` (CLI 인증 대체용)

---

## 5. CursorAdapter 상세 설계

### 5.1 구조

```rust
// crates/core/src/agents/adapters/cursor_adapter.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::process::Command;

pub struct CursorAdapter {
    name: String,
    model: String,
    system_prompt: String,
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
}

impl CursorAdapter {
    pub fn new(name: String, model: String, system_prompt: String) -> Result<Self> {
        Ok(Self {
            name,
            model,
            system_prompt,
            session_mapping: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// AGENTS.md 파일 생성 (시스템 프롬프트)
    async fn ensure_agent_md(&self, project_path: &str) -> Result<(), AgentError> {
        let agent_md_path = std::path::Path::new(project_path).join("AGENTS.md");

        // 이미 존재하면 스킵
        if agent_md_path.exists() {
            return Ok(());
        }

        // 시스템 프롬프트를 파일에 작성
        fs::write(&agent_md_path, &self.system_prompt)
            .await
            .map_err(|e| AgentError::ExecutionError(format!("Failed to write AGENTS.md: {}", e)))?;

        Ok(())
    }

    fn build_command(
        &self,
        context: &ExecutionContext,
        session_id: Option<&str>,
    ) -> Command {
        let mut cmd = Command::new("cursor-agent");

        cmd.args(&[
            "--force",
            "-p", &context.instruction,
            "--output-format", "stream-json",
            "-m", &self.model,
        ]);

        // 세션 재개
        if let Some(sid) = session_id {
            cmd.args(&["--resume", sid]);
        }

        // API 키 (환경 변수에서)
        if let Ok(api_key) = std::env::var("CURSOR_API_KEY") {
            cmd.args(&["--api-key", &api_key]);
        }

        cmd
    }
}

#[async_trait]
impl Agent for CursorAdapter {
    async fn check_availability(&self) -> bool {
        match Command::new("cursor-agent")
            .arg("-h")
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // 1. AGENTS.md 파일 생성
        self.ensure_agent_md(&context.project_path).await?;

        // 2. 세션 ID 조회
        let project_id = ClaudeAdapter::extract_project_id(&context.project_path);
        let session_id = {
            let mapping = self.session_mapping.lock().unwrap();
            mapping.get(&project_id).cloned()
        };

        // 3. 명령 실행
        let mut cmd = self.build_command(context, session_id.as_deref());
        cmd.current_dir(&context.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| AgentError::ExecutionError(format!("Failed to spawn cursor-agent: {}", e)))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdout".to_string()))?;

        // 4. NDJSON 파싱
        let reader = BufReader::new(stdout);
        let lines_stream = tokio_stream::wrappers::LinesStream::new(reader.lines());

        let session_mapping = self.session_mapping.clone();
        let project_id_clone = project_id.clone();

        let events_stream = lines_stream.filter_map(move |line_result| {
            let session_mapping = session_mapping.clone();
            let project_id = project_id_clone.clone();

            async move {
                match line_result {
                    Ok(line) => {
                        match serde_json::from_str::<CursorEvent>(&line) {
                            Ok(event) => {
                                convert_cursor_event(event, session_mapping, project_id).await
                            }
                            Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
                        }
                    }
                    Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
                }
            }
        });

        Ok(Box::pin(events_stream))
    }
}

/// Cursor NDJSON 이벤트 구조
#[derive(Debug, Deserialize)]
struct CursorEvent {
    #[serde(rename = "type")]
    event_type: String,
    subtype: Option<String>,
    message: Option<serde_json::Value>,
    tool_call: Option<serde_json::Value>,
    session_id: Option<String>,
    duration_ms: Option<u64>,
}

async fn convert_cursor_event(
    event: CursorEvent,
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
    project_id: String,
) -> Option<Result<AgentEvent, AgentError>> {
    match event.event_type.as_str() {
        "system" => {
            // 초기화 이벤트 (hidden)
            None
        }
        "user" => {
            // 에코백 (suppress)
            None
        }
        "assistant" => {
            // 텍스트 델타
            if let Some(msg) = event.message {
                if let Some(content_array) = msg.get("content").and_then(|c| c.as_array()) {
                    for item in content_array {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            return Some(Ok(AgentEvent::MessageChunk(text.to_string())));
                        }
                    }
                }
            }
            None
        }
        "tool_call" => {
            if let Some(subtype) = &event.subtype {
                if subtype == "started" {
                    // 도구 호출 시작
                    if let Some(tool_call) = event.tool_call {
                        return Some(Ok(AgentEvent::ToolCall(tool_call.to_string())));
                    }
                }
                // "completed"는 UI에 표시하지 않음
            }
            None
        }
        "result" => {
            // session_id 저장
            if let Some(sid) = event.session_id {
                let mut mapping = session_mapping.lock().unwrap();
                mapping.insert(project_id, sid);
            }
            // 세션 완료
            Some(Ok(AgentEvent::Completed))
        }
        _ => None,
    }
}
```

### 5.2 환경 변수

- **선택**: `CURSOR_API_KEY` (Cursor API 키)

---

## 6. GeminiAdapter 상세 설계

### 6.1 구조

```rust
// crates/core/src/agents/adapters/gemini_adapter.rs

use tokio::io::{AsyncWriteExt, AsyncBufReadExt};

pub struct GeminiAdapter {
    name: String,
    model: String,
    system_prompt: String,
}

impl GeminiAdapter {
    pub fn new(name: String, model: String, system_prompt: String) -> Result<Self> {
        Ok(Self { name, model, system_prompt })
    }
}

#[async_trait]
impl Agent for GeminiAdapter {
    async fn check_availability(&self) -> bool {
        // gemini-cli 설치 확인 + API 키 확인
        let cli_available = Command::new("gemini-cli")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);

        let api_key_available = std::env::var("GEMINI_API_KEY").is_ok();

        cli_available && api_key_available
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // 1. gemini-cli 프로세스 실행 (stdin/stdout 파이프)
        let mut child = Command::new("gemini-cli")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| AgentError::ExecutionError(e.to_string()))?;

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        // 2. JSON-RPC 요청 생성 및 전송
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "generate",
            "params": {
                "model": self.model,
                "system": self.system_prompt,
                "prompt": context.instruction,
            }
        });

        let request_str = serde_json::to_string(&request)
            .map_err(|e| AgentError::ExecutionError(e.to_string()))?;

        stdin.write_all(request_str.as_bytes()).await
            .map_err(|e| AgentError::ExecutionError(e.to_string()))?;
        stdin.write_all(b"\n").await
            .map_err(|e| AgentError::ExecutionError(e.to_string()))?;
        stdin.flush().await
            .map_err(|e| AgentError::ExecutionError(e.to_string()))?;

        // 3. 응답 스트림 파싱
        let reader = BufReader::new(stdout);
        let lines_stream = tokio_stream::wrappers::LinesStream::new(reader.lines());

        let events_stream = lines_stream.filter_map(|line_result| async move {
            match line_result {
                Ok(line) => {
                    match serde_json::from_str::<JsonRpcResponse>(&line) {
                        Ok(response) => convert_gemini_response(response),
                        Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
                    }
                }
                Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
            }
        });

        Ok(Box::pin(events_stream))
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u32,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

fn convert_gemini_response(
    response: JsonRpcResponse
) -> Option<Result<AgentEvent, AgentError>> {
    if let Some(error) = response.error {
        return Some(Err(AgentError::ApiError(error.message)));
    }

    if let Some(result) = response.result {
        // result.text 또는 result.content 추출
        if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
            return Some(Ok(AgentEvent::MessageChunk(text.to_string())));
        }
        if let Some(parts) = result.get("parts").and_then(|p| p.as_array()) {
            for part in parts {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    return Some(Ok(AgentEvent::MessageChunk(text.to_string())));
                }
            }
        }
    }

    Some(Ok(AgentEvent::Completed))
}
```

### 6.2 환경 변수

- **필수**: `GEMINI_API_KEY` (Gemini API 키)

---

## 7. AgentFactory 설계

### 7.1 AgentType Enum

```rust
// crates/core/src/agents/agent_type.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentType {
    Claude,
    Cursor,
    Gemini,
    Codex,  // Phase 2
    Qwen,   // Phase 2
    Mock,
}

impl AgentType {
    /// model 필드에서 AgentType 추론
    pub fn from_model_name(model: &str) -> Self {
        let model_lower = model.to_lowercase();

        if model_lower.contains("claude") {
            Self::Claude
        } else if model_lower.starts_with("gpt") || model_lower.contains("cursor") {
            Self::Cursor
        } else if model_lower.contains("gemini") {
            Self::Gemini
        } else if model_lower.contains("codex") {
            Self::Codex
        } else if model_lower.contains("qwen") {
            Self::Qwen
        } else {
            // 기본값: Mock
            Self::Mock
        }
    }
}
```

### 7.2 AgentFactory 구현

```rust
// crates/core/src/agents/factory.rs

use crate::agents::{Agent, AgentType};
use pk_protocol::agent_models;
use std::sync::Arc;
use anyhow::{Result, anyhow};

pub struct AgentFactory;

impl AgentFactory {
    /// agent_models::Agent 설정에서 실제 Agent 인스턴스 생성
    pub fn create(config: &agent_models::Agent) -> Result<Arc<dyn Agent>> {
        let agent_type = AgentType::from_model_name(&config.model);

        match agent_type {
            AgentType::Claude => {
                let adapter = crate::agents::adapters::ClaudeAdapter::new(
                    config.name.clone(),
                    config.model.clone(),
                    config.system_prompt.clone(),
                )?;
                Ok(Arc::new(adapter))
            }
            AgentType::Cursor => {
                let adapter = crate::agents::adapters::CursorAdapter::new(
                    config.name.clone(),
                    config.model.clone(),
                    config.system_prompt.clone(),
                )?;
                Ok(Arc::new(adapter))
            }
            AgentType::Gemini => {
                let adapter = crate::agents::adapters::GeminiAdapter::new(
                    config.name.clone(),
                    config.model.clone(),
                    config.system_prompt.clone(),
                )?;
                Ok(Arc::new(adapter))
            }
            AgentType::Mock => {
                Ok(Arc::new(crate::agents::adapters::MockAgent::success()))
            }
            _ => {
                // Phase 2에서 구현할 어댑터
                eprintln!("Warning: Agent type {:?} not implemented yet, using Mock", agent_type);
                Ok(Arc::new(crate::agents::adapters::MockAgent::success()))
            }
        }
    }
}
```

---

## 8. ExecutionContext 확장

### 8.1 기존 구조

```rust
// crates/core/src/agents/base.rs (기존)

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub instruction: String,
}
```

### 8.2 확장된 구조

```rust
// crates/core/src/agents/base.rs (수정)

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 사용자 지시사항 (프롬프트)
    pub instruction: String,

    /// 프로젝트 경로 (작업 디렉토리)
    pub project_path: String,

    /// 첫 프롬프트 여부 (도구 필터링에 사용)
    pub is_initial_prompt: bool,

    /// 추가 컨텍스트 (이미지, 파일 등)
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone)]
pub enum Attachment {
    Image { path: String, mime_type: String },
    File { path: String, content: String },
}
```

### 8.3 변경 사항 영향 분석

**영향 받는 파일**:
- `crates/core/src/agents/base.rs` (ExecutionContext 정의)
- `crates/core/src/agents/adapters/mock_agent.rs` (테스트 코드)
- `crates/core/src/agents/manager.rs` (테스트 코드)

**마이그레이션 전략**:
```rust
// 기존 코드 호환성 유지를 위한 builder 패턴
impl ExecutionContext {
    pub fn new(instruction: String) -> Self {
        Self {
            instruction,
            project_path: std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            is_initial_prompt: false,
            attachments: vec![],
        }
    }

    pub fn with_project_path(mut self, path: String) -> Self {
        self.project_path = path;
        self
    }

    pub fn with_initial_prompt(mut self, is_initial: bool) -> Self {
        self.is_initial_prompt = is_initial;
        self
    }
}
```

---

## 9. 세션 관리 전략

### 9.1 세션 관리의 중요성

- **대화 연속성**: 이전 컨텍스트를 유지하여 자연스러운 대화
- **비용 절감**: 반복적인 초기화 방지
- **사용자 경험**: 프로젝트별 독립적인 세션

### 9.2 설계

#### 세션 ID 저장소

각 어댑터 내부에 `Arc<Mutex<HashMap<String, String>>>` 사용:
- **Key**: project_id (프로젝트 경로에서 추출)
- **Value**: session_id (CLI가 반환)

```rust
// 각 어댑터의 공통 패턴
pub struct XxxAdapter {
    // ...
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
}

impl XxxAdapter {
    fn extract_project_id(project_path: &str) -> String {
        std::path::Path::new(project_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(project_path)
            .to_string()
    }
}
```

#### 세션 라이프사이클

1. **조회**: execute() 시작 시 session_mapping에서 조회
2. **전달**: CLI 명령에 `--resume` 또는 `--resume-session-id` 인자로 전달
3. **추출**: CLI 응답에서 session_id 추출 (SystemMessage 또는 result 이벤트)
4. **저장**: session_mapping에 저장

### 9.3 향후 개선 방안 (Phase 2)

- **영구 저장소**: SQLite 또는 파일 기반 저장
- **세션 만료**: TTL 기반 자동 삭제
- **프로젝트별 설정**: `.pipeline-kit/sessions.json`

---

## 10. 구현 우선순위 및 단계

### Phase 1: 핵심 어댑터 (Ticket 2.2)

**목표**: Claude, Cursor, Gemini 어댑터 구현 및 검증

#### Step 1: 기반 인프라 (1일)
- [ ] `ExecutionContext` 확장
- [ ] `AgentType` enum 정의
- [ ] `AgentFactory` 뼈대 구현
- [ ] 기존 테스트 수정 (ExecutionContext 변경 대응)

#### Step 2: ClaudeAdapter (2일)
- [ ] `ClaudeAdapter` 구조체 및 기본 메서드
- [ ] `check_availability` 구현 및 테스트
- [ ] `execute` 메서드 구현 (subprocess + JSON 파싱)
- [ ] 세션 관리 구현
- [ ] 단위 테스트 작성
- [ ] 통합 테스트 (실제 CLI 호출, feature flag)

#### Step 3: CursorAdapter (2일)
- [ ] `CursorAdapter` 구조체
- [ ] AGENTS.md 생성 로직
- [ ] NDJSON 파싱 구현
- [ ] 세션 관리 구현
- [ ] 단위 테스트 작성

#### Step 4: GeminiAdapter (1일)
- [ ] `GeminiAdapter` 구조체
- [ ] JSON-RPC 통신 구현
- [ ] 단위 테스트 작성

#### Step 5: AgentFactory 완성 및 통합 (1일)
- [ ] `AgentFactory::create` 완성
- [ ] `AgentManager::new`에서 Factory 사용
- [ ] End-to-end 테스트
- [ ] 문서화

**예상 기간**: 7일

---

### Phase 2: 추가 어댑터 (후속 Ticket)

#### CodexAdapter
- OpenAI Codex CLI 연동
- apply_patch 특화 처리

#### QwenAdapter
- Qwen CLI 연동

**예상 기간**: 3일

---

## 11. 테스트 전략

### 11.1 단위 테스트

#### 테스트 수준

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Level 1: 기본 생성 및 설정
    #[test]
    fn test_claude_adapter_new() {
        let adapter = ClaudeAdapter::new(
            "test".to_string(),
            "claude-sonnet-4.5".to_string(),
            "test prompt".to_string(),
        );
        assert!(adapter.is_ok());
    }

    // Level 2: check_availability (mock 없이)
    #[tokio::test]
    async fn test_claude_adapter_availability_without_cli() {
        // claude CLI 없는 환경 시뮬레이션
        let adapter = ClaudeAdapter::new(...).unwrap();
        // PATH를 수정하거나, which 크레이트를 mock
        // 이 테스트는 실제로는 integration test에 가까움
    }

    // Level 3: execute with mock subprocess (고급)
    // tokio::process::Command를 mock하기 어려우므로
    // 실제 CLI 호출은 integration test로 분리
}
```

### 11.2 통합 테스트

```rust
// tests/claude_adapter_integration.rs

#[tokio::test]
#[cfg(feature = "integration-tests")]
async fn test_claude_adapter_real_execution() {
    // 실제 claude CLI 호출
    // ANTHROPIC_API_KEY 환경 변수 필요

    let adapter = ClaudeAdapter::new(
        "test".to_string(),
        "claude-sonnet-4.5".to_string(),
        "You are a test assistant.".to_string(),
    ).unwrap();

    let context = ExecutionContext {
        instruction: "Hello, tell me a joke.".to_string(),
        project_path: "/tmp/test-project".to_string(),
        is_initial_prompt: true,
        attachments: vec![],
    };

    let mut stream = adapter.execute(&context).await.unwrap();
    let mut events = vec![];

    while let Some(event) = stream.next().await {
        events.push(event);
    }

    assert!(!events.is_empty());
    assert!(events.iter().any(|e| matches!(e, Ok(AgentEvent::Completed))));
}
```

#### Feature Flag 설정

```toml
# Cargo.toml
[features]
integration-tests = []

# 실행:
# cargo test --features integration-tests
```

### 11.3 E2E 테스트

```rust
// tests/agent_factory_e2e.rs

#[tokio::test]
async fn test_agent_factory_creates_correct_adapter() {
    let configs = vec![
        agent_models::Agent {
            name: "claude-dev".to_string(),
            model: "claude-sonnet-4.5".to_string(),
            description: "Claude developer".to_string(),
            color: "blue".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
        },
    ];

    let manager = AgentManager::new(configs);

    assert!(manager.has_agent("claude-dev"));

    let ctx = ExecutionContext {
        instruction: "Hello".to_string(),
        project_path: "/tmp/test".to_string(),
        is_initial_prompt: true,
        attachments: vec![],
    };

    // check_availability는 실제 CLI 없으면 실패하므로
    // mock adapter로 대체되는지 확인
    let result = manager.execute("claude-dev", &ctx).await;
    // Mock으로 폴백되면 성공
    assert!(result.is_ok());
}
```

---

## 12. 파일 구조

```
pipeline-kit-rs/crates/core/src/agents/
├── mod.rs                          # 기존, adapters 모듈 추가
├── base.rs                         # 수정: ExecutionContext 확장
├── manager.rs                      # 수정: Factory 사용
├── agent_type.rs                   # 신규: AgentType enum
├── factory.rs                      # 신규: AgentFactory
└── adapters/
    ├── mod.rs                      # 수정: 새 어댑터 export
    ├── mock_agent.rs               # 기존
    ├── claude_adapter.rs           # 신규
    ├── cursor_adapter.rs           # 신규
    ├── gemini_adapter.rs           # 신규
    ├── codex_adapter.rs            # Phase 2
    └── qwen_adapter.rs             # Phase 2

tests/
├── agent_factory_e2e.rs            # 신규: E2E 테스트
├── claude_adapter_integration.rs   # 신규: Claude 통합 테스트
└── cursor_adapter_integration.rs   # 신규: Cursor 통합 테스트
```

### mod.rs 변경사항

```rust
// crates/core/src/agents/mod.rs

mod base;
mod manager;
mod agent_type;
mod factory;

pub mod adapters;

pub use base::{Agent, AgentError, AgentEvent, ExecutionContext, Attachment};
pub use manager::AgentManager;
pub use agent_type::AgentType;
pub use factory::AgentFactory;

// adapters 재export
pub use adapters::{MockAgent, ClaudeAdapter, CursorAdapter, GeminiAdapter};
```

```rust
// crates/core/src/agents/adapters/mod.rs

mod mock_agent;
mod claude_adapter;
mod cursor_adapter;
mod gemini_adapter;

pub use mock_agent::MockAgent;
pub use claude_adapter::ClaudeAdapter;
pub use cursor_adapter::CursorAdapter;
pub use gemini_adapter::GeminiAdapter;
```

---

## 13. 의존성 및 환경 변수

### 13.1 Cargo.toml 추가

```toml
# pipeline-kit-rs/crates/core/Cargo.toml

[dependencies]
# 기존
async-trait = "0.1"
tokio = { version = "1.40", features = ["full"] }
tokio-stream = "0.1"
thiserror = "2.0"
pk-protocol = { path = "../protocol" }

# 신규 추가
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tempfile = "3.8"
which = "6.0"

[dev-dependencies]
tokio = { version = "1.40", features = ["full"] }
tempfile = "3.8"

[features]
integration-tests = []
```

### 13.2 환경 변수 정리

| 어댑터 | 환경 변수 | 필수 여부 | 용도 | 확인 방법 |
|--------|-----------|----------|------|----------|
| Claude | - | ❌ | - | `claude -h` |
| Cursor | `CURSOR_API_KEY` | ❌ (선택) | API 인증 | `cursor-agent -h` |
| Gemini | `GEMINI_API_KEY` | ✅ | API 인증 | `gemini-cli --version` |
| Codex (Phase 2) | `OPENAI_API_KEY` | ✅ | API 인증 | `codex-cli --version` |
| Qwen (Phase 2) | `QWEN_API_KEY` | ❌ (선택) | API 인증 | `qwen-cli --version` |

### 13.3 CLI 설치 확인 스크립트

```bash
#!/bin/bash
# scripts/check_agent_availability.sh

echo "Checking agent CLI availability..."

# Claude
if command -v claude &> /dev/null; then
    echo "✅ Claude CLI: $(claude --version)"
else
    echo "❌ Claude CLI: Not installed"
fi

# Cursor
if command -v cursor-agent &> /dev/null; then
    echo "✅ Cursor Agent CLI: $(cursor-agent --version)"
else
    echo "❌ Cursor Agent CLI: Not installed"
fi

# Gemini
if command -v gemini-cli &> /dev/null; then
    echo "✅ Gemini CLI: $(gemini-cli --version)"
else
    echo "❌ Gemini CLI: Not installed"
fi

# Environment variables
echo ""
echo "Environment variables:"
[ -n "$CURSOR_API_KEY" ] && echo "✅ CURSOR_API_KEY: Set" || echo "❌ CURSOR_API_KEY: Not set"
[ -n "$GEMINI_API_KEY" ] && echo "✅ GEMINI_API_KEY: Set" || echo "❌ GEMINI_API_KEY: Not set"
```

---

## 14. 구현 체크리스트

### Phase 1: Infrastructure

- [ ] `ExecutionContext` 확장
  - [ ] `project_path` 필드 추가
  - [ ] `is_initial_prompt` 필드 추가
  - [ ] `Attachment` enum 정의
  - [ ] Builder 패턴 구현
- [ ] `AgentType` enum 정의
  - [ ] `from_model_name` 구현
  - [ ] 테스트 작성
- [ ] `AgentFactory` 뼈대
  - [ ] `create` 메서드 시그니처
  - [ ] Mock 반환 기본 구현
- [ ] 기존 테스트 마이그레이션
  - [ ] `MockAgent` 테스트 수정
  - [ ] `AgentManager` 테스트 수정

### Phase 1: ClaudeAdapter

- [ ] 구조체 정의
  - [ ] 필드 정의 (name, model, system_prompt, session_mapping)
  - [ ] `new` 생성자
- [ ] `check_availability` 구현
  - [ ] `claude -h` 실행
  - [ ] 테스트 작성
- [ ] `execute` 구현
  - [ ] `create_settings_file` 구현
  - [ ] `build_command` 구현
  - [ ] subprocess 실행
  - [ ] JSON Lines 파싱
  - [ ] 메시지 타입별 변환 (`convert_claude_message`)
  - [ ] 세션 관리 (session_id 추출 및 저장)
- [ ] 테스트
  - [ ] 단위 테스트 (생성, settings 파일)
  - [ ] 통합 테스트 (feature flag)

### Phase 1: CursorAdapter

- [ ] 구조체 정의
- [ ] `ensure_agent_md` 구현
- [ ] `check_availability` 구현
- [ ] `execute` 구현
  - [ ] NDJSON 파싱
  - [ ] 이벤트 타입별 변환
  - [ ] 세션 관리
- [ ] 테스트

### Phase 1: GeminiAdapter

- [ ] 구조체 정의
- [ ] `check_availability` 구현
- [ ] `execute` 구현
  - [ ] JSON-RPC 요청/응답
- [ ] 테스트

### Phase 1: Integration

- [ ] `AgentFactory::create` 완성
  - [ ] 각 AgentType에 대한 매칭
  - [ ] 에러 핸들링
- [ ] `AgentManager::new` 수정
  - [ ] Factory 사용
  - [ ] 폴백 로직 유지
- [ ] E2E 테스트
- [ ] 문서화
  - [ ] README 업데이트
  - [ ] 환경 변수 가이드
  - [ ] 예제 코드

---

## 15. 예상 이슈 및 해결 방안

### Issue 1: subprocess stdout buffering
**문제**: stdout이 버퍼링되어 실시간 스트리밍이 안 될 수 있음
**해결**: `BufReader::lines()`는 줄 단위로 읽으므로 대부분 문제없음. 필요시 `tokio::io::split` 사용

### Issue 2: JSON 파싱 실패
**문제**: CLI 출력에 일반 텍스트와 JSON이 섞여 있을 수 있음
**해결**: `serde_json::from_str` 실패 시 해당 라인 무시 또는 로깅

### Issue 3: CLI 설치 확인
**문제**: `check_availability`가 환경에 따라 다르게 동작
**해결**: 명확한 에러 메시지와 설치 가이드 제공

### Issue 4: 세션 동기화
**문제**: 동시에 여러 요청이 세션을 수정할 수 있음
**해결**: `Arc<Mutex<HashMap>>` 사용으로 해결됨

### Issue 5: Windows 경로 처리
**문제**: Windows에서 경로 구분자 차이
**해결**: `std::path::Path` 사용으로 OS 독립적 처리

---

## 16. 문서화 계획

### README 업데이트

```markdown
# Agent Adapters

## Supported Agents

- **Claude**: Anthropic Claude via Claude Code CLI
- **Cursor**: Cursor Agent CLI
- **Gemini**: Google Gemini CLI

## Installation

### Claude
\`\`\`bash
npm install -g @anthropic-ai/claude-code
claude login
\`\`\`

### Cursor
\`\`\`bash
curl https://cursor.com/install -fsS | bash
cursor-agent login
\`\`\`

### Gemini
\`\`\`bash
pip install gemini-cli
export GEMINI_API_KEY=your_key_here
\`\`\`

## Usage

\`\`\`rust
use pk_core::{AgentManager, ExecutionContext};

let configs = load_config("/path/to/.pipeline-kit").await?;
let manager = AgentManager::new(configs.agents);

let context = ExecutionContext::new("Write a hello world program".to_string())
    .with_project_path("/path/to/project".to_string())
    .with_initial_prompt(true);

let mut stream = manager.execute("developer", &context).await?;

while let Some(event) = stream.next().await {
    match event? {
        AgentEvent::MessageChunk(text) => println!("{}", text),
        AgentEvent::ToolCall(tool) => println!("Tool: {}", tool),
        AgentEvent::Completed => break,
        _ => {}
    }
}
\`\`\`
```

---

## 17. 마일스톤

### Milestone 1: Infrastructure Ready (Day 1)
- ExecutionContext 확장 완료
- AgentType, AgentFactory 뼈대 완료
- 기존 테스트 마이그레이션 완료

### Milestone 2: Claude Working (Day 3)
- ClaudeAdapter 완전 구현
- 단위 테스트 통과
- 통합 테스트 통과 (feature flag)

### Milestone 3: Cursor Working (Day 5)
- CursorAdapter 완전 구현
- AGENTS.md 생성 동작 확인
- 세션 관리 검증

### Milestone 4: Gemini Working (Day 6)
- GeminiAdapter 완전 구현
- JSON-RPC 통신 검증

### Milestone 5: Integration Complete (Day 7)
- AgentFactory 완성
- AgentManager 통합
- E2E 테스트 통과
- 문서화 완료

---

## 18. 결론

이 설계는 Python 레퍼런스 코드의 검증된 패턴을 Rust로 충실히 이식합니다. 주요 특징:

1. **검증된 접근**: Python에서 이미 동작하는 방식 그대로 구현
2. **안정성**: subprocess 기반으로 CLI 오류에 강건
3. **확장성**: 새 어댑터 추가가 간단 (AgentFactory에 case 추가)
4. **테스트 가능**: 단위/통합/E2E 테스트 전략 명확
5. **세션 관리**: 대화 연속성 보장

Phase 1 완료 후 Ticket 2.2는 완전히 검증된 상태가 됩니다.

---

**Document Version**: 2.0
**Author**: AI Assistant
**Date**: 2025-10-11
**Status**: Final Design - Ready for Implementation
