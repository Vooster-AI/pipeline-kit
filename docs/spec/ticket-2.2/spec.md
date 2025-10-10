# Ticket 2.2: 에이전트 어댑터 패턴 및 관리자 구현 (`pk-core` 크레이트)

**Goal**: 다양한 AI 코딩 에이전트(CLI, SDK 기반 등)를 일관된 인터페이스로 제어하기 위한 추상화 계층을 구축합니다. `Agent` 트레이트(Adapter Pattern)를 정의하고, 이를 관리하는 `AgentManager`를 구현하여, 새로운 에이전트를 시스템의 다른 부분에 영향을 주지 않고 쉽게 추가할 수 있는 확장 가능한 구조를 만듭니다.

**Core Modules & Roles**:

- `pipeline-kit-rs/crates/core/src/agents/base.rs`: 모든 에이전트가 구현해야 할 공통 계약인 `Agent` 트레이트를 정의합니다.
- `pipeline-kit-rs/crates/core/src/agents/adapters/`: 각기 다른 AI 서비스에 대한 구체적인 어댑터 구현체들이 위치할 모듈 디렉터리입니다.
- `pipeline-kit-rs/crates/core/src/agents/adapters/mock_agent.rs`: TDD 및 초기 개발을 위한 `MockAgent`를 구현합니다.
- `pipeline-kit-rs/crates/core/src/agents/manager.rs`: 모든 `Agent` 어댑터를 등록하고, 이름으로 조회하며, 실행을 오케스트레이션하는 `AgentManager`를 구현합니다.

**Interfaces**:

- **`agents/base.rs`**: `Agent` 트레이트와 입출력 데이터 구조를 정의합니다.

  ```rust
  use async_trait::async_trait;
  use thiserror::Error;
  use tokio_stream::Stream;
  use std::pin::Pin;

  // Agent 실행 시 전달될 컨텍스트 정보
  pub struct ExecutionContext {
      pub instruction: String,
      // TODO: 향후 필요한 다른 필드들(예: 참조 파일 내용) 추가
  }

  // Agent가 스트리밍으로 반환할 이벤트 타입
  #[derive(Debug, Clone, PartialEq)]
  pub enum AgentEvent {
      Thought(String),
      ToolCall(String), // 예시: 툴 호출 정보
      MessageChunk(String),
      Completed,
  }

  #[derive(Error, Debug)]
  pub enum AgentError {
      #[error("Agent not available: {0}")]
      NotAvailable(String),
      #[error("API call failed: {0}")]
      ApiError(String),
      #[error("Stream parsing error: {0}")]
      StreamParseError(String),
      #[error("Execution failed: {0}")]
      ExecutionError(String),
  }

  #[async_trait]
  pub trait Agent: Send + Sync {
      /// 에이전트가 현재 시스템에서 사용 가능한지 확인합니다.
      async fn check_availability(&self) -> bool;

      /// 지시사항을 실행하고, 결과를 AgentEvent 스트림으로 반환합니다.
      async fn execute(
          &self,
          context: &ExecutionContext,
      ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError>;
  }
  ```

**Reference Code & Key Points**:

아래는 제공된 Python 코드베이스에서 `pipeline-kit`의 Rust 구현에 직접적으로 적용해야 할 핵심 패턴과 아이디어입니다.

---

#### 1. `base.py` (`BaseCLI`) -> `agents/base.rs` (`trait Agent`)

- **역할**: Python의 `BaseCLI` 추상 클래스는 Rust의 `trait Agent`에 해당합니다. 이는 모든 어댑터가 따라야 할 **계약(Contract)**입니다.
- **핵심 아이디어**:
  1.  **`check_availability`**: 각 어댑터는 해당 CLI가 설치되어 있는지, 또는 API 키가 설정되어 있는지 등을 확인하는 로직을 반드시 구현해야 합니다. 이는 `AgentManager`의 **폴백(Fallback) 로직**의 기반이 됩니다.
      - **Rust Hint**: `which::which("cli-name")`을 사용해 CLI 설치 여부를 확인하거나, `std::env::var("API_KEY_ENV_VAR")`를 사용해 환경 변수를 확인할 수 있습니다.
  2.  **`execute_with_streaming`**: 이 메서드는 Rust에서 `async fn execute(...)`에 해당하며, 반환 타입은 Python의 `AsyncGenerator` 대신 `Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>`가 됩니다.
      - **Rust Hint**: `tokio_stream::wrappers::UnboundedReceiverStream` 이나 `async-stream` 크레이트를 사용하면 이 스트림을 쉽게 구현할 수 있습니다.
  3.  **데이터 표준화**: `_normalize_role`, `_extract_content`, `_create_tool_summary` 등의 헬퍼 함수들이 가진 역할은 매우 중요합니다. 각 Rust 어댑터의 `execute` 메서드는 **반드시** 해당 에이전트의 고유한 출력(일반 텍스트, JSON, SDK 객체 등)을 표준화된 `AgentEvent` 열거형으로 변환하여 스트림을 통해 `yield`해야 합니다.

<details>
<summary><b>Click to view raw content of `apps/api/app/services/cli/base.py`</b></summary>

