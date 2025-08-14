use colored::*;
use env_logger::{Builder, Target, WriteStyle};
use log::{Level, LevelFilter};
use std::env;
use std::io::Write;

/// Initialize the logging system with environment variable support
/// Reads SWARMS_LOG_LEVEL environment variable and sets up colorful logging
pub fn init_logger() {
    let log_level = env::var("SWARMS_LOG_LEVEL")
        .unwrap_or_else(|_| "INFO".to_string())
        .to_uppercase();

    let level_filter = match log_level.as_str() {
        "TRACE" => LevelFilter::Trace,
        "DEBUG" => LevelFilter::Debug,
        "INFO" => LevelFilter::Info,
        "WARN" => LevelFilter::Warn,
        "ERROR" => LevelFilter::Error,
        "OFF" => LevelFilter::Off,
        _ => {
            eprintln!(
                "⚠️  Invalid SWARMS_LOG_LEVEL '{}', defaulting to INFO",
                log_level
            );
            LevelFilter::Info
        },
    };

    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .write_style(WriteStyle::Always)
        .filter_level(level_filter)
        .format(|buf, record| {
            let level_color = match record.level() {
                Level::Error => "ERROR".red().bold(),
                Level::Warn => "WARN".yellow().bold(),
                Level::Info => "INFO".green().bold(),
                Level::Debug => "DEBUG".blue().bold(),
                Level::Trace => "TRACE".purple().bold(),
            };

            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
            let target = record.target().bright_black();

            writeln!(
                buf,
                "{} [{}] {} - {}",
                timestamp.to_string().bright_black(),
                level_color,
                target,
                record.args()
            )
        })
        .init();

    log::info!(
        "🚀 Swarms-RS logging initialized with level: {}",
        log_level.green().bold()
    );
}

/// Macro for logging with agent context
#[macro_export]
macro_rules! log_agent {
    ($level:ident, $agent_name:expr, $agent_id:expr, $($arg:tt)*) => {
        log::$level!(
            "[{}:{}] {}",
            $agent_name.bright_cyan().bold(),
            $agent_id.bright_yellow(),
            format!($($arg)*)
        );
    };
}

/// Macro for logging task-related events with agent context
#[macro_export]
macro_rules! log_task {
    ($level:ident, $agent_name:expr, $agent_id:expr, $task:expr, $($arg:tt)*) => {
        log::$level!(
            "[{}:{}] 📋 Task: {} - {}",
            $agent_name.bright_cyan().bold(),
            $agent_id.bright_yellow(),
            $task.bright_magenta(),
            format!($($arg)*)
        );
    };
}

/// Macro for logging tool execution with agent context
#[macro_export]
macro_rules! log_tool {
    ($level:ident, $agent_name:expr, $agent_id:expr, $tool_name:expr, $($arg:tt)*) => {
        log::$level!(
            "[{}:{}] 🔧 Tool: {} - {}",
            $agent_name.bright_cyan().bold(),
            $agent_id.bright_yellow(),
            $tool_name.bright_green(),
            format!($($arg)*)
        );
    };
}

/// Macro for logging workflow events with agent context
#[macro_export]
macro_rules! log_workflow {
    ($level:ident, $agent_name:expr, $agent_id:expr, $workflow_name:expr, $($arg:tt)*) => {
        log::$level!(
            "[{}:{}] 🔄 Workflow: {} - {}",
            $agent_name.bright_cyan().bold(),
            $agent_id.bright_yellow(),
            $workflow_name.bright_blue(),
            format!($($arg)*)
        );
    };
}

/// Macro for logging memory operations with agent context
#[macro_export]
macro_rules! log_memory {
    ($level:ident, $agent_name:expr, $agent_id:expr, $operation:expr, $($arg:tt)*) => {
        log::$level!(
            "[{}:{}] 🧠 Memory: {} - {}",
            $agent_name.bright_cyan().bold(),
            $agent_id.bright_yellow(),
            $operation.bright_purple(),
            format!($($arg)*)
        );
    };
}

/// Macro for logging LLM interactions with agent context
#[macro_export]
macro_rules! log_llm {
    ($level:ident, $agent_name:expr, $agent_id:expr, $model:expr, $($arg:tt)*) => {
        log::$level!(
            "[{}:{}] 🤖 LLM: {} - {}",
            $agent_name.bright_cyan().bold(),
            $agent_id.bright_yellow(),
            $model.bright_red(),
            format!($($arg)*)
        );
    };
}

/// Macro for logging swarm operations
#[macro_export]
macro_rules! log_swarm {
    ($level:ident, $swarm_name:expr, $($arg:tt)*) => {
        log::$level!(
            "[🐝 Swarm: {}] {}",
            $swarm_name.bright_cyan().bold(),
            format!($($arg)*)
        );
    };
}

/// Macro for logging performance metrics
#[macro_export]
macro_rules! log_perf {
    ($level:ident, $component:expr, $metric:expr, $value:expr, $unit:expr) => {
        log::$level!(
            "📊 Performance: {} - {}: {} {}",
            $component.bright_cyan(),
            $metric.bright_yellow(),
            $value.to_string().bright_green().bold(),
            $unit.bright_white()
        );
    };
}

/// Macro for logging errors with context
#[macro_export]
macro_rules! log_error_ctx {
    ($agent_name:expr, $agent_id:expr, $error:expr, $context:expr) => {
        log::error!(
            "[{}:{}] ❌ Error in {}: {}",
            $agent_name.bright_cyan().bold(),
            $agent_id.bright_yellow(),
            $context.bright_red(),
            $error.to_string().red()
        );
    };
}

/// Utility function to log agent initialization
pub fn log_agent_init(agent_name: &str, agent_id: &str, config_summary: &str) {
    log::info!(
        "🎯 Agent [{}:{}] initialized with config: {}",
        agent_name.bright_cyan().bold(),
        agent_id.bright_yellow(),
        config_summary.bright_white()
    );
}

/// Utility function to log agent task completion
pub fn log_agent_task_completion(agent_name: &str, agent_id: &str, task: &str, duration_ms: u64) {
    log::info!(
        "✅ Agent [{}:{}] completed task '{}' in {}ms",
        agent_name.bright_cyan().bold(),
        agent_id.bright_yellow(),
        task.bright_magenta(),
        duration_ms.to_string().bright_green().bold()
    );
}

/// Utility function to log agent state changes
pub fn log_agent_state_change(agent_name: &str, agent_id: &str, from_state: &str, to_state: &str) {
    log::debug!(
        "🔄 Agent [{}:{}] state transition: {} → {}",
        agent_name.bright_cyan().bold(),
        agent_id.bright_yellow(),
        from_state.bright_red(),
        to_state.bright_green()
    );
}
