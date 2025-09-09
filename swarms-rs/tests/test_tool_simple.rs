//! Tests for Tool Module
//! This module tests the tool traits and tool implementations

use swarms_rs::structs::tool::{Tool, ToolDyn, ToolError};
use serde::{Deserialize, Serialize};
use swarms_rs::llm::request::ToolDefinition;

// Mock tool for testing
#[derive(Debug, Clone)]
struct MockTool {
    name: String,
    should_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockArgs {
    input: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockOutput {
    result: String,
}

#[derive(Debug, thiserror::Error)]
enum MockToolError {
    #[error("Mock tool error: {0}")]
    TestError(String),
}

impl MockTool {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_error: false,
        }
    }

    fn new_with_error(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_error: true,
        }
    }
}

impl Tool for MockTool {
    type Error = MockToolError;
    type Args = MockArgs;
    type Output = MockOutput;

    const NAME: &'static str = "mock_tool";

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: "A mock tool for testing".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Input string"
                    }
                },
                "required": ["input"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if self.should_error {
            Err(MockToolError::TestError("Simulated error".to_string()))
        } else {
            Ok(MockOutput {
                result: format!("Processed: {}", args.input),
            })
        }
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[tokio::test]
async fn test_tool_trait_basic_functionality() {
    let tool = MockTool::new("test_tool");
    
    assert_eq!(Tool::name(&tool), "test_tool");
    assert_eq!(MockTool::NAME, "mock_tool");
    
    let definition = Tool::definition(&tool);
    assert_eq!(definition.name, "test_tool");
    assert!(!definition.description.is_empty());
    assert!(definition.parameters.is_object());
}

#[tokio::test]
async fn test_tool_call_success() {
    let tool = MockTool::new("success_tool");
    let args = MockArgs {
        input: "test input".to_string(),
    };
    
    let result = Tool::call(&tool, args).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert_eq!(output.result, "Processed: test input");
}

#[tokio::test]
async fn test_tool_call_error() {
    let tool = MockTool::new_with_error("error_tool");
    let args = MockArgs {
        input: "test input".to_string(),
    };
    
    let result = Tool::call(&tool, args).await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "Mock tool error: Simulated error");
}

#[tokio::test]
async fn test_tool_dyn_trait_success() {
    let tool: Box<dyn ToolDyn> = Box::new(MockTool::new("dyn_tool"));
    
    assert_eq!(tool.name(), "dyn_tool");
    
    let definition = tool.definition();
    assert_eq!(definition.name, "dyn_tool");
    
    let args_json = r#"{"input": "dynamic test"}"#;
    let result = tool.call(args_json.to_string()).await;
    
    assert!(result.is_ok());
    let output_json = result.unwrap();
    let output: MockOutput = serde_json::from_str(&output_json).unwrap();
    assert_eq!(output.result, "Processed: dynamic test");
}

#[tokio::test]
async fn test_tool_dyn_trait_invalid_json() {
    let tool: Box<dyn ToolDyn> = Box::new(MockTool::new("json_tool"));
    
    let invalid_json = "invalid json";
    let result = tool.call(invalid_json.to_string()).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolError::JsonError(_)));
}

#[tokio::test]
async fn test_tool_dyn_trait_tool_error() {
    let tool: Box<dyn ToolDyn> = Box::new(MockTool::new_with_error("error_dyn_tool"));
    
    let args_json = r#"{"input": "error test"}"#;
    let result = tool.call(args_json.to_string()).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolError::ToolCallError(_)));
}

#[test]
fn test_tool_error_types() {
    // Test JsonError
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let tool_error = ToolError::JsonError(json_error);
    assert!(matches!(tool_error, ToolError::JsonError(_)));
    
    // Test ToolCallError
    let custom_error = MockToolError::TestError("custom error".to_string());
    let boxed_error: Box<dyn core::error::Error + Send + Sync> = Box::new(custom_error);
    let tool_error = ToolError::ToolCallError(boxed_error);
    assert!(matches!(tool_error, ToolError::ToolCallError(_)));
}

#[test]
fn test_tool_error_display() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let tool_error = ToolError::JsonError(json_error);
    assert!(tool_error.to_string().starts_with("JsonError:"));
    
    let custom_error = MockToolError::TestError("test message".to_string());
    let boxed_error: Box<dyn core::error::Error + Send + Sync> = Box::new(custom_error);
    let tool_error = ToolError::ToolCallError(boxed_error);
    assert!(tool_error.to_string().starts_with("ToolCallError:"));
}

#[test]
fn test_tool_definition_creation() {
    let definition = ToolDefinition {
        name: "test_definition".to_string(),
        description: "Test tool definition".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "param": {"type": "string"}
            }
        }),
    };
    
    assert_eq!(definition.name, "test_definition");
    assert!(!definition.description.is_empty());
    assert!(definition.parameters.is_object());
}
