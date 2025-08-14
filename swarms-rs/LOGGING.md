# Swarms-RS Logging Guide

## Overview

Swarms-RS includes a comprehensive logging system with colorful, descriptive output and environment variable configuration. The logging system provides detailed insights into agent operations, performance metrics, and execution flow.

## Features

- 🎨 **Colorful Output**: Different log levels and components have distinct colors
- 🔧 **Environment Variable Configuration**: Control log levels via `SWARMS_LOG_LEVEL`
- 📊 **Performance Metrics**: Built-in timing and performance logging
- 🤖 **Agent Context**: Every log includes agent name and ID for multi-agent scenarios
- 🔍 **Detailed Tracing**: Comprehensive logging of agent lifecycle events

## Quick Start

### Initialize Logging

```rust
use swarms_rs::logging::init_logger;

#[tokio::main]
async fn main() {
    // Initialize logging (reads SWARMS_LOG_LEVEL environment variable)
    init_logger();
    
    // Your agent code here...
}
```

### Environment Configuration

Set the `SWARMS_LOG_LEVEL` environment variable to control verbosity:

```bash
# Available levels (from most to least verbose)
export SWARMS_LOG_LEVEL=TRACE    # Everything
export SWARMS_LOG_LEVEL=DEBUG    # Debug + Info + Warn + Error
export SWARMS_LOG_LEVEL=INFO     # Info + Warn + Error (default)
export SWARMS_LOG_LEVEL=WARN     # Warn + Error only
export SWARMS_LOG_LEVEL=ERROR    # Error only
export SWARMS_LOG_LEVEL=OFF      # No logging
```

## Log Categories

### Agent Operations
- **Agent Initialization**: When agents are created and configured
- **Task Execution**: Start, progress, and completion of tasks
- **Loop Iterations**: Each step in the agent's autonomous loop
- **State Changes**: Agent state transitions and status updates

### LLM Interactions
- **Prompt Requests**: When prompts are sent to language models
- **Response Processing**: LLM response handling and parsing
- **Token Usage**: Performance metrics for LLM calls
- **Error Handling**: LLM-specific errors and retries

### Memory Operations
- **Short-term Memory**: Conversation history management
- **Long-term Memory**: Persistent storage operations
- **Caching**: Response caching and retrieval
- **Autosave**: Automatic state persistence

### Tool Execution
- **Tool Calls**: When agents invoke external tools
- **Tool Results**: Processing tool outputs
- **Tool Errors**: Tool execution failures and recovery

### Performance Metrics
- **Execution Time**: Total task completion time
- **LLM Latency**: Response time for language model calls
- **Memory Usage**: Memory operation performance
- **Loop Performance**: Per-iteration timing

## Usage Examples

### Basic Agent with Logging

```rust
use swarms_rs::{
    agent::swarms_agent::SwarmsAgentBuilder,
    llm::provider::openai::OpenAIProvider,
    logging::init_logger,
    structs::agent::AgentConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize colorful logging
    init_logger();

    let model = OpenAIProvider::new("your-api-key", "gpt-3.5-turbo");
    
    let config = AgentConfig::builder()
        .agent_name("MyAgent")
        .user_name("User")
        .max_loops(3)
        .enable_autosave()
        .build();

    let agent = SwarmsAgentBuilder::new_with_model(model)
        .config((*config).clone())
        .build();

    // This will generate comprehensive logs
    let result = agent.run("Write a short poem".to_string()).await?;
    
    Ok(())
}
```

### Custom Logging in Your Code

```rust
use swarms_rs::{log_agent, log_task, log_perf};

// Agent-specific logging
log_agent!(info, "MyAgent", "agent-123", "Custom operation starting");

// Task-specific logging
log_task!(info, "MyAgent", "agent-123", "Write poem", "Task progress: 50% complete");

// Performance logging
log_perf!(info, "MyComponent", "operation_time", 150, "ms");
```

## Log Output Format

The logging system produces structured, colorful output:

```
2024-12-19 15:30:45.123 UTC [INFO] swarms_rs::logging - 🚀 Swarms-RS logging initialized with level: INFO
2024-12-19 15:30:45.124 UTC [INFO] swarms_rs::agent - 🎯 Agent configuration built: MyAgent (ID: abc-123) - Max loops: 3, Temperature: 0.7, Max tokens: 8192
2024-12-19 15:30:45.125 UTC [INFO] swarms_rs::agent - 🏗️ Building SwarmsAgent: MyAgent
2024-12-19 15:30:45.126 UTC [INFO] swarms_rs::agent - ✅ SwarmsAgent built successfully: MyAgent (ID: abc-123) with 1 tools
2024-12-19 15:30:45.127 UTC [INFO] swarms_rs::agent - [MyAgent:abc-123] 📋 Task: Write a short poem - Task initializing - Agent starting autonomous execution loop
```

## Integration with Existing Systems

The logging system uses the standard Rust `log` crate, making it compatible with most logging frameworks:

```rust
// Works with standard log macros
log::info!("Standard log message");

// Enhanced with agent context
log_agent!(info, &agent.name(), &agent.id(), "Agent-specific message");
```

## Production Considerations

### Log Levels in Production
- Use `INFO` or `WARN` levels in production
- `DEBUG` and `TRACE` are for development and troubleshooting
- Consider log rotation for long-running agents

### Performance Impact
- Logging has minimal performance impact at `INFO` level
- `DEBUG` and `TRACE` levels may impact performance in high-throughput scenarios
- Performance metrics are logged at `INFO` level by default

### Monitoring Integration
The structured logging format makes it easy to integrate with monitoring systems:
- Parse JSON-like structured logs
- Extract performance metrics
- Monitor agent health and status
- Track task completion rates

## Troubleshooting

### Common Issues

1. **No logs appearing**: Check that `init_logger()` is called before any agent operations
2. **Too verbose**: Set `SWARMS_LOG_LEVEL=WARN` or `ERROR`
3. **Colors not showing**: Ensure your terminal supports ANSI colors
4. **Performance issues**: Reduce log level to `INFO` or higher

### Debug Mode

For maximum verbosity during debugging:

```bash
export SWARMS_LOG_LEVEL=TRACE
cargo run --example logging_example
```

This will show every operation, including:
- Detailed agent state changes
- Full LLM request/response cycles
- Memory operation details
- Tool execution traces
- Performance breakdowns
