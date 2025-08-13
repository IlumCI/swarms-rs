use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    path::{Path, PathBuf},
};

use chrono::Local;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::structs::persistence::{self, PersistenceError};

// Import ToolCallOutput from the agent module
use crate::agent::swarms_agent::ToolCallOutput;

/// AgentShortMemory provides thread-safe storage for agent conversations.
/// 
/// This struct now supports type-safe storage of tool call outputs, allowing
/// workflows to retrieve typed data from conversations without parsing strings.
/// 
/// # Examples
/// 
/// ```rust
/// use swarms_rs::structs::conversation::{AgentShortMemory, Role};
/// use swarms_rs::agent::swarms_agent::ToolCallOutput;
/// 
/// let memory = AgentShortMemory::new();
/// 
/// // Store text messages
/// memory.add("task1", "agent1", Role::User("user".to_string()), "Hello");
/// 
/// // Store tool calls with type safety
/// let tool_calls = vec![
///     ToolCallOutput {
///         name: "calculator".to_string(),
///         args: r#"{"operation": "add", "a": 1, "b": 2}"#.to_string(),
///         result: "3".to_string(),
///     }
/// ];
/// memory.add_tool_calls("task1", "agent1", Role::Assistant("agent1".to_string()), tool_calls);
/// 
/// // Retrieve typed tool call data
/// if let Some(conversation) = memory.0.get("task1") {
///     let all_tool_calls = conversation.get_tool_calls();
///     let calculator_calls = conversation.get_tool_calls_by_name("calculator");
///     let latest_calls = conversation.get_latest_tool_calls();
/// }
/// ```
#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("FilePersistence error: {0}")]
    FilePersistenceError(#[from] PersistenceError),
}

#[derive(Clone, Serialize)]
pub struct AgentShortMemory(pub DashMap<Task, AgentConversation>);
type Task = String;

impl AgentShortMemory {
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    pub fn add(
        &self,
        task: impl Into<String>,
        conversation_owner: impl Into<String>,
        role: Role,
        message: impl Into<String>,
    ) {
        let mut conversation = self
            .0
            .entry(task.into())
            .or_insert(AgentConversation::new(conversation_owner.into()));
        conversation.add(role, message.into())
    }

    pub fn add_tool_calls(
        &self,
        task: impl Into<String>,
        conversation_owner: impl Into<String>,
        role: Role,
        tool_calls: Vec<ToolCallOutput>,
    ) {
        let mut conversation = self
            .0
            .entry(task.into())
            .or_insert(AgentConversation::new(conversation_owner.into()));
        conversation.add_tool_calls(role, tool_calls)
    }
}

impl Default for AgentShortMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Serialize)]
pub struct AgentConversation {
    agent_name: String,
    save_filepath: Option<PathBuf>,
    pub history: Vec<Message>,
    max_messages: Option<usize>,
}

impl AgentConversation {
    pub fn new(agent_name: String) -> Self {
        Self {
            agent_name,
            save_filepath: None,
            history: Vec::new(),
            max_messages: Some(1_000_000), // Default maximum messages
        }
    }

    /// Create a new AgentConversation with a custom maximum message limit
    pub fn with_max_messages(agent_name: String, max_messages: Option<usize>) -> Self {
        Self {
            agent_name,
            save_filepath: None,
            history: Vec::new(),
            max_messages,
        }
    }

    /// Add a message to the conversation history.
    pub fn add(&mut self, role: Role, message: String) {
        // Only check message limit if it's set
        if let Some(max) = self.max_messages {
            if self.history.len() >= max {
                // Remove oldest messages to make room for new ones
                self.history.drain(0..(self.history.len() - max + 1));
            }
        }

        let timestamp = Local::now().timestamp_millis();
        let message = Message {
            role,
            content: Content::Text(format!("Timestamp(millis): {timestamp} \n{message}")),
        };
        self.history.push(message);

        if let Some(filepath) = &self.save_filepath {
            let filepath = filepath.clone();
            let history = self.history.clone();
            tokio::spawn(async move {
                let history = history;
                let _ = Self::save_as_json(&filepath, &history).await;
            });
        }
    }

    /// Add tool calls to the conversation history with type safety.
    pub fn add_tool_calls(&mut self, role: Role, tool_calls: Vec<ToolCallOutput>) {
        // Only check message limit if it's set
        if let Some(max) = self.max_messages {
            if self.history.len() >= max {
                // Remove oldest messages to make room for new ones
                self.history.drain(0..(self.history.len() - max + 1));
            }
        }

        let timestamp = Local::now().timestamp_millis();
        let message = Message {
            role,
            content: Content::ToolCalls(tool_calls),
        };
        self.history.push(message);

        if let Some(filepath) = &self.save_filepath {
            let filepath = filepath.clone();
            let history = self.history.clone();
            tokio::spawn(async move {
                let history = history;
                let _ = Self::save_as_json(&filepath, &history).await;
            });
        }
    }

    /// Delete a message from the conversation history.
    pub fn delete(&mut self, index: usize) {
        self.history.remove(index);
    }

    /// Update a message in the conversation history.
    pub fn update(&mut self, index: usize, role: Role, content: Content) {
        self.history[index] = Message { role, content };
    }

    /// Query a message in the conversation history.
    pub fn query(&self, index: usize) -> &Message {
        &self.history[index]
    }