```python
"""
Base abstractions and shared utilities for CLI providers.

This module defines a precise, minimal adapter contract (BaseCLI) and common
helpers so that adding a new provider remains consistent and easy.
"""
from __future inport annotations

import os
import uuid
from abc import ABC, abstractmethod
from datetime import datetime
from enum import Enum
from typing import Any, AsyncGenerator, Callable, Dict, List, Optional

from app.models.messages import Message


def get_project_root() -> str:
    """Return project root directory using relative path navigation.

    This function intentionally mirrors the logic previously embedded in
    unified_manager.py so imports remain stable after refactor.
    """
    current_file_dir = os.path.dirname(os.path.abspath(__file__))
    # base.py is in: app/services/cli/
    # Navigate: cli -> services -> app -> api -> apps -> project-root
    project_root = os.path.join(current_file_dir, "..", "..", "..", "..", "..")
    return os.path.abspath(project_root)


def get_display_path(file_path: str) -> str:
    """Convert absolute path to a shorter display path scoped to the project.

    - Strips the project root prefix when present
    - Compacts repo-specific prefixes (e.g., data/projects -> …/)
    """
    try:
        project_root = get_project_root()
        if file_path.startswith(project_root):
            display_path = file_path.replace(project_root + "/", "")
            return display_path.replace("data/projects/", "…/")
    except Exception:
        pass
    return file_path


# Model mapping from unified names to CLI-specific names
MODEL_MAPPING: Dict[str, Dict[str, str]] = {
    "claude": {
        "opus-4.1": "claude-opus-4-1-20250805",
        "sonnet-4.5": "claude-sonnet-4-5-20250929",
        "opus-4": "claude-opus-4-20250514",
        "haiku-3.5": "claude-3-5-haiku-20241022",
        # Handle claude-prefixed model names
        "claude-sonnet-4.5": "claude-sonnet-4-5-20250929",
        "claude-opus-4.1": "claude-opus-4-1-20250805",
        "claude-opus-4": "claude-opus-4-20250514",
        "claude-haiku-3.5": "claude-3-5-haiku-20241022",
        # Support direct full model names
        "claude-opus-4-1-20250805": "claude-opus-4-1-20250805",
        "claude-sonnet-4-5-20250929": "claude-sonnet-4-5-20250929",
        "claude-opus-4-20250514": "claude-opus-4-20250514",
        "claude-3-5-haiku-20241022": "claude-3-5-haiku-20241022",
    },
    "cursor": {
        "gpt-5": "gpt-5",
        "sonnet-4.5": "sonnet-4.5",
        "opus-4.1": "opus-4.1",
        "sonnet-4-thinking": "sonnet-4-thinking",
        # Handle mapping from unified Claude model names
        "claude-sonnet-4.5": "sonnet-4.5",
        "claude-opus-4.1": "opus-4.1",
        "claude-sonnet-4-5-20250929": "sonnet-4.5",
        "claude-opus-4-1-20250805": "opus-4.1",
    },
    "codex": {
        "gpt-5": "gpt-5",
        "gpt-4o": "gpt-4o",
        "gpt-4o-mini": "gpt-4o-mini",
        "o1-preview": "o1-preview",
        "o1-mini": "o1-mini",
        "claude-3.5-sonnet": "claude-3.5-sonnet",
        "claude-3-haiku": "claude-3-haiku",
        # Handle unified model names
        "sonnet-4.5": "claude-3.5-sonnet",
        "claude-sonnet-4.5": "claude-3.5-sonnet",
        "haiku-3.5": "claude-3-haiku",
        "claude-haiku-3.5": "claude-3-haiku",
    },
    "qwen": {
        # Unified name → provider mapping
        "qwen3-coder-plus": "qwen-coder",
        "Qwen3 Coder Plus": "qwen-coder",
        # Allow direct
        "qwen-coder": "qwen-coder",
    },
    "gemini": {
        "gemini-2.5-pro": "gemini-2.5-pro",
        "gemini-2.5-flash": "gemini-2.5-flash",
    },
}


class CLIType(str, Enum):
    """Provider key used across the manager and adapters."""

    CLAUDE = "claude"
    CURSOR = "cursor"
    CODEX = "codex"
    QWEN = "qwen"
    GEMINI = "gemini"


class BaseCLI(ABC):
    """Abstract adapter contract for CLI providers.

    Subclasses must implement availability checks, streaming execution, and
    session persistence. Common utilities (model mapping, content parsing,
    tool summaries) are provided here for reuse.
    """

    def __init__(self, cli_type: CLIType):
        self.cli_type = cli_type

    # ---- Mandatory adapter interface ------------------------------------
    @abstractmethod
    async def check_availability(self) -> Dict[str, Any]:
        """Return provider availability/configuration status.

        Expected keys in the returned dict used by the manager:
        - available: bool
        - configured: bool
        - models/default_models (optional): List[str]
        - error (optional): str
        """

    @abstractmethod
    async def execute_with_streaming(
        self,
        instruction: str,
        project_path: str,
        session_id: Optional[str] = None,
        log_callback: Optional[Callable[[str], Any]] = None,
        images: Optional[List[Dict[str, Any]]] = None,
        model: Optional[str] = None,
        is_initial_prompt: bool = False,
    ) -> AsyncGenerator[Message, None]:
        """Execute an instruction and yield `Message` objects in real time."""

    @abstractmethod
    async def get_session_id(self, project_id: str) -> Optional[str]:
        """Return the active session ID for a project, if any."""

    @abstractmethod
    async def set_session_id(self, project_id: str, session_id: str) -> None:
        """Persist the active session ID for a project."""

    # ---- Common helpers (available to adapters) --------------------------
    def _get_cli_model_name(self, model: Optional[str]) -> Optional[str]:
        """Translate unified model name to provider-specific model name.

        If the input is already a provider name or mapping fails, return as-is.
        """
        if not model:
            return None

        from app.core.terminal_ui import ui

        ui.debug(f"Input model: '{model}' for CLI: {self.cli_type.value}", "Model")
        cli_models = MODEL_MAPPING.get(self.cli_type.value, {})

        # Try exact mapping
        if model in cli_models:
            mapped_model = cli_models[model]
            ui.info(
                f"Mapped '{model}' to '{mapped_model}' for {self.cli_type.value}", "Model"
            )
            return mapped_model

        # Already a provider-specific name
        if model in cli_models.values():
            ui.info(
                f"Using direct model name '{model}' for {self.cli_type.value}", "Model"
            )
            return model

        # Debug available models
        available_models = list(cli_models.keys())
        ui.warning(
            f"Model '{model}' not found in mapping for {self.cli_type.value}", "Model"
        )
        ui.debug(
            f"Available models for {self.cli_type.value}: {available_models}", "Model"
        )
        ui.warning(f"Using model as-is: '{model}'", "Model")
        return model

    def get_supported_models(self) -> List[str]:
        cli_models = MODEL_MAPPING.get(self.cli_type.value, {})
        return list(cli_models.keys()) + list(cli_models.values())

    def is_model_supported(self, model: str) -> bool:
        return (
            model in self.get_supported_models()
            or model in MODEL_MAPPING.get(self.cli_type.value, {}).values()
        )

    def parse_message_data(self, data: Dict[str, Any], project_id: str, session_id: str) -> Message:
        """Normalize provider-specific message payload to our `Message`."""
        return Message(
            id=str(uuid.uuid4()),
            project_id=project_id,
            role=self._normalize_role(data.get("role", "assistant")),
            message_type="chat",
            content=self._extract_content(data),
            metadata_json={
                **data,
                "cli_type": self.cli_type.value,
                "original_format": data,
            },
            session_id=session_id,
            created_at=datetime.utcnow(),
        )

    def _normalize_role(self, role: str) -> str:
        role_mapping = {
            "model": "assistant",
            "ai": "assistant",
            "human": "user",
            "bot": "assistant",
        }
        return role_mapping.get(role.lower(), role.lower())

    def _extract_content(self, data: Dict[str, Any]) -> str:
        """Extract best-effort text content from various provider formats."""
        # Claude content array
        if "content" in data and isinstance(data["content"], list):
            content = ""
            for item in data["content"]:
                if item.get("type") == "text":
                    content += item.get("text", "")
                elif item.get("type") == "tool_use":
                    tool_name = item.get("name", "Unknown")
                    tool_input = item.get("input", {})
                    summary = self._create_tool_summary(tool_name, tool_input)
                    content += f"{summary}\n"
            return content

        # Simple text
        elif "content" in data:
            return str(data["content"])

        # Gemini parts
        elif "parts" in data:
            content = ""
            for part in data["parts"]:
                if "text" in part:
                    content += part.get("text", "")
                elif "functionCall" in part:
                    func_call = part["functionCall"]
                    tool_name = func_call.get("name", "Unknown")
                    tool_input = func_call.get("args", {})
                    summary = self._create_tool_summary(tool_name, tool_input)
                    content += f"{summary}\n"
            return content

        # OpenAI/Codex choices
        elif "choices" in data and data["choices"]:
            choice = data["choices"][0]
            if "message" in choice:
                return choice["message"].get("content", "")
            elif "text" in choice:
                return choice.get("text", "")

        # Direct text fields
        elif "text" in data:
            return str(data["text"])
        elif "message" in data:
            if isinstance(data["message"], dict):
                return self._extract_content(data["message"])
            return str(data["message"])

        # Generic response field
        elif "response" in data:
            return str(data["response"])

        # Delta streaming
        elif "delta" in data and "content" in data["delta"]:
            return str(data["delta"]["content"])

        # Fallback
        else:
            return str(data)

    def _normalize_tool_name(self, tool_name: str) -> str:
        """Normalize tool names across providers to a unified label."""
        key = (tool_name or "").strip()
        key_lower = key.replace(" ", "").lower()
        tool_mapping = {
            # File operations
            "read_file": "Read",
            "read": "Read",
            "write_file": "Write",
            "write": "Write",
            "edit_file": "Edit",
            "replace": "Edit",
            "edit": "Edit",
            "delete": "Delete",
            # Qwen/Gemini variants (CamelCase / spaced)
            "readfile": "Read",
            "readfolder": "LS",
            "readmanyfiles": "Read",
            "writefile": "Write",
            "findfiles": "Glob",
            "savememory": "SaveMemory",
            "save memory": "SaveMemory",
            "searchtext": "Grep",
            # Terminal operations
            "shell": "Bash",
            "run_terminal_command": "Bash",
            # Search operations
            "search_file_content": "Grep",
            "codebase_search": "Grep",
            "grep": "Grep",
            "find_files": "Glob",
            "glob": "Glob",
            "list_directory": "LS",
            "list_dir": "LS",
            "ls": "LS",
            "semSearch": "SemSearch",
            # Web operations
            "google_web_search": "WebSearch",
            "web_search": "WebSearch",
            "googlesearch": "WebSearch",
            "web_fetch": "WebFetch",
            "fetch": "WebFetch",
            # Task/Memory operations
            "save_memory": "SaveMemory",
            # Codex operations
            "exec_command": "Bash",
            "apply_patch": "Edit",
            "mcp_tool_call": "MCPTool",
            # Generic simple names
            "search": "Grep",
        }
        return tool_mapping.get(tool_name, tool_mapping.get(key_lower, key))

    def _get_clean_tool_display(self, tool_name: str, tool_input: Dict[str, Any]) -> str:
        """Return a concise, Claude-like tool usage display line."""
        normalized_name = self._normalize_tool_name(tool_name)

        if normalized_name == "Read":
            file_path = (
                tool_input.get("file_path")
                or tool_input.get("path")
                or tool_input.get("file", "")
            )
            if file_path:
                filename = file_path.split("/")[-1]
                return f"Reading {filename}"
            return "Reading file"
        elif normalized_name == "Write":
            file_path = (
                tool_input.get("file_path")
                or tool_input.get("path")
                or tool_input.get("file", "")
            )
            if file_path:
                filename = file_path.split("/")[-1]
                return f"Writing {filename}"
            return "Writing file"
        elif normalized_name == "Edit":
            file_path = (
                tool_input.get("file_path")
                or tool_input.get("path")
                or tool_input.get("file", "")
            )
            if file_path:
                filename = file_path.split("/")[-1]
                return f"Editing {filename}"
            return "Editing file"
        elif normalized_name == "Bash":
            command = (
                tool_input.get("command")
                or tool_input.get("cmd")
                or tool_input.get("script", "")
            )
            if command:
                cmd_display = command.split()[0] if command.split() else command
                return f"Running {cmd_display}"
            return "Running command"
        elif normalized_name == "LS":
            return "Listing directory"
        elif normalized_name == "TodoWrite":
            return "Planning next steps"
        elif normalized_name == "WebSearch":
            query = tool_input.get("query", "")
            if query:
                return f"Searching: {query[:50]}..."
            return "Web search"
        elif normalized_name == "WebFetch":
            url = tool_input.get("url", "")
            if url:
                domain = (
                    url.split("//")[-1].split("/")[0]
                    if "//" in url
                    else url.split("/")[0]
                )
                return f"Fetching from {domain}"
            return "Fetching web content"
        else:
            return f"Using {tool_name}"

    def _create_tool_summary(self, tool_name: str, tool_input: Dict[str, Any]) -> str:
        """Create a visual markdown summary for tool usage.

        NOTE: Special-cases Codex `apply_patch` to render one-line summaries per
        file similar to Claude Code.
        """
        # Handle apply_patch BEFORE normalization to avoid confusion with Edit
        if tool_name == "apply_patch":
            changes = tool_input.get("changes", {})
            if isinstance(changes, dict) and changes:
                if len(changes) == 1:
                    path, change = next(iter(changes.items()))
                    filename = str(path).split("/")[-1]
                    if isinstance(change, dict):
                        if "add" in change:
                            return f"**Write** `{filename}`"
                        elif "delete" in change:
                            return f"**Delete** `{filename}`"
                        elif "update" in change:
                            upd = change.get("update") or {}
                            move_path = upd.get("move_path")
                            if move_path:
                                new_filename = move_path.split("/")[-1]
                                return f"**Rename** `{filename}` → `{new_filename}`"
                            else:
                                return f"**Edit** `{filename}`"
                        else:
                            return f"**Edit** `{filename}`"
                    else:
                        return f"**Edit** `{filename}`"
                else:
                    file_summaries: List[str] = []
                    for raw_path, change in list(changes.items())[:3]:  # max 3 files
                        path = str(raw_path)
                        filename = path.split("/")[-1]
                        if isinstance(change, dict):
                            if "add" in change:
                                file_summaries.append(f"• **Write** `{filename}`")
                            elif "delete" in change:
                                file_summaries.append(f"• **Delete** `{filename}`")
                            elif "update" in change:
                                upd = change.get("update") or {}
                                move_path = upd.get("move_path")
                                if move_path:
                                    new_filename = move_path.split("/")[-1]
                                    file_summaries.append(
                                        f"• **Rename** `{filename}` → `{new_filename}`"
                                    )
                                else:
                                    file_summaries.append(f"• **Edit** `{filename}`")
                            else:
                                file_summaries.append(f"• **Edit** `{filename}`")
                        else:
                            file_summaries.append(f"• **Edit** `{filename}`")

                    result = "\n".join(file_summaries)
                    if len(changes) > 3:
                        result += f"\n• ... +{len(changes) - 3} more files"
                    return result
            return "**ApplyPatch** `files`"

        # Normalize name after handling apply_patch
        normalized_name = self._normalize_tool_name(tool_name)

        if normalized_name == "Edit":
            file_path = (
                tool_input.get("file_path")
                or tool_input.get("path")
                or tool_input.get("file", "")
            )
            if file_path:
                display_path = get_display_path(file_path)
                if len(display_path) > 40:
                    display_path = "…/" + "/".join(display_path.split("/")[-2:])
                return f"**Edit** `{display_path}`"
            return "**Edit** `file`"
        elif normalized_name == "Read":
            file_path = (
                tool_input.get("file_path")
                or tool_input.get("path")
                or tool_input.get("file", "")
            )
            if file_path:
                display_path = get_display_path(file_path)
                if len(display_path) > 40:
                    display_path = "…/" + "/".join(display_path.split("/")[-2:])
                return f"**Read** `{display_path}`"
            return "**Read** `file`"
        elif normalized_name == "Bash":
            command = (
                tool_input.get("command")
                or tool_input.get("cmd")
                or tool_input.get("script", "")
            )
            if command:
                display_cmd = command[:40] + "..." if len(command) > 40 else command
                return f"**Bash** `{display_cmd}`"
            return "**Bash** `command`"
        elif normalized_name == "TodoWrite":
            return "`Planning for next moves...`"
        elif normalized_name == "SaveMemory":
            fact = tool_input.get("fact", "")
            if fact:
                return f"**SaveMemory** `{fact[:40]}{'...' if len(fact) > 40 else ''}`"
            return "**SaveMemory** `storing information`"
        elif normalized_name == "Grep":
            pattern = (
                tool_input.get("pattern")
                or tool_input.get("query")
                or tool_input.get("search", "")
            )
            path = (
                tool_input.get("path")
                or tool_input.get("file")
                or tool_input.get("directory", "")
            )
            if pattern:
                if path:
                    display_path = get_display_path(path)
                    return f"**Search** `{pattern}` in `{display_path}`"
                return f"**Search** `{pattern}`"
            return "**Search** `pattern`"
        elif normalized_name == "Glob":
            if tool_name == "find_files":
                name = tool_input.get("name", "")
                if name:
                    return f"**Glob** `{name}`"
                return "**Glob** `finding files`"
            pattern = tool_input.get("pattern", "") or tool_input.get(
                "globPattern", ""
            )
            if pattern:
                return f"**Glob** `{pattern}`"
            return "**Glob** `pattern`"
        elif normalized_name == "Write":
            file_path = (
                tool_input.get("file_path")
                or tool_input.get("path")
                or tool_input.get("file", "")
            )
            if file_path:
                display_path = get_display_path(file_path)
                if len(display_path) > 40:
                    display_path = "…/" + "/".join(display_path.split("/")[-2:])
                return f"**Write** `{display_path}`"
            return "**Write** `file`"
        elif normalized_name == "MultiEdit":
            file_path = (
                tool_input.get("file_path")
                or tool_input.get("path")
                or tool_input.get("file", "")
            )
            if file_path:
                display_path = get_display_path(file_path)
                if len(display_path) > 40:
                    display_path = "…/" + "/".join(display_path.split("/")[-2:])
                return f"🔧 **MultiEdit** `{display_path}`"
            return "🔧 **MultiEdit** `file`"
        elif normalized_name == "LS":
            path = (
                tool_input.get("path")
                or tool_input.get("directory")
                or tool_input.get("dir", "")
            )
            if path:
                display_path = get_display_path(path)
                if len(display_path) > 40:
                    display_path = "…/" + display_path[-37:]
                return f"📁 **LS** `{display_path}`"
            return "📁 **LS** `directory`"
        elif normalized_name == "WebFetch":
            url = tool_input.get("url", "")
            if url:
                domain = (
                    url.split("//")[-1].split("/")[0]
                    if "//" in url
                    else url.split("/")[0]
                )
                return f"**WebFetch** [{domain}]({url})"
            return "**WebFetch** `url`"
        elif normalized_name == "WebSearch":
            query = tool_input.get("query") or tool_input.get("search_query", "")
            query = tool_input.get("query", "")
            if query:
                short_query = query[:40] + "..." if len(query) > 40 else query
                return f"**WebSearch** `{short_query}`"
            return "**WebSearch** `query`"
        elif normalized_name == "Task":
            description = tool_input.get("description", "")
            subagent_type = tool_input.get("subagent_type", "")
            if description and subagent_type:
                return (
                    f"🤖 **Task** `{subagent_type}`\n> "
                    f"{description[:50]}{'...' if len(description) > 50 else ''}"
                )
            elif description:
                return f"🤖 **Task** `{description[:40]}{'...' if len(description) > 40 else ''}`"
            return "🤖 **Task** `subtask`"
        elif normalized_name == "ExitPlanMode":
            return "✅ **ExitPlanMode** `planning complete`"
        elif normalized_name == "NotebookEdit":
            notebook_path = tool_input.get("notebook_path", "")
            if notebook_path:
                filename = notebook_path.split("/")[-1]
                return f"📓 **NotebookEdit** `{filename}`"
            return "📓 **NotebookEdit** `notebook`"
        elif normalized_name == "MCPTool" or tool_name == "mcp_tool_call":
            server = tool_input.get("server", "")
            tool_name_inner = tool_input.get("tool", "")
            if server and tool_name_inner:
                return f"🔧 **MCP** `{server}.{tool_name_inner}`"
            return "🔧 **MCP** `tool call`"
        elif tool_name == "exec_command":
            command = tool_input.get("command", "")
            if command:
                display_cmd = command[:40] + "..." if len(command) > 40 else command
                return f"⚡ **Exec** `{display_cmd}`"
            return "⚡ **Exec** `command`"
        else:
            return f"**{tool_name}** `executing...`"
```

