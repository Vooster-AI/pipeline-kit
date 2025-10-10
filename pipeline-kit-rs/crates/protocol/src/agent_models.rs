//! Agent configuration models for `.pipeline-kit/agents/*.md`.
//!
//! This module defines the structure of agent configuration files.
//! Agents are defined as Markdown files with YAML front matter.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Represents an AI agent's configuration and system prompt.
///
/// Agents are defined in `.pipeline-kit/agents/*.md` files with YAML front matter
/// containing metadata, and the file body containing the system prompt.
///
/// # Example
///
/// ```markdown
/// ---
/// name: code-reviewer
/// description: Reviews code for quality and best practices
/// model: claude-sonnet-4
/// color: blue
/// ---
///
/// You are an expert code reviewer. Analyze code for:
/// - Correctness
/// - Performance
/// - Security
/// - Best practices
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct Agent {
    /// Unique identifier for this agent.
    ///
    /// Used to reference the agent in pipeline process steps.
    pub name: String,

    /// Human-readable description of the agent's purpose.
    pub description: String,

    /// AI model to use for this agent (e.g., "claude-sonnet-4", "gpt-4").
    pub model: String,

    /// UI color hint for displaying this agent in the TUI.
    ///
    /// Defaults to empty string if not specified.
    #[serde(default)]
    pub color: String,

    /// The main content of the .md file, not part of the front matter.
    ///
    /// This contains the system prompt that defines the agent's behavior.
    /// Note: This field is skipped during JSON serialization as it's not
    /// part of the front matter metadata.
    #[serde(skip)]
    pub system_prompt: String,
}
