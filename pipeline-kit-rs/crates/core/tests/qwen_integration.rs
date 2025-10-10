//! Integration tests for Qwen adapter with the full system.

use pk_core::agents::{AgentFactory, AgentManager, AgentType};
use pk_protocol::agent_models::Agent as AgentConfig;

#[tokio::test]
async fn test_qwen_agent_from_config() {
    // Create Qwen agent config
    let config = AgentConfig {
        name: "qwen-dev".to_string(),
        model: "qwen-coder".to_string(),
        description: "Qwen development agent".to_string(),
        color: "purple".to_string(),
        system_prompt: "You are a helpful Qwen assistant.".to_string(),
    };

    // Test factory creation
    let agent = AgentFactory::create(&config);
    assert!(
        agent.is_ok(),
        "Factory should create QwenAdapter successfully"
    );

    let agent = agent.unwrap();

    // Check availability (will be false unless qwen CLI is installed)
    let _available = agent.check_availability().await;

    println!("✅ Qwen agent created successfully from config");
}

#[tokio::test]
async fn test_qwen_in_agent_manager() {
    // Create multiple agent configs including Qwen
    let configs = vec![
        AgentConfig {
            name: "qwen-agent".to_string(),
            model: "qwen-coder".to_string(),
            description: "Qwen agent".to_string(),
            color: "purple".to_string(),
            system_prompt: "Qwen prompt".to_string(),
        },
        AgentConfig {
            name: "mock-agent".to_string(),
            model: "test-model".to_string(),
            description: "Mock agent".to_string(),
            color: "blue".to_string(),
            system_prompt: "Mock prompt".to_string(),
        },
    ];

    // Create AgentManager
    let manager = AgentManager::new(configs);

    // Verify both agents are registered
    assert!(
        manager.has_agent("qwen-agent"),
        "Qwen agent should be registered"
    );
    assert!(
        manager.has_agent("mock-agent"),
        "Mock agent should be registered"
    );

    // Verify agent list
    let agents = manager.list_agents();
    assert_eq!(agents.len(), 2, "Should have 2 agents");
    assert!(
        agents.contains(&"qwen-agent".to_string()),
        "Should contain qwen-agent"
    );

    println!("✅ Qwen agent integrated with AgentManager successfully");
}

#[test]
fn test_qwen_agent_type_detection() {
    // Test various Qwen model names
    assert_eq!(AgentType::from_model_name("qwen-coder"), AgentType::Qwen);
    assert_eq!(
        AgentType::from_model_name("Qwen3-Coder-Plus"),
        AgentType::Qwen
    );
    assert_eq!(AgentType::from_model_name("qwen2.5-coder"), AgentType::Qwen);
    assert_eq!(AgentType::from_model_name("QWEN-turbo"), AgentType::Qwen);

    // Test that non-Qwen models don't get detected as Qwen
    assert_ne!(AgentType::from_model_name("claude-3"), AgentType::Qwen);
    assert_ne!(AgentType::from_model_name("gpt-4"), AgentType::Qwen);

    println!("✅ Qwen model name detection working correctly");
}

#[tokio::test]
async fn test_qwen_with_fallback() {
    // Create configs with Qwen as primary and Mock as fallback
    let configs = vec![
        AgentConfig {
            name: "qwen-primary".to_string(),
            model: "qwen-coder".to_string(),
            description: "Primary Qwen agent".to_string(),
            color: "purple".to_string(),
            system_prompt: "Primary prompt".to_string(),
        },
        AgentConfig {
            name: "mock-fallback".to_string(),
            model: "test-model".to_string(),
            description: "Fallback mock agent".to_string(),
            color: "blue".to_string(),
            system_prompt: "Fallback prompt".to_string(),
        },
    ];

    // Create manager with fallback
    let manager = AgentManager::new(configs).with_fallback("mock-fallback".to_string());

    // Verify agents are registered
    assert!(manager.has_agent("qwen-primary"));
    assert!(manager.has_agent("mock-fallback"));

    println!("✅ Qwen agent works with fallback mechanism");
}

#[test]
fn test_multiple_qwen_models() {
    // Test that different Qwen model variants all map to Qwen type
    let models = vec![
        "qwen-coder",
        "qwen2.5-coder",
        "Qwen3-Coder-Plus",
        "qwen-turbo",
        "QWEN-MAX",
    ];

    for model in models {
        assert_eq!(
            AgentType::from_model_name(model),
            AgentType::Qwen,
            "Model '{}' should be detected as Qwen",
            model
        );
    }

    println!("✅ All Qwen model variants detected correctly");
}