</details>

---

#### 2. `adapters/*.py` -> `agents/adapters/*.rs`

- **역할**: 각 파일은 특정 에이전트와의 통신 **복잡성을 완전히 캡슐화**하는 어댑터입니다.
- **핵심 아이디어**:
  - **`CursorAgentCLI` (서브프로세스 + NDJSON)**:
    - **전략**: `cursor-agent --output-format stream-json`과 같이 서브프로세스를 실행하고, `stdout`을 줄 단위로 읽어 JSON을 파싱합니다.
    - **Rust Hint**: `tokio::process::Command`를 사용하여 자식 프로세스를 생성하고, `stdout` 파이프를 `tokio::io::BufReader`로 감싸 `lines()` 스트림을 만드세요. 각 라인을 `serde_json::from_str`로 파싱하여 `AgentEvent`로 변환합니다. `codex-rs/mcp-client/src/mcp_client.rs`의 `reader_handle` 구현이 좋은 참고자료입니다.
  - **`ClaudeCodeCLI` (SDK 사용)**:
    - **전략**: Anthropic의 공식 Python SDK를 사용합니다.
    - **Rust Hint**: `anthropic` 또는 유사한 Rust SDK 크레이트를 `Cargo.toml`에 추가합니다. SDK가 제공하는 스트리밍 API를 호출하고, 반환되는 SDK 고유의 이벤트 객체들(`ContentBlockDeltaEvent` 등)을 `AgentEvent`로 매핑하는 변환 로직을 구현합니다.
  - **`GeminiCLI` (stdio를 통한 JSON-RPC)**:
    - **전략**: CLI를 실행하고, 표준 입출력(stdio)을 통해 JSON-RPC 통신을 수행합니다.
    - **Rust Hint**: `tokio::process::Command`로 `stdin`과 `stdout`을 파이프로 설정합니다. `stdin`에는 `write_all`을, `stdout`에는 `BufReader`를 사용합니다. `codex-rs/mcp-client` 크레이트가 이 패턴의 완벽한 예시입니다.

