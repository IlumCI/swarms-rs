//! Tests for Agent Configuration
//! This module tests the agent configuration builder and agent config struct

use swarms_rs::structs::agent::{AgentConfig, AgentError};
use tempfile::tempdir;

#[test]
fn test_agent_config_builder_creation() {
    let config = AgentConfig::builder()
        .agent_name("TestAgent")
        .user_name("TestUser")
        .description("A test agent")
        .temperature(0.7)
        .max_loops(5)
        .max_tokens(1000)
        .build();

    assert_eq!(config.name, "TestAgent");
    assert_eq!(config.user_name, "TestUser");
    assert_eq!(config.description, Some("A test agent".to_string()));
    assert_eq!(config.temperature, 0.7);
    assert_eq!(config.max_loops, 5);
    assert_eq!(config.max_tokens, 1000);
    assert!(!config.id.is_empty());
}

#[test]
fn test_agent_config_builder_defaults() {
    let config = AgentConfig::builder()
        .agent_name("DefaultAgent")
        .user_name("DefaultUser")
        .build();

    assert_eq!(config.name, "DefaultAgent");
    assert_eq!(config.user_name, "DefaultUser");
    assert_eq!(config.description, None);
    assert_eq!(config.temperature, 0.7); // Default value
    assert_eq!(config.max_loops, 1); // Default value
    assert!(!config.id.is_empty());
}

#[test]
fn test_agent_config_builder_plan_enabled() {
    let config = AgentConfig::builder()
        .agent_name("PlanAgent")
        .user_name("PlanUser")
        .enable_plan(Some("Test planning prompt".to_string()))
        .build();

    assert!(config.plan_enabled);
    assert_eq!(config.planning_prompt, Some("Test planning prompt".to_string()));
}

#[test]
fn test_agent_config_builder_autosave() {
    let config = AgentConfig::builder()
        .agent_name("AutosaveAgent")
        .user_name("AutosaveUser")
        .enable_autosave()
        .build();

    assert!(config.autosave);
}

#[test]
fn test_agent_config_builder_retry_attempts() {
    let config = AgentConfig::builder()
        .agent_name("RetryAgent")
        .user_name("RetryUser")
        .retry_attempts(5)
        .build();

    assert_eq!(config.retry_attempts, 5);
}

#[test]
fn test_agent_config_builder_rag_every_loop() {
    let config = AgentConfig::builder()
        .agent_name("RAGAgent")
        .user_name("RAGUser")
        .enable_rag_every_loop()
        .build();

    assert!(config.rag_every_loop);
}

#[test]
fn test_agent_config_builder_save_state_path() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("agent_state.json");
    let path_str = path.to_str().unwrap();

    let config = AgentConfig::builder()
        .agent_name("SaveStateAgent")
        .user_name("SaveStateUser")
        .save_sate_path(path_str)
        .build();

    assert_eq!(config.save_state_dir, Some(path_str.to_string()));
}

#[test]
fn test_agent_config_builder_stop_words() {
    let config = AgentConfig::builder()
        .agent_name("StopWordsAgent")
        .user_name("StopWordsUser")
        .add_stop_word("STOP")
        .add_stop_word("END")
        .build();

    assert!(config.stop_words.contains("STOP"));
    assert!(config.stop_words.contains("END"));
    assert_eq!(config.stop_words.len(), 2);
}

#[test]
fn test_agent_config_builder_stop_words_batch() {
    let stop_words = vec!["STOP".to_string(), "END".to_string(), "QUIT".to_string()];
    
    let config = AgentConfig::builder()
        .agent_name("BatchStopWordsAgent")
        .user_name("BatchStopWordsUser")
        .stop_words(stop_words.clone())
        .build();

    assert_eq!(config.stop_words.len(), 3);
    for word in stop_words {
        assert!(config.stop_words.contains(&word));
    }
}

#[test]
fn test_agent_config_builder_verbose() {
    let config = AgentConfig::builder()
        .agent_name("VerboseAgent")
        .user_name("VerboseUser")
        .verbose(true)
        .build();

    assert!(config.verbose);
}

#[test]
fn test_agent_config_builder_chaining() {
    let config = AgentConfig::builder()
        .agent_name("ChainedAgent")
        .user_name("ChainedUser")
        .description("A chained configuration test")
        .temperature(0.8)
        .max_loops(3)
        .max_tokens(500)
        .enable_plan(Some("Chained plan".to_string()))
        .enable_autosave()
        .retry_attempts(2)
        .enable_rag_every_loop()
        .add_stop_word("CHAIN_STOP")
        .verbose(true)
        .build();

    assert_eq!(config.name, "ChainedAgent");
    assert_eq!(config.user_name, "ChainedUser");
    assert_eq!(config.description, Some("A chained configuration test".to_string()));
    assert_eq!(config.temperature, 0.8);
    assert_eq!(config.max_loops, 3);
    assert_eq!(config.max_tokens, 500);
    assert!(config.plan_enabled);
    assert_eq!(config.planning_prompt, Some("Chained plan".to_string()));
    assert!(config.autosave);
    assert_eq!(config.retry_attempts, 2);
    assert!(config.rag_every_loop);
    assert!(config.stop_words.contains("CHAIN_STOP"));
    assert!(config.verbose);
}

#[test]
fn test_agent_config_serialization() {
    let config = AgentConfig::builder()
        .agent_name("SerializableAgent")
        .user_name("SerializableUser")
        .description("Test serialization")
        .temperature(0.5)
        .max_loops(7)
        .build();

    // Test serialization to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("SerializableAgent"));

    // Test deserialization from JSON
    let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, config.name);
    assert_eq!(deserialized.user_name, config.user_name);
    assert_eq!(deserialized.description, config.description);
    assert_eq!(deserialized.temperature, config.temperature);
    assert_eq!(deserialized.max_loops, config.max_loops);
}

#[test]
fn test_agent_error_types() {
    // Test different error types
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let agent_error = AgentError::IoError(io_error);
    assert!(matches!(agent_error, AgentError::IoError(_)));

    let serde_error = serde_json::from_str::<AgentConfig>("invalid json").unwrap_err();
    let agent_error = AgentError::SerdeError(serde_error);
    assert!(matches!(agent_error, AgentError::SerdeError(_)));

    let agent_error = AgentError::NoChoiceFound;
    assert!(matches!(agent_error, AgentError::NoChoiceFound));

    let agent_error = AgentError::ToolNotFound("nonexistent_tool".to_string());
    assert!(matches!(agent_error, AgentError::ToolNotFound(_)));
}

#[cfg(test)]
mod agent_error_tests {
    use super::*;

    #[test]
    fn test_agent_error_display() {
        let error = AgentError::InvalidSaveStatePath("invalid/path".to_string());
        assert_eq!(error.to_string(), "Invalid save state path: invalid/path");

        let error = AgentError::ToolNotFound("missing_tool".to_string());
        assert_eq!(error.to_string(), "Tool missing_tool not found");

        let error = AgentError::NoChoiceFound;
        assert_eq!(error.to_string(), "No choice found");
    }
}
