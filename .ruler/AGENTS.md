### **Summary**

This document provides development guidelines for `pipeline-kit`, an AI agent pipeline orchestration CLI. The project's architecture is a monorepo combining a high-performance **Rust core engine** (`pipeline-kit-rs`) with a lightweight **TypeScript wrapper** (`pipeline-kit-cli`) for easy distribution via npm. The core principle is strict separation of concerns: Rust handles all business logic, while TypeScript manages platform-specific execution and dependencies. All communication between the UI and the core logic is funneled through a well-defined protocol in the `pk-protocol` crate.

### **Directory Structure**

- `pipeline-kit-cli/`: The npm package wrapper. **Do not add business logic here.** Its sole purpose is to download and execute the correct Rust binary.
- `pipeline-kit-rs/`: The Rust Cargo workspace containing all core logic.
  - `crates/cli/`: The main entry point (`main.rs`) for the Rust binary. Handles command-line argument parsing with `clap`.
  - `crates/core/`: The application's brain. Contains the `PipelineEngine`, `AgentManager`, and `StateManager`.
  - `crates/tui/`: The interactive terminal UI, built with `ratatui`.
  - `crates/protocol/`: The nervous system. Defines all shared data structures (`Op`, `Event`, configuration models) used for communication between `core` and `tui`.
  - `crates/protocol-ts/`: A utility crate to generate TypeScript types from `pk-protocol` structs, ensuring type safety with external clients.

### **Top Level Building Blocks**

- **Rust/TypeScript Separation:**

  - **Rust (`pipeline-kit-rs`):** Implement all performance-critical and stateful logic here. This includes pipeline execution, agent interaction, TUI rendering, and state management.
  - **TypeScript (`pipeline-kit-cli`):** Only for distribution. It detects the user's OS/architecture and spawns the corresponding native Rust binary. It is a simple, transparent launcher.

- **Core-TUI Communication:**

  - All interactions between the UI (`pk-tui`) and the core logic (`pk-core`) MUST use the asynchronous channel-based system defined in `pk-protocol`.
  - The UI sends `Op` (Operation) messages to the core.
  - The core sends `Event` messages back to the UI to report state changes.
  - This decouples the UI from the business logic, enabling parallel development and easier testing.

- **Agent Abstraction (Adapter Pattern):**
  - All AI agent integrations MUST implement the `Agent` trait defined in `pk-core/src/agents/base.rs`.
  - Each specific agent (e.g., Claude, Gemini) is an "adapter" that translates its unique API/SDK into the common `Agent` interface.
  - The `AgentManager` is responsible for loading agent configurations and creating `Arc<dyn Agent>` instances.

### **Coding Convention**

- **Crate Naming:** All crates within the `pipeline-kit-rs` workspace MUST be prefixed with `pk-`.

  - ✅ `pk-core`, `pk-protocol`
  - ❌ `core`, `protocol`

- **Error Handling:**

  - Use `thiserror` for creating specific, typed errors within library crates (`pk-core`, `pk-protocol`).
  - Use `anyhow` for simple, flexible error handling in the main binary application crates (`pk-cli`, `pk-tui`).

- **Async Runtime:** The entire Rust application MUST use `tokio` as the async runtime.

- **Shared Data Structures:**
  - Any data structure shared between `pk-core` and `pk-tui` (or intended for external clients) MUST be defined in `pk-protocol`.
  - All such structs MUST derive `Serialize`, `Deserialize`, `Debug`, `Clone`, and `ts_rs::TS`.

- **Git Commits:** Commit after each meaningful task. Write concise English commit messages.

### **Guidelines**