<details>
<summary><b>Click to view raw content of `apps/api/app/services/cli/adapters/cursor_agent.py`</b></summary>

```python
"""Cursor Agent provider implementation.

Moved from unified_manager.py to a dedicated adapter module.
"""
from __future inport annotations

import asyncio
import json
import os
import uuid
from datetime import datetime
from typing import Any, AsyncGenerator, Callable, Dict, List, Optional

from app.models.messages import Message
from app.core.terminal_ui import ui

from ..base import BaseCLI, CLIType


class CursorAgentCLI(BaseCLI):
    """Cursor Agent CLI implementation with stream-json support and session continuity"""

    def __init__(self, db_session=None):
        super().__init__(CLIType.CURSOR)
        self.db_session = db_session
        self._session_store = {}  # Fallback for when db_session is not available

    async def check_availability(self) -> Dict[str, Any]:
        """Check if Cursor Agent CLI is available"""
        try:
            # Check if cursor-agent is installed and working
            result = await asyncio.create_subprocess_shell(
                "cursor-agent -h",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await result.communicate()

            if result.returncode != 0:
                return {
                    "available": False,
                    "configured": False,
                    "error": (
                        "Cursor Agent CLI not installed or not working.\n\nTo install:\n"
                        "1. Install Cursor: curl https://cursor.com/install -fsS | bash\n"
                        "2. Login to Cursor: cursor-agent login\n3. Try running your prompt again"
                    ),
                }

            # Check if help output contains expected content
            help_output = stdout.decode() + stderr.decode()
            if "cursor-agent" not in help_output.lower():
                return {
                    "available": False,
                    "configured": False,
                    "error": (
                        "Cursor Agent CLI not responding correctly.\n\nPlease try:\n"
                        "1. Reinstall: curl https://cursor.com/install -fsS | bash\n"
                        "2. Login: cursor-agent login\n3. Check installation: cursor-agent -h"
                    ),
                }

            return {
                "available": True,
                "configured": True,
                "models": self.get_supported_models(),
                "default_models": ["gpt-5", "sonnet-4"],
            }
        except Exception as e:
            return {
                "available": False,
                "configured": False,
                "error": (
                    f"Failed to check Cursor Agent: {str(e)}\n\nTo install:\n"
                    "1. Install Cursor: curl https://cursor.com/install -fsS | bash\n"
                    "2. Login: cursor-agent login"
                ),
            }

    def _handle_cursor_stream_json(
        self, event: Dict[str, Any], project_path: str, session_id: str
    ) -> Optional[Message]:
        """Handle Cursor stream-json format (NDJSON events) to be compatible with Claude Code CLI output"""
        event_type = event.get("type")

        if event_type == "system":
            # System initialization event
            return Message(
                id=str(uuid.uuid4()),
                project_id=project_path,
                role="system",
                message_type="system",
                content=f"🔧 Cursor Agent initialized (Model: {event.get('model', 'unknown')})",
                metadata_json={
                    "cli_type": self.cli_type.value,
                    "event_type": "system",
                    "cwd": event.get("cwd"),
                    "api_key_source": event.get("apiKeySource"),
                    "original_event": event,
                    "hidden_from_ui": True,  # Hide system init messages
                },
                session_id=session_id,
                created_at=datetime.utcnow(),
            )

        elif event_type == "user":
            # Cursor echoes back the user's prompt. Suppress it to avoid duplicates.
            return None

        elif event_type == "assistant":
            # Assistant response event (text delta)
            message_content = event.get("message", {}).get("content", [])
            content = ""

            if message_content and isinstance(message_content, list):
                for part in message_content:
                    if part.get("type") == "text":
                        content += part.get("text", "")

            if content:
                return Message(
                    id=str(uuid.uuid4()),
                    project_id=project_path,
                    role="assistant",
                    message_type="chat",
                    content=content,
                    metadata_json={
                        "cli_type": self.cli_type.value,
                        "event_type": "assistant",
                        "original_event": event,
                    },
                    session_id=session_id,
                    created_at=datetime.utcnow(),
                )

        elif event_type == "tool_call":
            subtype = event.get("subtype")
            tool_call_data = event.get("tool_call", {})
            if not tool_call_data:
                return None

            tool_name_raw = next(iter(tool_call_data), None)
            if not tool_name_raw:
                return None

            # Normalize tool name: lsToolCall -> ls
            tool_name = tool_name_raw.replace("ToolCall", "")

            if subtype == "started":
                tool_input = tool_call_data[tool_name_raw].get("args", {})
                summary = self._create_tool_summary(tool_name, tool_input)

                return Message(
                    id=str(uuid.uuid4()),
                    project_id=project_path,
                    role="assistant",
                    message_type="chat",
                    content=summary,
                    metadata_json={
                        "cli_type": self.cli_type.value,
                        "event_type": "tool_call_started",
                        "tool_name": tool_name,
                        "tool_input": tool_input,
                        "original_event": event,
                    },
                    session_id=session_id,
                    created_at=datetime.utcnow(),
                )

            elif subtype == "completed":
                result = tool_call_data[tool_name_raw].get("result", {})
                content = ""
                if "success" in result:
                    content = json.dumps(result["success"])
                elif "error" in result:
                    content = json.dumps(result["error"])

                return Message(
                    id=str(uuid.uuid4()),
                    project_id=project_path,
                    role="system",
                    message_type="tool_result",
                    content=content,
                    metadata_json={
                        "cli_type": self.cli_type.value,
                        "original_format": event,
                        "tool_name": tool_name,
                        "hidden_from_ui": True,
                    },
                    session_id=session_id,
                    created_at=datetime.utcnow(),
                )

        elif event_type == "result":
            # Final result event
            duration = event.get("duration_ms", 0)
            result_text = event.get("result", "")

            if result_text:
                return Message(
                    id=str(uuid.uuid4()),
                    project_id=project_path,
                    role="system",
                    message_type="system",
                    content=(
                        f"Execution completed in {duration}ms. Final result: {result_text}"
                    ),
                    metadata_json={
                        "cli_type": self.cli_type.value,
                        "event_type": "result",
                        "duration_ms": duration,
                        "original_event": event,
                        "hidden_from_ui": True,
                    },
                    session_id=session_id,
                    created_at=datetime.utcnow(),
                )

        return None

    async def _ensure_agent_md(self, project_path: str) -> None:
        """Ensure AGENTS.md exists in project repo with system prompt"""
        # Determine the repo path
        project_repo_path = os.path.join(project_path, "repo")
        if not os.path.exists(project_repo_path):
            project_repo_path = project_path

        agent_md_path = os.path.join(project_repo_path, "AGENTS.md")

        # Check if AGENTS.md already exists
        if os.path.exists(agent_md_path):
            print(f"📝 [Cursor] AGENTS.md already exists at: {agent_md_path}")
            return

        try:
            # Read system prompt from the source file using relative path
            current_file_dir = os.path.dirname(os.path.abspath(__file__))
            # this file is in: app/services/cli/adapters/
            # go up to app/: adapters -> cli -> services -> app
            app_dir = os.path.abspath(os.path.join(current_file_dir, "..", "..", ".."))
            system_prompt_path = os.path.join(app_dir, "prompt", "system-prompt.md")

            if os.path.exists(system_prompt_path):
                with open(system_prompt_path, "r", encoding="utf-8") as f:
                    system_prompt_content = f.read()

                # Write to AGENTS.md in the project repo
                with open(agent_md_path, "w", encoding="utf-8") as f:
                    f.write(system_prompt_content)

                print(f"📝 [Cursor] Created AGENTS.md at: {agent_md_path}")
            else:
                print(
                    f"⚠️ [Cursor] System prompt file not found at: {system_prompt_path}"
                )
        except Exception as e:
            print(f"❌ [Cursor] Failed to create AGENTS.md: {e}")

    async def execute_with_streaming(
        self,
        instruction: str,
        project_path: str,
        session_id: Optional[str] = None,
        log_callback: Optional[Callable[[str], Any]] = None,
        images: Optional[List[Dict[str, Any]]] = None,
        model: Optional[str] = None,
        is_initial_prompt: bool = False,
    ) -> AsyncGenerator[Message, None]:
        """Execute Cursor Agent CLI with stream-json format and session continuity"""
        # Ensure AGENTS.md exists for system prompt
        await self._ensure_agent_md(project_path)

        # Extract project ID from path (format: .../projects/{project_id}/repo)
        # We need the project_id, not "repo"
        path_parts = project_path.split("/")
        if "repo" in path_parts and len(path_parts) >= 2:
            # Get the folder before "repo"
            repo_index = path_parts.index("repo")
            if repo_index > 0:
                project_id = path_parts[repo_index - 1]
            else:
                project_id = path_parts[-1] if path_parts else project_path
        else:
            project_id = path_parts[-1] if path_parts else project_path

        stored_session_id = await self.get_session_id(project_id)

        cmd = [
            "cursor-agent",
            "--force",
            "-p",
            instruction,
            "--output-format",
            "stream-json",  # Use stream-json format
        ]

        # Add session resume if available (prefer stored session over parameter)
        active_session_id = stored_session_id or session_id
        if active_session_id:
            cmd.extend(["--resume", active_session_id])
            print(f"🔗 [Cursor] Resuming session: {active_session_id}")

        # Add API key if available
        if os.getenv("CURSOR_API_KEY"):
            cmd.extend(["--api-key", os.getenv("CURSOR_API_KEY")])

        # Add model - prioritize parameter over environment variable
        cli_model = self._get_cli_model_name(model) or os.getenv("CURSOR_MODEL")
        if cli_model:
            cmd.extend(["-m", cli_model])
            print(f"🔧 [Cursor] Using model: {cli_model}")

        project_repo_path = os.path.join(project_path, "repo")
        if not os.path.exists(project_repo_path):
            project_repo_path = project_path  # Fallback to project_path if repo subdir doesn't exist

        try:
            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=project_repo_path,
            )

            cursor_session_id = None
            assistant_message_buffer = ""
            result_received = False  # Track if we received result event

            async for line in process.stdout:
                line_str = line.decode().strip()
                if not line_str:
                    continue

                try:
                    # Parse NDJSON event
                    event = json.loads(line_str)

                    event_type = event.get("type")

                    # Priority: Extract session ID from type: "result" event (most reliable)
                    if event_type == "result" and not cursor_session_id:
                        print(f"🔍 [Cursor] Result event received: {event}")
                        session_id_from_result = event.get("session_id")
                        if session_id_from_result:
                            cursor_session_id = session_id_from_result
                            await self.set_session_id(project_id, cursor_session_id)
                            print(
                                f"💾 [Cursor] Session ID extracted from result event: {cursor_session_id}"
                            )

                        # Mark that we received result event
                        result_received = True

                    # Extract session ID from various event types
                    if not cursor_session_id:
                        # Try to extract session ID from any event that contains it
                        potential_session_id = (
                            event.get("sessionId")
                            or event.get("chatId")
                            or event.get("session_id")
                            or event.get("chat_id")
                            or event.get("threadId")
                            or event.get("thread_id")
                        )

                        # Also check in nested structures
                        if not potential_session_id and isinstance(
                            event.get("message"), dict
                        ):
                            potential_session_id = (
                                event["message"].get("sessionId")
                                or event["message"].get("chatId")
                                or event["message"].get("session_id")
                                or event["message"].get("chat_id")
                            )

                        if potential_session_id and potential_session_id != active_session_id:
                            cursor_session_id = potential_session_id
                            await self.set_session_id(project_id, cursor_session_id)
                            print(
                                f"💾 [Cursor] Updated session ID for project {project_id}: {cursor_session_id}"
                            )
                            print(f"   Previous: {active_session_id}")
                            print(f"   New: {cursor_session_id}")

                    # If we receive a non-assistant message, flush the buffer first
                    if event.get("type") != "assistant" and assistant_message_buffer:
                        yield Message(
                            id=str(uuid.uuid4()),
                            project_id=project_path,
                            role="assistant",
                            message_type="chat",
                            content=assistant_message_buffer,
                            metadata_json={
                                "cli_type": "cursor",
                                "event_type": "assistant_aggregated",
                            },
                            session_id=session_id,
                            created_at=datetime.utcnow(),
                        )
                        assistant_message_buffer = ""

                    # Process the event
                    message = self._handle_cursor_stream_json(
                        event, project_path, session_id
                    )

                    if message:
                        if message.role == "assistant" and message.message_type == "chat":
                            assistant_message_buffer += message.content
                        else:
                            if log_callback:
                                await log_callback(f"📝 [Cursor] {message.content}")
                            yield message

                    # ★ CRITICAL: Break after result event to end streaming
                    if result_received:
                        print(
                            f"🏁 [Cursor] Result event received, terminating stream early"
                        )
                        try:
                            process.terminate()
                            print(f"🔪 [Cursor] Process terminated")
                        except Exception as e:
                            print(f"⚠️ [Cursor] Failed to terminate process: {e}")
                        break

                except json.JSONDecodeError as e:
                    # Handle malformed JSON
                    print(f"⚠️ [Cursor] JSON decode error: {e}")
                    print(f"⚠️ [Cursor] Raw line: {line_str}")

                    # Still yield as raw output
                    message = Message(
                        id=str(uuid.uuid4()),
                        project_id=project_path,
                        role="assistant",
                        message_type="chat",
                        content=line_str,
                        metadata_json={
                            "cli_type": "cursor",
                            "raw_output": line_str,
                            "parse_error": str(e),
                        },
                        session_id=session_id,
                        created_at=datetime.utcnow(),
                    )
                    yield message

            # Flush any remaining content in the buffer
            if assistant_message_buffer:
                yield Message(
                    id=str(uuid.uuid4()),
                    project_id=project_path,
                    role="assistant",
                    message_type="chat",
                    content=assistant_message_buffer,
                    metadata_json={
                        "cli_type": "cursor",
                        "event_type": "assistant_aggregated",
                    },
                    session_id=session_id,
                    created_at=datetime.utcnow(),
                )

            await process.wait()

            # Log completion
            if cursor_session_id:
                print(f"✅ [Cursor] Session completed: {cursor_session_id}")

        except FileNotFoundError:
            error_msg = (
                "❌ Cursor Agent CLI not found. Please install with: curl https://cursor.com/install -fsS | bash"
            )
            yield Message(
                id=str(uuid.uuid4()),
                project_id=project_path,
                role="assistant",
                message_type="error",
                content=error_msg,
                metadata_json={"error": "cli_not_found", "cli_type": "cursor"},
                session_id=session_id,
                created_at=datetime.utcnow(),
            )
        except Exception as e:
            error_msg = f"❌ Cursor Agent execution failed: {str(e)}"
            yield Message(
                id=str(uuid.uuid4()),
                project_id=project_path,
                role="assistant",
                message_type="error",
                content=error_msg,
                metadata_json={
                    "error": "execution_failed",
                    "cli_type": "cursor",
                    "exception": str(e),
                },
                session_id=session_id,
                created_at=datetime.utcnow(),
            )

    async def get_session_id(self, project_id: str) -> Optional[str]:
        """Get stored session ID for project to enable session continuity"""
        if self.db_session:
            try:
                from app.models.projects import Project

                project = (
                    self.db_session.query(Project)
                    .filter(Project.id == project_id)
                    .first()
                )
                if project and project.active_cursor_session_id:
                    print(
                        f"💾 [Cursor] Retrieved session ID from DB: {project.active_cursor_session_id}"
                    )
                    return project.active_cursor_session_id
            except Exception as e:
                print(f"⚠️ [Cursor] Failed to get session ID from DB: {e}")

        # Fallback to in-memory storage
        return self._session_store.get(project_id)

    async def set_session_id(self, project_id: str, session_id: str) -> None:
        """Store session ID for project to enable session continuity"""
        # Store in database if available
        if self.db_session:
            try:
                from app.models.projects import Project

                project = (
                    self.db_session.query(Project)
                    .filter(Project.id == project_id)
                    .first()
                )
                if project:
                    project.active_cursor_session_id = session_id
                    self.db_session.commit()
                    print(
                        f"💾 [Cursor] Session ID saved to DB for project {project_id}: {session_id}"
                    )
                    return
                else:
                    print(f"⚠️ [Cursor] Project {project_id} not found in DB")
            except Exception as e:
                print(f"⚠️ [Cursor] Failed to save session ID to DB: {e}")
                import traceback

                traceback.print_exc()
        else:
            print(f"⚠️ [Cursor] No DB session available")

        # Fallback to in-memory storage
        self._session_store[project_id] = session_id
        print(
            f"💾 [Cursor] Session ID stored in memory for project {project_id}: {session_id}"
        )


__all__ = ["CursorAgentCLI"]
```

