//! Agent type enumeration for determining which adapter to use.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentType {
    Claude,
    Cursor,
    Gemini,
    Codex,
    Qwen,
    Mock,
}

impl AgentType {
    /// Infer the agent type from a model name.
    ///
    /// # Arguments
    ///
    /// * `model` - The model name from the agent configuration
    ///
    /// # Returns
    ///
    /// The inferred `AgentType`. Defaults to `Mock` if the model doesn't match any known pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use pk_core::agents::AgentType;
    ///
    /// assert_eq!(AgentType::from_model_name("claude-sonnet-4.5"), AgentType::Claude);
    /// assert_eq!(AgentType::from_model_name("gpt-5"), AgentType::Cursor);
    /// assert_eq!(AgentType::from_model_name("gemini-2.5-pro"), AgentType::Gemini);
    /// assert_eq!(AgentType::from_model_name("unknown-model"), AgentType::Mock);
    /// ```
    pub fn from_model_name(model: &str) -> Self {
        let model_lower = model.to_lowercase();

        if model_lower.contains("claude") {
            Self::Claude
        } else if model_lower.starts_with("gpt")
            || model_lower.contains("cursor")
            || model_lower.starts_with("sonnet")
            || model_lower.starts_with("opus") {
            // Cursor uses GPT models and also has cursor-specific names
            // Also handles sonnet-4.5, opus-4.1 which are Cursor shorthand
            Self::Cursor
        } else if model_lower.contains("gemini") {
            Self::Gemini
        } else if model_lower.contains("codex") {
            Self::Codex
        } else if model_lower.contains("qwen") {
            Self::Qwen
        } else {
            // Default to Mock for unknown models
            Self::Mock
        }
    }

    /// Get a human-readable name for the agent type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Cursor => "Cursor",
            Self::Gemini => "Gemini",
            Self::Codex => "Codex",
            Self::Qwen => "Qwen",
            Self::Mock => "Mock",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_model_name_claude() {
        assert_eq!(AgentType::from_model_name("claude-sonnet-4.5"), AgentType::Claude);
        assert_eq!(AgentType::from_model_name("claude-opus-4.1"), AgentType::Claude);
        assert_eq!(AgentType::from_model_name("Claude-Haiku-3.5"), AgentType::Claude);
        assert_eq!(AgentType::from_model_name("claude"), AgentType::Claude);
    }

    #[test]
    fn test_from_model_name_cursor() {
        assert_eq!(AgentType::from_model_name("gpt-5"), AgentType::Cursor);
        assert_eq!(AgentType::from_model_name("gpt-4o"), AgentType::Cursor);
        assert_eq!(AgentType::from_model_name("cursor-model"), AgentType::Cursor);
        assert_eq!(AgentType::from_model_name("sonnet-4.5"), AgentType::Cursor);
        assert_eq!(AgentType::from_model_name("opus-4.1"), AgentType::Cursor);
    }

    #[test]
    fn test_from_model_name_gemini() {
        assert_eq!(AgentType::from_model_name("gemini-2.5-pro"), AgentType::Gemini);
        assert_eq!(AgentType::from_model_name("gemini-2.5-flash"), AgentType::Gemini);
        assert_eq!(AgentType::from_model_name("Gemini-Pro"), AgentType::Gemini);
    }

    #[test]
    fn test_from_model_name_codex() {
        assert_eq!(AgentType::from_model_name("codex-model"), AgentType::Codex);
        assert_eq!(AgentType::from_model_name("openai-codex"), AgentType::Codex);
    }

    #[test]
    fn test_from_model_name_qwen() {
        assert_eq!(AgentType::from_model_name("qwen-coder"), AgentType::Qwen);
        assert_eq!(AgentType::from_model_name("Qwen3-Coder-Plus"), AgentType::Qwen);
    }

    #[test]
    fn test_from_model_name_unknown() {
        assert_eq!(AgentType::from_model_name("unknown-model"), AgentType::Mock);
        assert_eq!(AgentType::from_model_name(""), AgentType::Mock);
        assert_eq!(AgentType::from_model_name("random-string"), AgentType::Mock);
    }

    #[test]
    fn test_agent_type_name() {
        assert_eq!(AgentType::Claude.name(), "Claude");
        assert_eq!(AgentType::Cursor.name(), "Cursor");
        assert_eq!(AgentType::Gemini.name(), "Gemini");
        assert_eq!(AgentType::Codex.name(), "Codex");
        assert_eq!(AgentType::Qwen.name(), "Qwen");
        assert_eq!(AgentType::Mock.name(), "Mock");
    }

    #[test]
    fn test_agent_type_eq() {
        assert_eq!(AgentType::Claude, AgentType::Claude);
        assert_ne!(AgentType::Claude, AgentType::Cursor);
    }
}
