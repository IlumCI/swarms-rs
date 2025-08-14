use std::env;
use std::sync::Once;
use colored::Colorize;
use swarms_rs::logging::init_logger;
use swarms_rs::{log_agent, log_task, log_tool, log_memory, log_llm};

static INIT: Once = Once::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_logger_initialization() {
        INIT.call_once(|| {
            unsafe {
                env::set_var("SWARMS_LOG_LEVEL", "DEBUG");
            }
            init_logger();
        });

        log::info!("Test log message");
        log::debug!("Test debug message");
        log::error!("Test error message");
    }

    #[test]
    pub fn test_agent_logging_macros() {
        INIT.call_once(|| {
            unsafe {
                env::set_var("SWARMS_LOG_LEVEL", "TRACE");
            }
            init_logger();
        });

        log_agent!(info, "TestAgent", "agent-123", "Agent started successfully");
        log_task!(
            info,
            "TestAgent",
            "agent-123",
            "Process data",
            "Task initialized"
        );
        log_tool!(
            debug,
            "TestAgent",
            "agent-123",
            "WebScraper",
            "Tool execution started"
        );
        log_memory!(
            trace,
            "TestAgent",
            "agent-123",
            "Save",
            "Saving to long-term memory"
        );
        log_llm!(
            info,
            "TestAgent",
            "agent-123",
            "GPT-4",
            "Sending completion request"
        );
    }
}
