//! Test fixtures for creating sample configurations and test data.

use pk_protocol::agent_models::Agent;
use pk_protocol::pipeline_models::MasterAgentConfig;
use pk_protocol::pipeline_models::Pipeline;
use pk_protocol::pipeline_models::ProcessStep;
use pk_protocol::process_models::Process;
use pk_protocol::process_models::ProcessStatus;
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;

/// Create a temporary project directory with .pipeline-kit configuration.
///
/// This creates a complete test environment with:
/// - `.pipeline-kit/pipelines/` directory
/// - `.pipeline-kit/agents/` directory
/// - Sample pipeline and agent configuration files
///
/// Returns a TempDir that must be kept alive for the test duration.
#[allow(dead_code)]
pub fn create_test_project() -> std::io::Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();

    // Create directory structure
    std::fs::create_dir_all(root.join(".pipeline-kit/pipelines"))?;
    std::fs::create_dir_all(root.join(".pipeline-kit/agents"))?;

    // Create a sample pipeline YAML
    let pipeline_yaml = r#"
name: test-pipeline
required-reference-file:
  1: "docs/requirements.md"
output-file:
  1: "src/output.rs"
master:
  model: "claude-sonnet-4"
  system-prompt: "You are a test orchestrator"
  process:
    - "agent-1"
    - "HUMAN_REVIEW"
    - "agent-2"
sub-agents:
  - "agent-1"
  - "agent-2"
"#;
    std::fs::write(
        root.join(".pipeline-kit/pipelines/test.yaml"),
        pipeline_yaml,
    )?;

    // Create sample agent markdown files
    let agent1_md = r#"---
name: agent-1
description: Test agent 1
model: claude-sonnet-4
color: blue
---
You are a helpful test agent."#;

    let agent2_md = r#"---
name: agent-2
description: Test agent 2
model: claude-sonnet-4
color: green
---
You are another helpful test agent."#;

    std::fs::write(root.join(".pipeline-kit/agents/agent-1.md"), agent1_md)?;
    std::fs::write(root.join(".pipeline-kit/agents/agent-2.md"), agent2_md)?;

    // Create dummy reference files
    std::fs::create_dir_all(root.join("docs"))?;
    std::fs::write(
        root.join("docs/requirements.md"),
        "# Test Requirements\nThis is a test.",
    )?;

    Ok(temp_dir)
}

/// Create a test Agent configuration.
pub fn create_test_agent(name: &str) -> Agent {
    Agent {
        name: name.to_string(),
        description: format!("Test agent {}", name),
        model: "test-model".to_string(),
        color: "blue".to_string(),
        system_prompt: "Test prompt".to_string(),
    }
}

/// Create a test Pipeline configuration.
pub fn create_test_pipeline(name: &str, steps: Vec<ProcessStep>) -> Pipeline {
    Pipeline {
        name: name.to_string(),
        required_reference_file: HashMap::new(),
        output_file: HashMap::new(),
        master: MasterAgentConfig {
            model: "test-model".to_string(),
            system_prompt: "Test orchestration".to_string(),
            process: steps.clone(),
        },
        sub_agents: steps
            .iter()
            .filter_map(|step| match step {
                ProcessStep::Agent(name) => Some(name.clone()),
                _ => None,
            })
            .collect(),
    }
}

/// Create a simple pipeline with sequential agent steps.
pub fn create_simple_pipeline(name: &str, num_agents: usize) -> Pipeline {
    let steps = (1..=num_agents)
        .map(|i| ProcessStep::Agent(format!("agent-{}", i)))
        .collect();
    create_test_pipeline(name, steps)
}

/// Create a pipeline with HUMAN_REVIEW steps.
pub fn create_pipeline_with_human_review(name: &str) -> Pipeline {
    let steps = vec![
        ProcessStep::Agent("agent-1".to_string()),
        ProcessStep::HumanReview(pk_protocol::pipeline_models::HumanReviewMarker),
        ProcessStep::Agent("agent-2".to_string()),
    ];
    create_test_pipeline(name, steps)
}

/// Create a test Process instance.
pub fn create_test_process(pipeline_name: &str, status: ProcessStatus) -> Process {
    Process {
        id: Uuid::new_v4(),
        pipeline_name: pipeline_name.to_string(),
        status,
        current_step_index: 0,
        logs: Vec::new(),
        started_at: chrono::Utc::now(),
        completed_at: None,
        resume_notifier: std::sync::Arc::new(tokio::sync::Notify::new()),
    }
}

/// Create a running process with some log entries.
#[allow(dead_code)]
pub fn create_running_process(pipeline_name: &str) -> Process {
    let mut process = create_test_process(pipeline_name, ProcessStatus::Running);
    process.logs = vec![
        "Process started".to_string(),
        "Agent 1 executing".to_string(),
    ];
    process
}

/// Create a paused process.
#[allow(dead_code)]
pub fn create_paused_process(pipeline_name: &str) -> Process {
    create_test_process(pipeline_name, ProcessStatus::Paused)
}

/// Create a completed process.
#[allow(dead_code)]
pub fn create_completed_process(pipeline_name: &str) -> Process {
    let mut process = create_test_process(pipeline_name, ProcessStatus::Completed);
    process.completed_at = Some(chrono::Utc::now());
    process
}
