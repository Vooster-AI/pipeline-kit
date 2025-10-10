use pk_protocol::*;
use serde_json;
use serde_yaml;

#[test]
fn test_pipeline_deserialization_from_yaml() {
    // Sample YAML structure based on the spec
    let yaml_str = r#"
name: test-pipeline
required-reference-file:
  1: "docs/requirements.md"
  2: "docs/design.md"
output-file:
  1: "src/output.rs"
  2: "tests/output_test.rs"
master:
  model: "claude-sonnet-4"
  system-prompt: "You are a helpful pipeline orchestrator"
  process:
    - "agent-1"
    - "agent-2"
    - "HUMAN_REVIEW"
    - "agent-3"
sub-agents:
  - "agent-1"
  - "agent-2"
  - "agent-3"
"#;

    let pipeline: Pipeline = serde_yaml::from_str(yaml_str).expect("Failed to deserialize Pipeline");

    assert_eq!(pipeline.name, "test-pipeline");
    assert_eq!(pipeline.required_reference_file.len(), 2);
    assert_eq!(pipeline.output_file.len(), 2);
    assert_eq!(pipeline.master.model, "claude-sonnet-4");
    assert_eq!(pipeline.master.process.len(), 4);
    assert_eq!(pipeline.sub_agents.len(), 3);

    // Verify ProcessStep variants
    assert_eq!(pipeline.master.process[0], ProcessStep::Agent("agent-1".to_string()));
    assert!(matches!(pipeline.master.process[2], ProcessStep::HumanReview(_)));
}

#[test]
fn test_agent_serialization() {
    let agent = Agent {
        name: "test-agent".to_string(),
        description: "A test agent".to_string(),
        model: "claude-sonnet-4".to_string(),
        color: "blue".to_string(),
        system_prompt: "Be helpful".to_string(),
    };

    let json = serde_json::to_string(&agent).expect("Failed to serialize Agent");
    let deserialized: Agent = serde_json::from_str(&json).expect("Failed to deserialize Agent");

    assert_eq!(deserialized.name, agent.name);
    assert_eq!(deserialized.description, agent.description);
    assert_eq!(deserialized.model, agent.model);
    assert_eq!(deserialized.color, agent.color);
    // system_prompt is skipped in serialization
    assert_eq!(deserialized.system_prompt, "");
}

#[test]
fn test_process_status_serialization() {
    let status = ProcessStatus::Running;
    let json = serde_json::to_value(&status).expect("Failed to serialize ProcessStatus");

    assert_eq!(json, "RUNNING");

    let deserialized: ProcessStatus = serde_json::from_value(json).expect("Failed to deserialize ProcessStatus");
    assert_eq!(deserialized, ProcessStatus::Running);
}

#[test]
fn test_process_serialization() {
    use uuid::Uuid;

    let process_id = Uuid::new_v4();
    let process = Process {
        id: process_id,
        pipeline_name: "test-pipeline".to_string(),
        status: ProcessStatus::Pending,
        current_step_index: 0,
        started_at: chrono::Utc::now(),
        completed_at: None,
        logs: vec!["Log entry 1".to_string(), "Log entry 2".to_string()],
    };

    let json = serde_json::to_string(&process).expect("Failed to serialize Process");
    let deserialized: Process = serde_json::from_str(&json).expect("Failed to deserialize Process");

    assert_eq!(deserialized.id, process.id);
    assert_eq!(deserialized.pipeline_name, process.pipeline_name);
    assert_eq!(deserialized.status, process.status);
    assert_eq!(deserialized.current_step_index, process.current_step_index);
    assert_eq!(deserialized.logs.len(), 2);
}

#[test]
fn test_global_config_serialization() {
    let config = GlobalConfig { git: true };

    let json = serde_json::to_string(&config).expect("Failed to serialize GlobalConfig");
    let deserialized: GlobalConfig = serde_json::from_str(&json).expect("Failed to deserialize GlobalConfig");

    assert_eq!(deserialized.git, config.git);
}

#[test]
fn test_op_enum_serialization() {
    use std::path::PathBuf;
    use uuid::Uuid;

    let op = Op::StartPipeline {
        name: "test-pipeline".to_string(),
        reference_file: Some(PathBuf::from("test.md")),
    };

    let json = serde_json::to_value(&op).expect("Failed to serialize Op");
    assert_eq!(json["type"], "startPipeline");
    assert!(json["payload"].is_object());

    let deserialized: Op = serde_json::from_value(json).expect("Failed to deserialize Op");
    match deserialized {
        Op::StartPipeline { name, reference_file } => {
            assert_eq!(name, "test-pipeline");
            assert!(reference_file.is_some());
        }
        _ => panic!("Wrong variant"),
    }

    let pause_op = Op::PauseProcess { process_id: Uuid::new_v4() };
    let json = serde_json::to_value(&pause_op).expect("Failed to serialize Op::PauseProcess");
    assert_eq!(json["type"], "pauseProcess");
}

#[test]
fn test_event_enum_serialization() {
    use uuid::Uuid;

    let event = Event::ProcessStarted {
        process_id: Uuid::new_v4(),
        pipeline_name: "test-pipeline".to_string(),
    };

    let json = serde_json::to_value(&event).expect("Failed to serialize Event");
    assert_eq!(json["type"], "processStarted");
    assert!(json["payload"].is_object());

    let status_update = Event::ProcessStatusUpdate {
        process_id: Uuid::new_v4(),
        status: ProcessStatus::Running,
        step_index: 1,
    };
    let json = serde_json::to_value(&status_update).expect("Failed to serialize Event");
    assert_eq!(json["type"], "processStatusUpdate");
}

#[test]
fn test_process_step_untagged_serialization() {
    use pk_protocol::HumanReviewMarker;

    // Test Agent variant
    let agent_step = ProcessStep::Agent("my-agent".to_string());
    let json = serde_json::to_value(&agent_step).expect("Failed to serialize ProcessStep::Agent");
    assert_eq!(json, "my-agent");

    // Test HumanReview variant
    let human_review_step = ProcessStep::HumanReview(HumanReviewMarker);
    let json = serde_json::to_value(&human_review_step).expect("Failed to serialize ProcessStep::HumanReview");
    assert_eq!(json, "HUMAN_REVIEW");

    // Test deserialization of HUMAN_REVIEW
    let deserialized: ProcessStep = serde_json::from_str("\"HUMAN_REVIEW\"").expect("Failed to deserialize HUMAN_REVIEW");
    assert!(matches!(deserialized, ProcessStep::HumanReview(_)));
}
