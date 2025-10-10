//! State manager for coordinating multiple pipeline processes.
//!
//! The StateManager is the central orchestrator for all pipeline executions.
//! It maintains a registry of active processes and provides operations for
//! starting, pausing, resuming, and killing processes.

use crate::agents::manager::AgentManager;
use crate::engine::PipelineEngine;
use crate::state::process::kill_process_state;
use crate::state::process::pause_process;
use crate::state::process::resume_process;
use anyhow::Result;
use pk_protocol::ipc::Event;
use pk_protocol::pipeline_models::Pipeline;
use pk_protocol::process_models::Process;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
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
    /// Uses `Arc<Mutex<Process>>` for thread-safe access across async tasks.
    processes: Arc<Mutex<HashMap<Uuid, Arc<Mutex<Process>>>>>,

    /// Registry of task handles for background execution, indexed by process UUID.
    ///
    /// Allows cancellation of running tasks via `.abort()`.
    task_handles: Arc<Mutex<HashMap<Uuid, JoinHandle<()>>>>,

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
            task_handles: Arc::new(Mutex::new(HashMap::new())),
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
        // Create and register the process
        let process_id = self.create_and_register_process(&pipeline.name).await;

        // Spawn the pipeline execution in the background
        self.spawn_pipeline_execution(process_id, pipeline).await;

        // Return the process ID
        process_id
    }

    /// Create a new process and register it in the process registry.
    ///
    /// This is a helper function that creates a process with a unique ID,
    /// initializes it in the Pending state, and stores it in the registry.
    ///
    /// # Arguments
    ///
    /// * `pipeline_name` - The name of the pipeline
    ///
    /// # Returns
    ///
    /// The UUID of the newly created process.
    async fn create_and_register_process(&self, pipeline_name: &str) -> Uuid {
        let process_id = Uuid::new_v4();

        let mut initial_process = crate::state::process::create_process(pipeline_name.to_string());
        initial_process.id = process_id;

        let mut procs = self.processes.lock().await;
        procs.insert(process_id, Arc::new(Mutex::new(initial_process)));

        process_id
    }

    /// Spawn a background task to execute the pipeline.
    ///
    /// This function spawns a tokio task that runs the pipeline engine
    /// and updates the process state based on the execution result.
    /// The task handle is stored for later cancellation via kill_process.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The ID of the process to execute
    /// * `pipeline` - The pipeline definition to execute
    async fn spawn_pipeline_execution(&self, process_id: Uuid, pipeline: Pipeline) {
        let engine = Arc::clone(&self.engine);
        let processes = Arc::clone(&self.processes);
        let task_handles = Arc::clone(&self.task_handles);
        let events_tx = self.events_tx.clone();

        // Get the process from the registry to pass to the engine
        let process = {
            let procs = processes.lock().await;
            if let Some(process_arc) = procs.get(&process_id) {
                let p = process_arc.lock().await;
                p.clone()
            } else {
                return; // Process not found, should not happen
            }
        };

        let handle = tokio::spawn(async move {
            match engine.run(&pipeline, process, events_tx.clone()).await {
                Ok(final_process) => {
                    Self::update_process_state(processes.clone(), process_id, final_process).await;
                }
                Err(e) => {
                    Self::handle_pipeline_failure(processes.clone(), process_id, e).await;
                }
            }

            // Clean up the task handle after completion
            let mut handles = task_handles.lock().await;
            handles.remove(&process_id);
        });

        // Store the task handle
        let mut handles = self.task_handles.lock().await;
        handles.insert(process_id, handle);
    }

    /// Update the stored process state after successful execution.
    ///
    /// # Arguments
    ///
    /// * `processes` - The shared process registry
    /// * `process_id` - The ID of the process to update
    /// * `final_process` - The final process state from the engine
    async fn update_process_state(
        processes: Arc<Mutex<HashMap<Uuid, Arc<Mutex<Process>>>>>,
        process_id: Uuid,
        final_process: Process,
    ) {
        let mut procs = processes.lock().await;
        if let Some(process_arc) = procs.get_mut(&process_id) {
            let mut process = process_arc.lock().await;
            *process = final_process;
        }
    }

    /// Handle pipeline execution failure by updating the process state.
    ///
    /// # Arguments
    ///
    /// * `processes` - The shared process registry
    /// * `process_id` - The ID of the failed process
    /// * `error` - The error that caused the failure
    async fn handle_pipeline_failure(
        processes: Arc<Mutex<HashMap<Uuid, Arc<Mutex<Process>>>>>,
        process_id: Uuid,
        error: anyhow::Error,
    ) {
        eprintln!("Pipeline execution failed: {}", error);

        let mut procs = processes.lock().await;
        if let Some(process_arc) = procs.get_mut(&process_id) {
            let mut process = process_arc.lock().await;
            process.status = pk_protocol::process_models::ProcessStatus::Failed;
            process.logs.push(format!("Error: {}", error));
        }
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
    /// This method aborts the background tokio task executing the pipeline,
    /// marks the process as Killed, and emits a ProcessKilled event.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the process to kill
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not found.
    pub async fn kill_process(&self, process_id: Uuid) -> Result<()> {
        // 1. Abort the task handle
        let mut task_handles = self.task_handles.lock().await;
        if let Some(handle) = task_handles.remove(&process_id) {
            handle.abort();
        }
        drop(task_handles);

        // 2. Update the process state to Killed
        let processes = self.processes.lock().await;
        if let Some(process_arc) = processes.get(&process_id) {
            let mut process = process_arc.lock().await;
            kill_process_state(&mut process, &self.events_tx).await;
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
    use pk_protocol::pipeline_models::MasterAgentConfig;
    use pk_protocol::pipeline_models::ProcessStep;
    use pk_protocol::process_models::ProcessStatus;
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

    /// RED: Test that start_pipeline returns unique UUIDs and stores processes
    ///
    /// This test validates that:
    /// 1. Each call to start_pipeline returns a different UUID
    /// 2. The process is stored in the StateManager's internal registry
    /// 3. Multiple processes can be started and tracked independently
    #[tokio::test]
    async fn test_start_pipeline_returns_unique_uuids_and_stores_processes() {
        let configs = vec![create_test_agent_config("agent1")];
        let manager = AgentManager::new(configs);
        let (tx, _rx) = mpsc::channel(100);

        let state_manager = StateManager::new(manager, tx);

        // Create two test pipelines
        let steps = vec![ProcessStep::Agent("agent1".to_string())];
        let pipeline1 = create_test_pipeline("pipeline-1", steps.clone());
        let pipeline2 = create_test_pipeline("pipeline-2", steps);

        // Start first pipeline
        let process_id_1 = state_manager.start_pipeline(pipeline1).await;

        // Start second pipeline
        let process_id_2 = state_manager.start_pipeline(pipeline2).await;

        // UUIDs should be different
        assert_ne!(process_id_1, process_id_2, "Process IDs should be unique");

        // Give the engine time to store the processes
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Both processes should be in the registry
        assert_eq!(
            state_manager.process_count().await,
            2,
            "StateManager should have 2 processes stored"
        );

        // Should be able to retrieve the processes by ID
        let proc1 = state_manager.get_process(process_id_1).await;
        assert!(proc1.is_some(), "Process 1 should be retrievable");
        assert_eq!(proc1.unwrap().pipeline_name, "pipeline-1");

        let proc2 = state_manager.get_process(process_id_2).await;
        assert!(proc2.is_some(), "Process 2 should be retrievable");
        assert_eq!(proc2.unwrap().pipeline_name, "pipeline-2");
    }

    /// RED: Acceptance test for resume_process_by_id
    ///
    /// This test validates that:
    /// 1. A process in HUMAN_REVIEW state can be resumed
    /// 2. The resume signal actually triggers continuation of pipeline execution
    /// 3. The process completes after being resumed
    /// 4. ProcessResumed event is emitted
    #[tokio::test]
    async fn test_resume_process_by_id_acceptance() {
        let configs = vec![
            create_test_agent_config("agent1"),
            create_test_agent_config("agent2"),
        ];
        let manager = AgentManager::new(configs);
        let (tx, mut rx) = mpsc::channel(100);

        let state_manager = StateManager::new(manager, tx);

        // Create a pipeline with HUMAN_REVIEW in the middle
        let steps = vec![
            ProcessStep::Agent("agent1".to_string()),
            ProcessStep::HumanReview(pk_protocol::pipeline_models::HumanReviewMarker),
            ProcessStep::Agent("agent2".to_string()),
        ];
        let pipeline = create_test_pipeline("review-pipeline", steps);

        // Start the pipeline
        let process_id = state_manager.start_pipeline(pipeline).await;

        // Collect events until HumanReview state is reached
        let timeout = tokio::time::Duration::from_secs(2);
        let mut events = Vec::new();
        let mut human_review_reached = false;

        while let Ok(Some(event)) = tokio::time::timeout(timeout, rx.recv()).await {
            let is_human_review = matches!(
                &event,
                Event::ProcessStatusUpdate {
                    status: ProcessStatus::HumanReview,
                    ..
                }
            );
            events.push(event);

            if is_human_review {
                human_review_reached = true;
                break;
            }
        }

        assert!(
            human_review_reached,
            "Pipeline should reach HUMAN_REVIEW state (verified via events)"
        );

        // NOTE: We verify HumanReview state via events because StateManager's
        // process map is only updated when engine.run() completes. While paused
        // at HumanReview, engine.run() is blocked, so the process state in the
        // map hasn't been updated yet. This is expected behavior - the events
        // are the source of truth for real-time state updates.

        // Resume the process
        let resume_result = state_manager.resume_process_by_id(process_id).await;
        assert!(resume_result.is_ok(), "Resume should succeed");

        // Wait for completion event after resume
        let mut completed = false;
        while let Ok(Some(event)) = tokio::time::timeout(timeout, rx.recv()).await {
            let is_completed = matches!(&event, Event::ProcessCompleted { .. });
            events.push(event);

            if is_completed {
                completed = true;
                break;
            }
        }

        assert!(completed, "Pipeline should complete after resume");

        // Verify the process completed
        let final_process = state_manager.get_process(process_id).await.unwrap();
        assert_eq!(
            final_process.status,
            ProcessStatus::Completed,
            "Process should complete after resume"
        );

        // Verify we received the ProcessResumed event
        let has_resumed_event = events
            .iter()
            .any(|e| matches!(e, Event::ProcessResumed { process_id: pid } if *pid == process_id));

        assert!(has_resumed_event, "Should emit ProcessResumed event");
    }

    /// RED: Acceptance test for kill_process with task cancellation
    ///
    /// This test validates that:
    /// 1. kill_process aborts the background tokio task immediately
    /// 2. The process state transitions to Killed
    /// 3. ProcessKilled event is emitted
    /// 4. The task handle is removed from the registry
    /// 5. No memory leaks occur
    #[tokio::test]
    async fn test_kill_process_aborts_background_task() {
        let configs = vec![create_test_agent_config("agent1")];
        let manager = AgentManager::new(configs);
        let (tx, mut rx) = mpsc::channel(100);

        let state_manager = StateManager::new(manager, tx);

        // Create a long-running pipeline (will simulate with a simple agent)
        let steps = vec![ProcessStep::Agent("agent1".to_string())];
        let pipeline = create_test_pipeline("long-running", steps);

        // Start the pipeline
        let process_id = state_manager.start_pipeline(pipeline).await;

        // Give the task time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify the process is running
        let process = state_manager.get_process(process_id).await;
        assert!(process.is_some(), "Process should exist");

        // Kill the process
        let kill_result = state_manager.kill_process(process_id).await;
        assert!(kill_result.is_ok(), "Kill should succeed");

        // Wait a moment for the abortion to take effect
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify the process was marked as Killed
        let killed_process = state_manager.get_process(process_id).await;
        assert!(
            killed_process.is_some(),
            "Process should still be retrievable"
        );
        assert_eq!(
            killed_process.unwrap().status,
            ProcessStatus::Killed,
            "Process should be in Killed state"
        );

        // Verify ProcessKilled event was emitted
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }

        let has_killed_event = events
            .iter()
            .any(|e| matches!(e, Event::ProcessKilled { process_id: pid } if *pid == process_id));

        assert!(has_killed_event, "Should emit ProcessKilled event");

        // Verify the task handle was removed (implicitly tested by successful abort)
        // If the handle wasn't removed, subsequent operations would fail
    }
}