- **Adding a New Command (e.g., `/stop <process-id>`)**

  1.  **Protocol First:** Add the new operation to the `Op` enum in `pk-protocol/src/op.rs`.
      ```rust
      // in pk-protocol/src/op.rs
      pub enum Op {
          // ... existing ops
          StopProcess { process_id: Uuid },
      }
      ```
  2.  **Core Logic:** In `pk-core`, handle the new `Op` in the main submission loop. Delegate the logic to the appropriate manager (e.g., `StateManager`).
  3.  **UI Interaction:** In `pk-tui`, update the slash command parser to recognize the command and send the corresponding `Op` to the core.

- **Adding a New Agent Adapter (e.g., for Gemini)**

  1.  **Create Adapter:** Create a new file `pk-core/src/agents/adapters/gemini.rs`.
  2.  **Implement Trait:** Implement the `Agent` trait for your new `GeminiAgent` struct. The `execute` method will contain the Gemini-specific SDK/API calls.
  3.  **Register:** Update `AgentManager` in `pk-core/src/agents/manager.rs` to recognize the new agent type from the `agents/*.md` config and instantiate your `GeminiAgent`.

- **State Management:**
  - All state related to running pipelines (e.g., `Process` instances) MUST be managed within the `StateManager` in `pk-core`.
  - Use `Arc<Mutex<T>>` to ensure thread-safe access to shared state across asynchronous tasks.
  - The TUI (`pk-tui`) should be stateless regarding business logic; it only renders the state reported by `Event`s from the core.

### **Test Strategies**

- **TDD is Mandatory:** Every ticket MUST be developed following the RED/GREEN/REFACTOR cycle. Write a failing test first that defines the acceptance criteria.
- **Unit Tests:** Every public function or module MUST have corresponding unit tests.
  - For `pk-core`, use mock implementations of the `Agent` trait (`MockAgent`) to test the `PipelineEngine` without making real API calls.
  - For `pk-protocol`, ensure every data structure can be correctly serialized and deserialized with `serde_json`.
- **TUI Snapshot Testing:** Use `ratatui::backend::TestBackend` to capture the state of the UI as text. This is mandatory for all UI components to prevent visual regressions. Reference `codex-rs/tui/src/chatwidget/tests.rs` for examples.
- **Acceptance Tests:** Each ticket must end with an acceptance test that validates the feature from a higher level.

**Example TDD Test for a Ticket:**
This example demonstrates testing the `ConfigLoader` from Ticket 2.1.

```rust
// in pk-core/src/config/loader.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_load_config_acceptance() {
        // --- RED: Setup a failing test that defines the goal ---
        // Create a temporary directory structure mimicking .pipeline-kit/
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join(".pipeline-kit/pipelines")).unwrap();
        fs::create_dir_all(root.join(".pipeline-kit/agents")).unwrap();

        let pipeline_yaml = "name: test-pipeline\nprocess:\n  - 'step 1'";
        fs::write(root.join(".pipeline-kit/pipelines/test.yaml"), pipeline_yaml).unwrap();

        let agent_md = "---\nname: test-agent\n---\nBe a helpful assistant.";
        fs::write(root.join(".pipeline-kit/agents/test.md"), agent_md).unwrap();

        // Initially, this function doesn't exist, causing a compile error (RED).
        // let config = load_config(root).await;
        // assert!(config.is_ok());

        // --- GREEN: Implement the feature to make the test pass ---
        // After implementing load_config, uncomment the lines above.
        let config = load_config(root).await.unwrap();

        assert_eq!(config.pipelines.len(), 1);
        assert_eq!(config.pipelines[0].name, "test-pipeline");
        assert_eq!(config.agents.len(), 1);
        assert_eq!(config.agents[0].name, "test-agent");
        assert_eq!(config.agents[0].system_prompt.trim(), "Be a helpful assistant.");

        // --- REFACTOR: Improve the implementation (e.g., error handling) ---
        // Add tests for missing files, invalid YAML/Markdown, etc.
        let empty_dir = tempdir().unwrap();
        let empty_config = load_config(empty_dir.path()).await.unwrap();
        assert!(empty_config.pipelines.is_empty());
        assert!(empty_config.agents.is_empty());
    }
}
```