    /// Get all tool calls from the conversation history.
    pub fn get_tool_calls(&self) -> Vec<&ToolCallOutput> {
        self.history
            .iter()
            .filter_map(|msg| match &msg.content {
                Content::ToolCalls(tool_calls) => Some(tool_calls.iter().collect::<Vec<_>>()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Get tool calls by tool name from the conversation history.
    pub fn get_tool_calls_by_name(&self, tool_name: &str) -> Vec<&ToolCallOutput> {
        self.history
            .iter()
            .filter_map(|msg| match &msg.content {
                Content::ToolCalls(tool_calls) => Some(
                    tool_calls
                        .iter()
                        .filter(|tool_call| tool_call.name == tool_name)
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Get the most recent tool calls from the conversation history.
    pub fn get_latest_tool_calls(&self) -> Option<&Vec<ToolCallOutput>> {
        self.history
            .iter()
            .rev()
            .find_map(|msg| match &msg.content {
                Content::ToolCalls(tool_calls) => Some(tool_calls),
                _ => None,
            })
    }

    /// Search for a message in the conversation history.
    pub fn search(&self, keyword: &str) -> Vec<&Message> {
        self.history
            .iter()
            .filter(|message| message.content.to_string().contains(keyword))
            .collect()
    }

    // Clear the conversation history.
    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn to_json(&self) -> Result<String, ConversationError> {
        Ok(serde_json::to_string(&self.history)?)
    }

    /// Save the conversation history to a JSON file.
    async fn save_as_json(filepath: &Path, data: &[Message]) -> Result<(), ConversationError> {
        let json_data = serde_json::to_string_pretty(data)?;
        persistence::save_to_file(json_data.as_bytes(), filepath).await?;
        Ok(())
    }

    // TODO: We don't need this function now
    // Load the conversation history from a JSON file.
    // async fn load_from_json(&self, filepath: &Path) -> Result<Vec<Message>, ConversationError> {
    //     let data = persistence::load_from_file(filepath).await?;
    //     let history = serde_json::from_slice(&data)?;
    //     Ok(history)
    // }

    /// Export the conversation history to a file
    pub async fn export_to_file(&self, filepath: &Path) -> Result<(), ConversationError> {
        let data = self.to_string();
        persistence::save_to_file(data.as_bytes(), filepath).await?;
        Ok(())
    }

    /// Import the conversation history from a file
    pub async fn import_from_file(&mut self, filepath: &Path) -> Result<(), ConversationError> {
        let data = persistence::load_from_file(filepath).await?;
        let history = data
            .split(|s| *s == b'\n')
            .map(|line| {
                let line = String::from_utf8_lossy(line);
                // M4n5ter(User): hello
                let (role, content) = line.split_once(": ").unwrap();
                if role.contains("(User)") {
                    let role = Role::User(role.replace("(User)", "").to_string());
                    let content = Content::Text(content.to_string());
                    Message { role, content }
                } else {
                    let role = Role::Assistant(role.replace("(Assistant)", "").to_string());
                    let content = Content::Text(content.to_string());
                    Message { role, content }
                }
            })
            .collect();
        self.history = history;
        Ok(())
    }

    /// Count the number of messages by role
    pub fn count_messages_by_role(&self) -> HashMap<String, usize> {
        let mut count = HashMap::new();
        for message in &self.history {
            *count.entry(message.role.to_string()).or_insert(0) += 1;
        }
        count
    }
}

impl Display for AgentConversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for message in &self.history {
            writeln!(f, "{}: {}", message.role, message.content)?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User(String),
    Assistant(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Content {
    Text(String),
    ToolCalls(Vec<ToolCallOutput>),
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User(name) => write!(f, "{}(User)", name),
            Role::Assistant(name) => write!(f, "{}(Assistant)", name),
        }
    }
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Text(text) => f.pad(text),
            Content::ToolCalls(tool_calls) => {
                let formatted = tool_calls
                    .iter()
                    .map(|tool_call| {
                        format!(
                            "[Tool name]: {}\n[Tool args]: {}\n[Tool result]: {}\n\n",
                            tool_call.name, tool_call.args, tool_call.result
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("");
                f.pad(&formatted)
            }
        }
    }
}

#[derive(Serialize)]
#[serde(rename = "history")]
pub struct SwarmConversation {
    pub logs: VecDeque<AgentLog>,
}

impl SwarmConversation {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::new(),
        }
    }

    pub fn add_log(&mut self, agent_name: String, task: String, response: String) {
        tracing::info!("Agent: {agent_name} | Task: {task} | Response: {response}");
        let log = AgentLog {
            agent_name,
            task,
            response,
        };
        self.logs.push_back(log);
    }
}

impl Default for SwarmConversation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
pub struct AgentLog {
    pub agent_name: String,
    pub task: String,
    pub response: String,
}

impl From<&AgentConversation> for Vec<crate::llm::completion::Message> {
    fn from(conv: &AgentConversation) -> Self {
        conv.history
            .iter()
            .map(|msg| match &msg.role {
                Role::User(name) => {
                    crate::llm::completion::Message::user(format!("{}: {}", name, msg.content))
                },
                Role::Assistant(name) => {
                    match &msg.content {
                        Content::Text(text) => {
                            crate::llm::completion::Message::assistant(format!("{}: {}", name, text))
                        },
                        Content::ToolCalls(tool_calls) => {
                            // For tool calls, we need to preserve the structured format
                            // that the LLM can understand. We'll format them in a way that
                            // maintains the tool call information while being readable.
                            let tool_call_text = tool_calls
                                .iter()
                                .map(|tool_call| {
                                    format!(
                                        "Tool call: {} with args: {} returned: {}",
                                        tool_call.name, tool_call.args, tool_call.result
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            crate::llm::completion::Message::assistant(format!("{}: {}", name, tool_call_text))
                        }
                    }
                },
            })
            .collect()
    }
}
