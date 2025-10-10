//! Pipeline configuration models for `.pipeline-kit/pipelines/*.yaml`.
//!
//! This module defines the structure of pipeline definition files that
//! orchestrate multi-agent workflows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// Represents a single step in a pipeline's process.
///
/// A process step can be either:
/// - An agent execution (referenced by agent name)
/// - A special command like HUMAN_REVIEW that pauses for manual intervention
///
/// The enum uses `#[serde(untagged)]` to allow flexible YAML syntax where
/// steps can be simple strings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, TS)]
#[serde(untagged)]
pub enum ProcessStep {
    /// A special step that pauses the pipeline for human review.
    ///
    /// When encountered, the pipeline execution stops and waits for
    /// manual approval before proceeding to the next step.
    HumanReview(HumanReviewMarker),

    /// Execute a specific agent by name.
    ///
    /// The string should match the `name` field of an agent defined
    /// in `.pipeline-kit/agents/*.md`.
    Agent(String),
}

/// Marker type for HUMAN_REVIEW step that deserializes from the literal string "HUMAN_REVIEW".
///
/// This type ensures that only the exact string "HUMAN_REVIEW" is accepted
/// when deserializing a human review step from YAML.
#[derive(Debug, Clone, PartialEq, Eq, TS)]
pub struct HumanReviewMarker;

impl<'de> Deserialize<'de> for HumanReviewMarker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "HUMAN_REVIEW" {
            Ok(HumanReviewMarker)
        } else {
            Err(serde::de::Error::custom(format!(
                "expected HUMAN_REVIEW, got {}",
                s
            )))
        }
    }
}

impl Serialize for HumanReviewMarker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("HUMAN_REVIEW")
    }
}

/// Defines the configuration for the master agent orchestrating the pipeline.
///
/// The master agent is responsible for coordinating the execution of all
/// sub-agents and managing the overall pipeline flow.
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "kebab-case")]
pub struct MasterAgentConfig {
    /// AI model to use for the master agent.
    pub model: String,

    /// System prompt defining the master agent's orchestration behavior.
    pub system_prompt: String,

    /// Sequential list of process steps to execute.
    ///
    /// Each step is either an agent name or a special command like HUMAN_REVIEW.
    pub process: Vec<ProcessStep>,
}

/// Defines a full pipeline, including its agents and process flow.
///
/// Pipelines are defined in `.pipeline-kit/pipelines/*.yaml` files and specify
/// the complete workflow for a multi-agent task.
///
/// # Example
///
/// ```yaml
/// name: code-review-pipeline
/// required-reference-file:
///   1: "docs/coding-standards.md"
///   2: "docs/security-checklist.md"
/// output-file:
///   1: "review-report.md"
/// master:
///   model: "claude-sonnet-4"
///   system-prompt: "Orchestrate a thorough code review process"
///   process:
///     - "static-analyzer"
///     - "security-reviewer"
///     - "HUMAN_REVIEW"
///     - "final-reporter"
/// sub-agents:
///   - "static-analyzer"
///   - "security-reviewer"
///   - "final-reporter"
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "kebab-case")]
pub struct Pipeline {
    /// Unique name identifying this pipeline.
    pub name: String,

    /// Map of step index to required reference file paths.
    ///
    /// These files provide context to agents at specific pipeline steps.
    /// The key is the 1-based step index.
    #[serde(default)]
    pub required_reference_file: HashMap<u32, String>,

    /// Map of step index to output file paths.
    ///
    /// Specifies where agents should write their output at each step.
    /// The key is the 1-based step index.
    #[serde(default)]
    pub output_file: HashMap<u32, String>,

    /// Configuration for the master orchestrator agent.
    pub master: MasterAgentConfig,

    /// List of sub-agent names that can be used in the process.
    ///
    /// All agent names referenced in `master.process` should be listed here.
    pub sub_agents: Vec<String>,
}