</details>

---

#### 3. `manager.py` (`UnifiedCLIManager`) -> `agents/manager.rs` (`AgentManager`)

- **역할**: `UnifiedCLIManager`는 모든 CLI 어댑터를 중앙에서 관리하고 오케스트레이션합니다. Rust의 `AgentManager`가 이 역할을 수행합니다.
- **핵심 아이디어**:
  1.  **초기화 (`__init__` -> `AgentManager::new`)**: `AgentManager`의 `new` 함수는 에이전트 설정 목록(`Vec<pk_protocol::agent_models::Agent>`)을 인자로 받습니다. 이 설정에 따라 지원하는 모든 어댑터 인스턴스를 생성하여 `HashMap<String, Arc<dyn Agent>>`에 저장합니다.
  2.  **실행 (`execute_instruction` -> `AgentManager::execute`)**: `AgentManager`는 `execute`와 같은 고수준 메서드를 제공해야 합니다. 이 메서드는 다음 로직을 수행합니다.
      - 요청된 에이전트 이름으로 `HashMap`에서 어댑터를 조회합니다.
      - `check_availability()`를 호출하여 사용 가능 여부를 확인합니다.
      - 사용 불가 시, 미리 정의된 **폴백 에이전트**로 재시도합니다. 이는 시스템 안정성을 위해 필수적입니다.
      - 사용 가능한 어댑터의 `execute()` 메서드를 호출하고 반환된 스트림을 그대로 반환합니다.

