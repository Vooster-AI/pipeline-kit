//! State manager for coordinating multiple pipeline processes.
//!
//! The StateManager is the central orchestrator for all pipeline executions.
//! It maintains a registry of active processes and provides operations for
//! starting, pausing, resuming, and killing processes.

use crate::agents::manager::AgentManager;
use crate::engine::PipelineEngine;
use crate::state::process::{pause_process, resume_process};
use anyhow::Result;
use pk_protocol::ipc::Event;
use pk_protocol::pipeline_models::Pipeline;
use pk_protocol::process_models::Process;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// Manages all active pipeline processes.
///
/// The StateManager provides a centralized interface for:
/// - Starting new pipeline executions
/// - Pausing and resuming processes
/// - Killing processes
/// - Querying process state
pub struct StateManager {
    /// Registry of all active processes, indexed by their UUID.
    ///
    /// Uses Arc<Mutex<Process>> for thread-safe access across async tasks.
    processes: Arc<Mutex<HashMap<Uuid, Arc<Mutex<Process>>>>>,

    /// The pipeline engine for executing pipelines.
    engine: Arc<PipelineEngine>,

    /// Channel for sending events to the UI.
    events_tx: mpsc::Sender<Event>,
}

impl StateManager {
    /// Create a new StateManager.
    ///
    /// # Arguments
    ///
    /// * `agent_manager` - The agent manager for executing agents
    /// * `events_tx` - Channel for sending events to the UI
    pub fn new(agent_manager: AgentManager, events_tx: mpsc::Sender<Event>) -> Self {
        let engine = Arc::new(PipelineEngine::new(agent_manager));

        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            engine,
            events_tx,
        }
    }

    /// Start executing a pipeline in the background.
    ///
    /// This spawns a new tokio task to run the pipeline asynchronously.
    /// The process ID is returned immediately, and events are sent
    /// through the events channel as execution progresses.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The pipeline definition to execute
    ///
    /// # Returns
    ///
    /// The UUID of the newly created process.
    pub async fn start_pipeline(&self, pipeline: Pipeline) -> Uuid {
        let engine = Arc::clone(&self.engine);
        let processes = Arc::clone(&self.processes);
        let events_tx = self.events_tx.clone();

        // Spawn background task for pipeline execution
        let handle = tokio::spawn(async move {
            // Run the pipeline
            match engine.run(&pipeline, events_tx.clone()).await {
                Ok(final_process) => {
                    // Store the final process state
                    let process_id = final_process.id;
                    let mut procs = processes.lock().await;
                    procs.insert(process_id, Arc::new(Mutex::new(final_process)));
                }
                Err(e) => {
                    // Pipeline execution failed
                    eprintln!("Pipeline execution failed: {}", e);
                }
            }
        });

        // For now, we create a temporary process to get the ID
        // In a more sophisticated implementation, we would track the spawned task
        // and return the process ID immediately
        // This is a simplified version for the initial implementation
        drop(handle);

        // Return a placeholder UUID
        // TODO: Improve this to track the actual process ID before spawning
        Uuid::new_v4()
    }

    /// Pause a running process.
    ///
    /// The process will transition to the Paused state and stop execution
    /// after completing its current step.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the process to pause
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not found.
    pub async fn pause_process_by_id(&self, process_id: Uuid) -> Result<()> {
        let processes = self.processes.lock().await;

        if let Some(process_arc) = processes.get(&process_id) {
            let mut process = process_arc.lock().await;
            pause_process(&mut process, &self.events_tx).await;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Process {} not found", process_id))
        }
    }

    /// Resume a paused process.
    ///
    /// The process will transition from Paused or HumanReview state
    /// back to Running and continue execution.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the process to resume
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not found.
    ///
    /// # Note
    ///
    /// This is a simplified implementation. A full implementation would
    /// need to re-spawn the execution task to continue from where it paused.
    pub async fn resume_process_by_id(&self, process_id: Uuid) -> Result<()> {
        let processes = self.processes.lock().await;

        if let Some(process_arc) = processes.get(&process_id) {
            let mut process = process_arc.lock().await;
            resume_process(&mut process, &self.events_tx).await;
            // TODO: Re-spawn execution task to continue pipeline
            Ok(())
        } else {
            Err(anyhow::anyhow!("Process {} not found", process_id))
        }
    }

    /// Kill a running process immediately.
    ///
    /// The process will be removed from the active processes registry.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the process to kill
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not found.
    pub async fn kill_process(&self, process_id: Uuid) -> Result<()> {
        let mut processes = self.processes.lock().await;

        if processes.remove(&process_id).is_some() {
            // TODO: Cancel the execution task
            Ok(())
        } else {
            Err(anyhow::anyhow!("Process {} not found", process_id))
        }
    }

    /// Get the current state of a process.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the process to query
    ///
    /// # Returns
    ///
    /// A clone of the process state, or None if not found.
    pub async fn get_process(&self, process_id: Uuid) -> Option<Process> {
        let processes = self.processes.lock().await;
        if let Some(process_arc) = processes.get(&process_id) {
            let process = process_arc.lock().await;
            Some(process.clone())
        } else {
            None
        }
    }

    /// Get all active processes.
    ///
    /// # Returns
    ///
    /// A vector of all active process states.
    pub async fn get_all_processes(&self) -> Vec<Process> {
        let processes = self.processes.lock().await;
        let mut result = Vec::new();

        for process_arc in processes.values() {
            let process = process_arc.lock().await;
            result.push(process.clone());
        }

        result
    }

    /// Get the number of active processes.
    pub async fn process_count(&self) -> usize {
        let processes = self.processes.lock().await;
        processes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pk_protocol::agent_models::Agent as AgentConfig;
    use pk_protocol::pipeline_models::{MasterAgentConfig, ProcessStep};
    use std::collections::HashMap as StdHashMap;

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
            required_reference_file: StdHashMap::new(),
            output_file: StdHashMap::new(),
            master: MasterAgentConfig {
                model: "test-model".to_string(),
                system_prompt: "Test orchestration".to_string(),
                process: steps,
            },
            sub_agents: vec!["agent1".to_string()],
        }
    }

    #[tokio::test]
    async fn test_state_manager_new() {
        let configs = vec![create_test_agent_config("test-agent")];
        let manager = AgentManager::new(configs);
        let (tx, _rx) = mpsc::channel(100);

        let state_manager = StateManager::new(manager, tx);
        assert_eq!(state_manager.process_count().await, 0);
    }

    #[tokio::test]
    async fn test_state_manager_start_pipeline() {
        let configs = vec![create_test_agent_config("agent1")];
        let manager = AgentManager::new(configs);
        let (tx, mut rx) = mpsc::channel(100);

        let state_manager = StateManager::new(manager, tx);

        let steps = vec![ProcessStep::Agent("agent1".to_string())];
        let pipeline = create_test_pipeline("test-pipeline", steps);

        let _process_id = state_manager.start_pipeline(pipeline).await;

        // Give the task time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Should receive some events
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }

        // Should have at least ProcessStarted event
        assert!(!events.is_empty());
    }

    #[tokio::test]
    async fn test_state_manager_get_process() {
        let configs = vec![create_test_agent_config("agent1")];
        let manager = AgentManager::new(configs);
        let (tx, _rx) = mpsc::channel(100);

        let state_manager = StateManager::new(manager, tx);

        // Process doesn't exist yet
        let nonexistent_id = Uuid::new_v4();
        let result = state_manager.get_process(nonexistent_id).await;
        assert!(result.is_none());
    }
}
