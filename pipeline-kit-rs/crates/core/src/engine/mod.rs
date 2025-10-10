//! Pipeline execution engine.
//!
//! The PipelineEngine is responsible for executing pipeline steps sequentially,
//! managing agent interactions, and coordinating process state transitions.

use crate::agents::base::{AgentEvent, ExecutionContext};
use crate::agents::manager::AgentManager;
use crate::state::process::{
    advance_step, complete_process, create_process, fail_process, log_to_process,
    pause_for_human_review, start_process,
};
use anyhow::{anyhow, Result};
use pk_protocol::ipc::Event;
use pk_protocol::pipeline_models::{Pipeline, ProcessStep};
use pk_protocol::process_models::Process;
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;

/// The main pipeline execution engine.
///
/// PipelineEngine takes a Pipeline definition and executes its steps
/// sequentially, delegating agent execution to the AgentManager.
pub struct PipelineEngine {
    agent_manager: AgentManager,
}

impl PipelineEngine {
    /// Create a new PipelineEngine with the given AgentManager.
    ///
    /// # Arguments
    ///
    /// * `agent_manager` - The manager responsible for agent lookup and execution
    pub fn new(agent_manager: AgentManager) -> Self {
        Self { agent_manager }
    }

    /// Execute a pipeline and return the final Process state.
    ///
    /// This is the main entry point for pipeline execution. It:
    /// 1. Creates a new Process
    /// 2. Emits ProcessStarted event
    /// 3. Iterates through pipeline steps sequentially
    /// 4. Executes agents or pauses for HUMAN_REVIEW
    /// 5. Emits appropriate events for each state change
    /// 6. Returns the final Process state
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The pipeline definition to execute
    /// * `events_tx` - Channel for sending events to the UI
    ///
    /// # Returns
    ///
    /// The final Process state after execution completes or pauses.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An agent is not found
    /// - Agent execution fails
    /// - Any other execution error occurs
    pub async fn run(&self, pipeline: &Pipeline, events_tx: Sender<Event>) -> Result<Process> {
        // Create new process
        let mut process = create_process(pipeline.name.clone());

        // Emit ProcessStarted event
        let _ = events_tx
            .send(Event::ProcessStarted {
                process_id: process.id,
                pipeline_name: pipeline.name.clone(),
            })
            .await;

        // Start the process (transition to Running)
        start_process(&mut process, &events_tx).await;

        // Execute each step in the pipeline
        for (step_index, step) in pipeline.master.process.iter().enumerate() {
            // Update current step
            if step_index > 0 {
                advance_step(&mut process);
            }

            match step {
                ProcessStep::Agent(agent_name) => {
                    // Log the step
                    log_to_process(
                        &mut process,
                        &events_tx,
                        format!("Executing agent: {}", agent_name),
                    )
                    .await;

                    // Execute the agent
                    if let Err(e) = self
                        .execute_agent_step(&mut process, agent_name, &events_tx)
                        .await
                    {
                        fail_process(
                            &mut process,
                            &events_tx,
                            format!("Agent execution failed: {}", e),
                        )
                        .await;
                        return Err(e);
                    }

                    // Log completion of this step
                    log_to_process(
                        &mut process,
                        &events_tx,
                        format!("Agent {} completed", agent_name),
                    )
                    .await;
                }
                ProcessStep::HumanReview(_) => {
                    // Log the human review step
                    log_to_process(
                        &mut process,
                        &events_tx,
                        "Pausing for human review".to_string(),
                    )
                    .await;

                    // Pause for human review
                    pause_for_human_review(&mut process, &events_tx).await;

                    // Return early - execution pauses here
                    // The process can be resumed later via StateManager
                    return Ok(process);
                }
            }
        }

        // All steps completed successfully
        complete_process(&mut process, &events_tx).await;

        Ok(process)
    }