<details>
<summary><b>Click to view raw content of `apps/api/app/services/cli/manager.py`</b></summary>

```python
"""Unified CLI Manager implementation.

Moved from unified_manager.py to a dedicated module.
"""
from __future inport annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from app.core.terminal_ui import ui
from app.core.websocket.manager import manager as ws_manager
from app.models.messages import Message

from .base import CLIType
from .adapters import ClaudeCodeCLI, CursorAgentCLI, CodexCLI, QwenCLI, GeminiCLI


class UnifiedCLIManager:
    """Unified manager for all CLI implementations"""

    def __init__(
        self,
        project_id: str,
        project_path: str,
        session_id: str,
        conversation_id: str,
        db: Any,  # SQLAlchemy Session
    ):
        self.project_id = project_id
        self.project_path = project_path
        self.session_id = session_id
        self.conversation_id = conversation_id
        self.db = db

        # Initialize CLI adapters with database session
        self.cli_adapters = {
            CLIType.CLAUDE: ClaudeCodeCLI(),  # Use SDK implementation if available
            CLIType.CURSOR: CursorAgentCLI(db_session=db),
            CLIType.CODEX: CodexCLI(db_session=db),
            CLIType.QWEN: QwenCLI(db_session=db),
            CLIType.GEMINI: GeminiCLI(db_session=db),
        }

    async def _attempt_fallback(
        self,
        failed_cli: CLIType,
        instruction: str,
        images: Optional[List[Dict[str, Any]]],
        model: Optional[str],
        is_initial_prompt: bool,
    ) -> Optional[Dict[str, Any]]:
        fallback_type = CLIType.CLAUDE
        if failed_cli == fallback_type:
            return None

        fallback_cli = self.cli_adapters.get(fallback_type)
        if not fallback_cli:
            ui.warning("Fallback CLI Claude not configured", "CLI")
            return None

        status = await fallback_cli.check_availability()
        if not status.get("available") or not status.get("configured"):
            ui.error(
                f"Fallback CLI {fallback_type.value} unavailable: {status.get('error', 'unknown error')}",
                "CLI",
            )
            return None

        ui.warning(
            f"CLI {failed_cli.value} unavailable; falling back to {fallback_type.value}",
            "CLI",
        )

        try:
            result = await self._execute_with_cli(
                fallback_cli, instruction, images, model, is_initial_prompt
            )
            result["fallback_used"] = True
            result["fallback_from"] = failed_cli.value
            return result
        except Exception as error:
            ui.error(
                f"Fallback CLI {fallback_type.value} failed: {error}",
                "CLI",
            )
            return None

    async def execute_instruction(
        self,
        instruction: str,
        cli_type: CLIType,
        fallback_enabled: bool = True,  # Kept for backward compatibility but not used
        images: Optional[List[Dict[str, Any]]] = None,
        model: Optional[str] = None,
        is_initial_prompt: bool = False,
    ) -> Dict[str, Any]:
        """Execute instruction with specified CLI"""

        # Try the specified CLI
        if cli_type in self.cli_adapters:
            cli = self.cli_adapters[cli_type]

            # Check if CLI is available
            status = await cli.check_availability()
            if status.get("available") and status.get("configured"):
                try:
                    return await self._execute_with_cli(
                        cli, instruction, images, model, is_initial_prompt
                    )
                except Exception as e:
                    ui.error(f"CLI {cli_type.value} failed: {e}", "CLI")
                    if fallback_enabled:
                        fallback_result = await self._attempt_fallback(
                            cli_type, instruction, images, model, is_initial_prompt
                        )
                        if fallback_result:
                            return fallback_result
                    return {
                        "success": False,
                        "error": str(e),
                        "cli_attempted": cli_type.value,
                    }
            else:
                ui.warning(
                    f"CLI {cli_type.value} unavailable: {status.get('error', 'CLI not available')}",
                    "CLI",
                )
                if fallback_enabled:
                    fallback_result = await self._attempt_fallback(
                        cli_type, instruction, images, model, is_initial_prompt
                    )
                    if fallback_result:
                        return fallback_result
                return {
                    "success": False,
                    "error": status.get("error", "CLI not available"),
                    "cli_attempted": cli_type.value,
                }

        if fallback_enabled:
            fallback_result = await self._attempt_fallback(
                cli_type, instruction, images, model, is_initial_prompt
            )
            if fallback_result:
                return fallback_result

        return {
            "success": False,
            "error": f"CLI type {cli_type.value} not implemented",
            "cli_attempted": cli_type.value,
        }

    async def _execute_with_cli(
        self,
        cli,
        instruction: str,
        images: Optional[List[Dict[str, Any]]],
        model: Optional[str] = None,
        is_initial_prompt: bool = False,
    ) -> Dict[str, Any]:
        """Execute instruction with a specific CLI"""

        ui.info(f"Starting {cli.cli_type.value} execution", "CLI")
        if model:
            ui.debug(f"Using model: {model}", "CLI")

        messages_collected: List[Message] = []
        has_changes = False
        files_modified: set[str] = set()
        has_error = False  # Track if any error occurred
        result_success: Optional[bool] = None  # Track result event success status

        # Log callback
        async def log_callback(message: str):
            # CLI output logs are now only printed to console, not sent to UI
            pass

        async for message in cli.execute_with_streaming(
            instruction=instruction,
            project_path=self.project_path,
            session_id=self.session_id,
            log_callback=log_callback,
            images=images,
            model=model,
            is_initial_prompt=is_initial_prompt,
        ):
            # Check for error messages or result status
            if message.message_type == "error":
                has_error = True
                ui.error(f"CLI error detected: {message.content[:100]}", "CLI")

            if message.metadata_json:
                files = message.metadata_json.get("files_modified")
                if isinstance(files, (list, tuple, set)):
                    files_modified.update(str(f) for f in files)

            # Check for Cursor result event (stored in metadata)
            if message.metadata_json:
                event_type = message.metadata_json.get("event_type")
                original_event = message.metadata_json.get("original_event", {})

                if event_type == "result" or original_event.get("type") == "result":
                    # Cursor sends result event with success/error status
                    is_error = original_event.get("is_error", False)
                    subtype = original_event.get("subtype", "")

                    # DEBUG: Log the complete result event structure
                    ui.info(f"🔍 [Cursor] Result event received:", "DEBUG")
                    ui.info(f"   Full event: {original_event}", "DEBUG")
                    ui.info(f"   is_error: {is_error}", "DEBUG")
                    ui.info(f"   subtype: '{subtype}'", "DEBUG")
                    ui.info(f"   has event.result: {'result' in original_event}", "DEBUG")
                    ui.info(f"   has event.status: {'status' in original_event}", "DEBUG")
                    ui.info(f"   has event.success: {'success' in original_event}", "DEBUG")

                    if is_error or subtype == "error":
                        has_error = True
                        result_success = False
                        ui.error(
                            f"Cursor result: error (is_error={is_error}, subtype='{subtype}')",
                            "CLI",
                        )
                    elif subtype == "success":
                        result_success = True
                        ui.success(
                            f"Cursor result: success (subtype='{subtype}')", "CLI"
                        )
                    else:
                        # Handle case where subtype is not "success" but execution was successful
                        ui.warning(
                            f"Cursor result: no explicit success subtype (subtype='{subtype}', is_error={is_error})",
                            "CLI",
                        )
                        # If there's no error indication, assume success
                        if not is_error:
                            result_success = True
                            ui.success(
                                f"Cursor result: assuming success (no error detected)", "CLI"
                            )

            # Save message to database
            message.project_id = self.project_id
            message.conversation_id = self.conversation_id
            self.db.add(message)
            self.db.commit()

            messages_collected.append(message)

            # Check if message should be hidden from UI
            should_hide = (
                message.metadata_json and message.metadata_json.get("hidden_from_ui", False)
            )

            # Send message via WebSocket only if not hidden
            if not should_hide:
                ws_message = {
                    "type": "message",
                    "data": {
                        "id": message.id,
                        "role": message.role,
                        "message_type": message.message_type,
                        "content": message.content,
                        "metadata": message.metadata_json,
                        "parent_message_id": getattr(message, "parent_message_id", None),
                        "session_id": message.session_id,
                        "conversation_id": self.conversation_id,
                        "created_at": message.created_at.isoformat(),
                    },
                    "timestamp": message.created_at.isoformat(),
                }
                try:
                    await ws_manager.send_message(self.project_id, ws_message)
                except Exception as e:
                    ui.error(f"WebSocket send failed: {e}", "Message")

            # Check if changes were made
            if message.metadata_json and "changes_made" in message.metadata_json:
                has_changes = True

        # Determine final success status
        # For Cursor: check result_success if available, otherwise check has_error
        # For others: check has_error
        ui.info(
            f"🔍 Final success determination: cli_type={cli.cli_type}, result_success={result_success}, has_error={has_error}",
            "CLI",
        )

        if cli.cli_type == CLIType.CURSOR and result_success is not None:
            success = result_success
            ui.info(f"Using Cursor result_success: {result_success}", "CLI")
        else:
            success = not has_error
            ui.info(f"Using has_error logic: not {has_error} = {success}", "CLI")

        if success:
            ui.success(
                f"Streaming completed successfully. Total messages: {len(messages_collected)}",
                "CLI",
            )
        else:
            ui.error(
                f"Streaming completed with errors. Total messages: {len(messages_collected)}",
                "CLI",
            )

        return {
            "success": success,
            "cli_used": cli.cli_type.value,
            "has_changes": has_changes,
            "message": f"{'Successfully' if success else 'Failed to'} execute with {cli.cli_type.value}",
            "error": "Execution failed" if not success else None,
            "messages_count": len(messages_collected),
        }

        # End _execute_with_cli

    async def check_cli_status(
        self, cli_type: CLIType, selected_model: Optional[str] = None
    ) -> Dict[str, Any]:
        """Check status of a specific CLI"""
        if cli_type in self.cli_adapters:
            status = await self.cli_adapters[cli_type].check_availability()

            # Add model validation if model is specified
            if selected_model and status.get("available"):
                cli = self.cli_adapters[cli_type]
                if not cli.is_model_supported(selected_model):
                    status[
                        "model_warning"
                    ] = f"Model '{selected_model}' may not be supported by {cli_type.value}"
                    status["suggested_models"] = status.get("default_models", [])
                else:
                    status["selected_model"] = selected_model
                    status["model_valid"] = True

            return status
        return {
            "available": False,
            "configured": False,
            "error": f"CLI type {cli_type.value} not implemented",
        }


__all__ = ["UnifiedCLIManager"]
```

