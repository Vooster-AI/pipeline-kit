//! Agent adapter implementations.

mod claude_adapter;
mod codex_adapter;
mod cursor_adapter;
mod gemini_adapter;
pub mod mock_agent;

pub use claude_adapter::ClaudeAdapter;
pub use codex_adapter::CodexAdapter;
pub use cursor_adapter::CursorAdapter;
pub use gemini_adapter::GeminiAdapter;
pub use mock_agent::MockAgent;