    /// Execute a single agent step.
    ///
    /// This method:
    /// 1. Looks up the agent by name
    /// 2. Creates an execution context
    /// 3. Executes the agent
    /// 4. Streams events and logs from the agent
    ///
    /// # Arguments
    ///
    /// * `process` - The current process state
    /// * `agent_name` - The name of the agent to execute
    /// * `events_tx` - Channel for sending events
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or execution fails.
    async fn execute_agent_step(
        &self,
        process: &mut Process,
        agent_name: &str,
        events_tx: &Sender<Event>,
    ) -> Result<()> {
        // Create execution context
        // For now, we use a simple instruction. Future versions may include
        // more context from the pipeline definition.
        let context = ExecutionContext {
            instruction: format!("Execute step for pipeline: {}", process.pipeline_name),
        };

        // Execute the agent
        let mut stream = self
            .agent_manager
            .execute(agent_name, &context)
            .await
            .map_err(|e| anyhow!("Failed to execute agent {}: {}", agent_name, e))?;

        // Process the event stream
        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(AgentEvent::Thought(thought)) => {
                    log_to_process(process, events_tx, format!("[Thought] {}", thought)).await;
                }
                Ok(AgentEvent::ToolCall(tool)) => {
                    log_to_process(process, events_tx, format!("[Tool Call] {}", tool)).await;
                }
                Ok(AgentEvent::MessageChunk(chunk)) => {
                    log_to_process(process, events_tx, chunk).await;
                }
                Ok(AgentEvent::Completed) => {
                    // Agent completed successfully
                    break;
                }
                Err(e) => {
                    // Agent execution error
                    return Err(anyhow!("Agent error: {}", e));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pk_protocol::agent_models::Agent as AgentConfig;
    use pk_protocol::pipeline_models::MasterAgentConfig;
    use pk_protocol::process_models::ProcessStatus;
    use std::collections::HashMap;
    use tokio::sync::mpsc;

    fn create_test_agent_config(name: &str) -> AgentConfig {
        AgentConfig {
            name: name.to_string(),
            description: format!("Test agent {}", name),
            model: "test-model".to_string(),
            color: "blue".to_string(),
            system_prompt: "Test prompt".to_string(),
        }
    }

    fn create_test_pipeline(name: &str, steps: Vec<ProcessStep>) -> Pipeline {
        Pipeline {
            name: name.to_string(),
            required_reference_file: HashMap::new(),
            output_file: HashMap::new(),
            master: MasterAgentConfig {
                model: "test-model".to_string(),
                system_prompt: "Test orchestration".to_string(),
                process: steps,
            },
            sub_agents: vec!["agent1".to_string(), "agent2".to_string()],
        }
    }

    #[tokio::test]
    async fn test_pipeline_engine_new() {
        let configs = vec![create_test_agent_config("test-agent")];
        let manager = AgentManager::new(configs);
        let engine = PipelineEngine::new(manager);

        // Engine should be created successfully
        assert_eq!(engine.agent_manager.list_agents().len(), 1);
    }

    #[tokio::test]
    async fn test_pipeline_engine_simple_execution() {
        let configs = vec![create_test_agent_config("agent1")];
        let manager = AgentManager::new(configs);
        let engine = PipelineEngine::new(manager);

        let steps = vec![ProcessStep::Agent("agent1".to_string())];
        let pipeline = create_test_pipeline("simple-pipeline", steps);

        let (tx, _rx) = mpsc::channel(100);

        let result = engine.run(&pipeline, tx).await;
        assert!(result.is_ok());

        let process = result.unwrap();
        assert_eq!(process.status, ProcessStatus::Completed);
        assert_eq!(process.pipeline_name, "simple-pipeline");
    }

    #[tokio::test]
    async fn test_pipeline_engine_with_human_review() {
        let configs = vec![
            create_test_agent_config("agent1"),
            create_test_agent_config("agent2"),
        ];
        let manager = AgentManager::new(configs);
        let engine = PipelineEngine::new(manager);

        let steps = vec![
            ProcessStep::Agent("agent1".to_string()),
            ProcessStep::HumanReview(pk_protocol::pipeline_models::HumanReviewMarker),
            ProcessStep::Agent("agent2".to_string()),
        ];
        let pipeline = create_test_pipeline("review-pipeline", steps);

        let (tx, _rx) = mpsc::channel(100);

        let result = engine.run(&pipeline, tx).await;
        assert!(result.is_ok());

        let process = result.unwrap();
        // Should pause at HUMAN_REVIEW, not complete
        assert_eq!(process.status, ProcessStatus::HumanReview);
        assert_eq!(process.current_step_index, 1); // Stopped at step 1 (HUMAN_REVIEW)
    }

    #[tokio::test]
    async fn test_pipeline_engine_agent_not_found() {
        let configs = vec![create_test_agent_config("agent1")];
        let manager = AgentManager::new(configs);
        let engine = PipelineEngine::new(manager);

        let steps = vec![ProcessStep::Agent("nonexistent-agent".to_string())];
        let pipeline = create_test_pipeline("failing-pipeline", steps);

        let (tx, _rx) = mpsc::channel(100);

        let result = engine.run(&pipeline, tx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pipeline_engine_event_emission() {
        let configs = vec![create_test_agent_config("agent1")];
        let manager = AgentManager::new(configs);
        let engine = PipelineEngine::new(manager);

        let steps = vec![ProcessStep::Agent("agent1".to_string())];
        let pipeline = create_test_pipeline("event-test", steps);

        let (tx, mut rx) = mpsc::channel(100);

        let handle = tokio::spawn(async move { engine.run(&pipeline, tx).await });

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            let is_completed = matches!(event, Event::ProcessCompleted { .. });
            events.push(event);
            if is_completed {
                break;
            }
        }

        // Should have ProcessStarted event
        assert!(matches!(&events[0], Event::ProcessStarted { .. }));

        // Should have ProcessStatusUpdate(Running) event
        assert!(events.iter().any(|e| matches!(
            e,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::Running,
                ..
            }
        )));

        // Should have log chunks
        assert!(events.iter().any(|e| matches!(e, Event::ProcessLogChunk { .. })));

        // Should have ProcessCompleted event
        assert!(events.iter().any(|e| matches!(e, Event::ProcessCompleted { .. })));

        let _ = handle.await;
    }
}