</details>

---

**Guidelines & Conventions**:

- **MockAgent 우선 구현**: TDD를 위해, 실제 API를 호출하지 않고 미리 정의된 `AgentEvent` 스트림을 반환하는 `MockAgent`를 가장 먼저 구현하세요. 이는 `PipelineEngine`을 에이전트 구현과 독립적으로 테스트할 수 있게 해줍니다.
- **스레드 안전성**: `AgentManager`는 `Arc<AgentManager>`로 감싸져 여러 비동기 태스크에서 공유될 것이므로, 내부 상태는 스레드 안전해야 합니다. 에이전트 맵은 `Arc<dyn Agent>`를 사용하여 공유 소유권을 명확히 합니다.
- **오류 처리**: `Agent` 트레이트의 메서드들은 `Result<..., AgentError>`를 반환해야 합니다. `AgentError`는 `thiserror`를 사용해 구체적인 오류 타입(예: `CliNotAvailable`, `ApiError`, `StreamParseError`)을 정의하세요.

**Acceptance Tests (TDD Process)**:

1.  **RED**: `tests/agent_manager.rs`를 생성합니다. `AgentManager::new`에 `pk_protocol::agent_models::Agent` 설정 목록을 전달하는 테스트를 작성합니다. `manager.get_agent("mock-agent")`를 호출하여 `Some(agent)`를 반환하는지, 그리고 `get_agent("non-existent-agent")`가 `None`을 반환하는지 검증하는 테스트를 추가합니다. `Agent` 트레이트가 없으므로 컴파일이 실패합니다.
2.  **GREEN**: `agents/base.rs`에 `Agent` 트레이트를 정의합니다. `agents/adapters/mock_agent.rs`에 `MockAgent`를 구현합니다. `MockAgent`의 `execute`는 `tokio_stream::iter`를 사용해 미리 정의된 `Vec<Result<AgentEvent, AgentError>>`를 스트림으로 반환하도록 합니다. `AgentManager`를 구현하여 `new`에서 `MockAgent`를 인스턴스화하고 `get_agent`에서 반환하도록 하여 테스트를 통과시킵니다.
3.  **REFACTOR**: `AgentManager`에 폴백 로직을 추가하고, 이를 검증하는 테스트 케이스를 작성합니다. 예를 들어, `check_availability`가 `false`를 반환하는 `FailingAgent`를 만들고, `AgentManager`가 폴백 에이전트인 `MockAgent`를 대신 실행하는지 확인합니다.
