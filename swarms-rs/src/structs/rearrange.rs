use std::{collections::HashMap, path::Path};

use chrono::Local;
use dashmap::DashSet;
use erased_serde::Serialize as ErasedSerialize;
use futures::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::structs::{
    agent::{Agent, AgentError},
    conversation::{AgentConversation, Role},
    persistence::{self, PersistenceError},
    swarm::{MetadataSchemaMap, Swarm, SwarmError},
};

/// Errors that can occur during agent rearrangement operations
#[derive(Debug, Error)]
pub enum AgentRearrangeError {
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    #[error("FilePersistence error: {0}")]
    FilePersistenceError(#[from] PersistenceError),
    #[error("Flow validation error: {0}")]
    FlowValidationError(String),
    #[error("Agent '{0}' not found")]
    AgentNotFound(String),
    #[error("Invalid flow format: {0}")]
    InvalidFlowFormat(String),
    #[error("Duplicate agent names in flow are not allowed")]
    DuplicateAgentNames,
    #[error("Tasks or Agents are empty")]
    EmptyTasksOrAgents,
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

/// Output format options for agent rearrange results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    /// Return all agent responses concatenated
    All,
    /// Return only the final agent's response
    Final,
    /// Return a list of all agent responses
    List,
    /// Return a dictionary mapping agent names to responses
    Dict,
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::All
    }
}

/// Configuration builder for AgentRearrange
#[derive(Default)]
pub struct AgentRearrangeBuilder {
    name: String,
    description: String,
    agents: Vec<Box<dyn Agent>>,
    flow: Option<String>,
    max_loops: u32,
    verbose: bool,
    output_type: OutputType,
    autosave: bool,
    return_json: bool,
    metadata_output_dir: String,
    rules: Option<String>,
    team_awareness: bool,
}

impl AgentRearrangeBuilder {
    /// Set the name of the agent rearrange instance
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the description of the agent rearrange instance
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add an agent to the rearrange configuration
    pub fn add_agent(mut self, agent: Box<dyn Agent>) -> Self {
        self.agents.push(agent);
        self
    }

    /// Set all agents at once
    pub fn agents(self, agents: Vec<Box<dyn Agent>>) -> Self {
        agents
            .into_iter()
            .fold(self, |builder, agent| builder.add_agent(agent))
    }

    /// Set the flow pattern for task execution
    pub fn flow(mut self, flow: impl Into<String>) -> Self {
        self.flow = Some(flow.into());
        self
    }

    /// Set the maximum number of execution loops
    pub fn max_loops(mut self, max_loops: u32) -> Self {
        self.max_loops = max_loops;
        self
    }

    /// Enable or disable verbose logging
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set the output format type
    pub fn output_type(mut self, output_type: OutputType) -> Self {
        self.output_type = output_type;
        self
    }

    /// Enable or disable autosave functionality
    pub fn autosave(mut self, autosave: bool) -> Self {
        self.autosave = autosave;
        self
    }

    /// Enable or disable JSON return format
    pub fn return_json(mut self, return_json: bool) -> Self {
        self.return_json = return_json;
        self
    }

    /// Set the metadata output directory
    pub fn metadata_output_dir(mut self, dir: impl Into<String>) -> Self {
        self.metadata_output_dir = dir.into();
        self
    }

    /// Set rules to be injected into all agents
    pub fn rules(mut self, rules: impl Into<String>) -> Self {
        self.rules = Some(rules.into());
        self
    }

    /// Enable team awareness functionality
    pub fn team_awareness(mut self, team_awareness: bool) -> Self {
        self.team_awareness = team_awareness;
        self
    }

    /// Build the AgentRearrange instance
    pub fn build(self) -> AgentRearrange {
        AgentRearrange {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            description: self.description,
            agents: self
                .agents
                .into_iter()
                .map(|agent| (agent.name(), agent))
                .collect(),
            flow: self.flow.unwrap_or_default(),
            max_loops: if self.max_loops > 0 {
                self.max_loops
            } else {
                1
            },
            verbose: self.verbose,
            output_type: self.output_type,
            autosave: self.autosave,
            return_json: self.return_json,
            metadata_output_dir: self.metadata_output_dir,
            conversation: AgentConversation::new("AgentRearrange".to_string()),
            metadata_map: MetadataSchemaMap::default(),
            tasks: DashSet::new(),
            rules: self.rules,
            team_awareness: self.team_awareness,
        }
    }
}

/// A swarm of agents for rearranging and executing tasks in specified flow patterns.
///
/// AgentRearrange enables flexible task execution by allowing agents to be organized
/// in sequential or parallel flows. It supports various output formats, concurrent
/// execution, batch processing, and extensive configuration options.
///
/// # Features
///
/// - **Flow-based execution**: Define custom flows with `->`  syntax for sequential and parallel execution
/// - **Multiple output formats**: Support for different output types (All, Final, List, Dict)
/// - **Concurrent processing**: Leverage tokio for efficient async task execution
/// - **Team awareness**: Optional team information sharing between agents
/// - **Autosave**: Automatic persistence of agent interactions and metadata
/// - **Batch processing**: Execute multiple tasks efficiently
///
/// # Flow Syntax
///
/// - Sequential: `"agent1 -> agent2 -> agent3"`
/// - Parallel: `"agent1, agent2 -> agent3"`
/// - Mixed: `"agent1 -> agent2, agent3 -> agent4"`
///
/// # Examples
///
/// ```rust,no_run
/// use swarms_rs::structs::rearrange::AgentRearrange;
///
/// let rearrange = AgentRearrange::builder()
///     .name("TaskProcessor")
///     .description("Processes tasks through multiple agents")
///     .flow("researcher -> analyst, reviewer -> summarizer")
///     .max_loops(3)
///     .verbose(true)
///     .build();
/// ```
pub struct AgentRearrange {
    /// Unique identifier for the swarm instance
    id: String,
    /// Name of the agent rearrange instance
    name: String,
    /// Description of the agent rearrange instance's purpose
    description: String,
    /// Map of agent names to Agent objects
    agents: HashMap<String, Box<dyn Agent>>,
    /// Flow pattern defining task execution order
    flow: String,
    /// Maximum number of execution loops
    max_loops: u32,
    /// Whether to enable verbose logging
    verbose: bool,
    /// Format of output (All, Final, List, Dict)
    output_type: OutputType,
    /// Whether to enable autosave functionality
    autosave: bool,
    /// Whether to return output in JSON format
    return_json: bool,
    /// Directory for metadata output
    metadata_output_dir: String,
    /// Conversation history and management
    conversation: AgentConversation,
    /// Metadata mapping for agent interactions
    #[allow(dead_code)]
    metadata_map: MetadataSchemaMap,
    /// Set of active tasks
    #[allow(dead_code)]
    tasks: DashSet<String>,
    /// Rules to inject into all agents
    rules: Option<String>,
    /// Whether team awareness is enabled
    team_awareness: bool,
}

impl Default for AgentRearrange {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "AgentRearrange".to_string(),
            description: "A swarm of agents for rearranging tasks.".to_string(),
            agents: HashMap::new(),
            flow: String::new(),
            max_loops: 1,
            verbose: false,
            output_type: OutputType::All,
            autosave: false,
            return_json: false,
            metadata_output_dir: String::new(),
            conversation: AgentConversation::new("AgentRearrange".to_string()),
            metadata_map: MetadataSchemaMap::default(),
            tasks: DashSet::new(),
            rules: None,
            team_awareness: false,
        }
    }
}

impl AgentRearrange {
    /// Create a new builder for AgentRearrange
    ///
    /// # Returns
    ///
    /// A new `AgentRearrangeBuilder` instance for configuring the agent rearrange system.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use swarms_rs::structs::rearrange::AgentRearrange;
    ///
    /// let rearrange = AgentRearrange::builder()
    ///     .name("MySwarm")
    ///     .description("A custom swarm configuration")
    ///     .build();
    /// ```
    pub fn builder() -> AgentRearrangeBuilder {
        AgentRearrangeBuilder::default()
    }

    /// Set a custom flow pattern for task execution
    ///
    /// # Arguments
    ///
    /// * `flow` - The flow pattern string defining execution order
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use swarms_rs::structs::rearrange::AgentRearrange;
    /// let mut rearrange = AgentRearrange::default();
    /// rearrange.set_custom_flow("agent1 -> agent2, agent3 -> agent4");
    /// ```
    pub fn set_custom_flow(&mut self, flow: impl Into<String>) {
        self.flow = flow.into();
        if self.verbose {
            tracing::info!("Custom flow set: {}", self.flow);
        }
    }

    /// Add an agent to the swarm
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to be added to the swarm
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use swarms_rs::structs::rearrange::AgentRearrange;
    /// let mut rearrange = AgentRearrange::default();
    /// // rearrange.add_agent(my_agent);
    /// ```
    pub fn add_agent(&mut self, agent: Box<dyn Agent>) {
        let agent_name = agent.name();
        if self.verbose {
            tracing::info!("Adding agent {} to the swarm", agent_name);
        }
        self.agents.insert(agent_name, agent);
    }

    /// Remove an agent from the swarm
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the agent to be removed
    ///
    /// # Returns
    ///
    /// The removed agent if it existed, None otherwise
    pub fn remove_agent(&mut self, agent_name: &str) -> Option<Box<dyn Agent>> {
        if self.verbose {
            tracing::info!("Removing agent {} from the swarm", agent_name);
        }
        self.agents.remove(agent_name)
    }

    /// Add multiple agents to the swarm
    ///
    /// # Arguments
    ///
    /// * `agents` - A vector of agents to be added
    pub fn add_agents(&mut self, agents: Vec<Box<dyn Agent>>) {
        for agent in agents {
            self.add_agent(agent);
        }
    }

    /// Validate the flow pattern for correctness
    ///
    /// This method checks that:
    /// - The flow contains the `->` separator
    /// - All referenced agents are registered in the swarm
    /// - The flow syntax is properly formatted
    ///
    /// # Returns
    ///
    /// `Result<(), AgentRearrangeError>` - Ok if valid, error with details if invalid
    ///
    /// # Errors
    ///
    /// - `FlowValidationError` if the flow format is incorrect
    /// - `AgentNotFound` if referenced agents are not registered
    pub fn validate_flow(&self) -> Result<(), AgentRearrangeError> {
        if self.flow.is_empty() {
            return Err(AgentRearrangeError::FlowValidationError(
                "Flow cannot be empty".to_string(),
            ));
        }

        // Handle both sequential (with ->) and parallel-only (without ->) flows
        let tasks: Vec<&str> = if self.flow.contains("->") {
            self.flow.split("->").collect()
        } else {
            // For parallel-only flows like "agent1, agent2"
            vec![self.flow.as_str()]
        };

        for task in tasks {
            let agent_names: Vec<&str> = task.split(',').map(|name| name.trim()).collect();

            for agent_name in agent_names {
                if agent_name != "H" && !self.agents.contains_key(agent_name) {
                    return Err(AgentRearrangeError::AgentNotFound(agent_name.to_string()));
                }
            }
        }

        if self.verbose {
            tracing::info!("Flow: {} is valid", self.flow);
        }

        Ok(())
    }

    /// Execute the agent rearrangement task
    ///
    /// This method processes a task through the configured flow of agents,
    /// supporting both sequential and parallel execution patterns.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to be processed by the agents
    ///
    /// # Returns
    ///
    /// `Result<String, AgentRearrangeError>` - The processed output in the specified format
    ///
    /// # Errors
    ///
    /// - `FlowValidationError` if the flow validation fails
    /// - `AgentError` if any agent execution fails
    /// - `ExecutionError` for other execution-related issues
    pub async fn run(&mut self, task: impl Into<String>) -> Result<String, AgentRearrangeError> {
        self.run_internal(task, None, None).await
    }

    /// Internal execution method with full parameter support
    async fn run_internal(
        &mut self,
        task: impl Into<String>,
        _img: Option<String>,
        _custom_tasks: Option<HashMap<String, String>>,
    ) -> Result<String, AgentRearrangeError> {
        let task = task.into();

        if self.verbose {
            tracing::info!("Starting task execution: {}", task);
        }

        // Add initial task to conversation
        self.conversation
            .add(Role::User("System".to_string()), task.clone());

        // Validate flow before execution
        self.validate_flow()?;

        // Apply rules if configured
        if let Some(rules) = &self.rules {
            self.conversation.add(
                Role::User("System".to_string()),
                format!("Rules: {}", rules),
            );
        }

        let tasks: Vec<&str> = if self.flow.contains("->") {
            self.flow.split("->").collect()
        } else {
            // For parallel-only flows like "agent1, agent2"
            vec![self.flow.as_str()]
        };
        let mut current_task = task.clone();
        let mut response_map = HashMap::new();

        for loop_count in 0..self.max_loops {
            if self.verbose {
                tracing::info!("Starting loop {}/{}", loop_count + 1, self.max_loops);
            }

            for task_step in tasks.iter() {
                let agent_names: Vec<&str> = task_step.split(',').map(|name| name.trim()).collect();

                if agent_names.len() > 1 {
                    // Parallel processing
                    if self.verbose {
                        tracing::info!("Running agents in parallel: {:?}", agent_names);
                    }

                    let parallel_results = self
                        .execute_agents_parallel(&agent_names, &current_task)
                        .await?;

                    for (agent_name, result) in parallel_results {
                        self.conversation
                            .add(Role::Assistant(agent_name.clone()), result.clone());
                        response_map.insert(agent_name, result);
                    }
                } else {
                    // Sequential processing
                    let agent_name = agent_names[0];

                    if self.verbose {
                        tracing::info!("Running agent sequentially: {}", agent_name);
                    }

                    if agent_name == "H" {
                        // Human-in-the-loop placeholder
                        if self.verbose {
                            tracing::info!("Human intervention point reached");
                        }
                        continue;
                    }

                    let agent = self.agents.get(agent_name).ok_or_else(|| {
                        AgentRearrangeError::AgentNotFound(agent_name.to_string())
                    })?;

                    let result = agent
                        .run(self.conversation.to_string())
                        .await
                        .map_err(AgentRearrangeError::AgentError)?;

                    self.conversation
                        .add(Role::Assistant(agent_name.to_string()), result.clone());

                    response_map.insert(agent_name.to_string(), result.clone());
                    current_task = result;
                }
            }
        }

        if self.verbose {
            tracing::info!("Task execution completed");
        }

        // Format output based on output_type
        let output = self.format_output(&response_map, &current_task);

        if self.autosave {
            self.save_metadata().await?;
        }

        Ok(output)
    }

    /// Execute multiple agents in parallel
    async fn execute_agents_parallel(
        &self,
        agent_names: &[&str],
        task: &str,
    ) -> Result<HashMap<String, String>, AgentRearrangeError> {
        let mut handles = Vec::new();

        for agent_name in agent_names {
            if *agent_name == "H" {
                continue; // Skip human-in-the-loop for parallel execution
            }

            let agent = self
                .agents
                .get(*agent_name)
                .ok_or_else(|| AgentRearrangeError::AgentNotFound(agent_name.to_string()))?;

            let task_clone = task.to_string();
            let agent_name_clone = agent_name.to_string();

            // Clone the agent for parallel execution
            let agent_clone = agent.clone_box();

            let handle = tokio::spawn(async move {
                let result = agent_clone.run(task_clone).await;
                (agent_name_clone, result)
            });

            handles.push(handle);
        }

        // Wait for all parallel tasks to complete
        let mut results = HashMap::new();
        for handle in handles {
            let (agent_name, result) = handle.await?;

            let result = result.map_err(AgentRearrangeError::AgentError)?;
            results.insert(agent_name, result);
        }

        Ok(results)
    }

    /// Format the output based on the configured output type
    fn format_output(&self, response_map: &HashMap<String, String>, final_result: &str) -> String {
        match self.output_type {
            OutputType::All => {
                let mut output = String::new();
                for (agent_name, response) in response_map {
                    output.push_str(&format!("{}: {}\n", agent_name, response));
                }
                output
            },
            OutputType::Final => final_result.to_string(),
            OutputType::List => {
                let responses: Vec<String> = response_map.values().cloned().collect();
                if self.return_json {
                    serde_json::to_string(&responses).unwrap_or_else(|_| "[]".to_string())
                } else {
                    responses.join("\n")
                }
            },
            OutputType::Dict => {
                if self.return_json {
                    serde_json::to_string(response_map).unwrap_or_else(|_| "{}".to_string())
                } else {
                    response_map
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            },
        }
    }

    /// Process multiple tasks in batches
    ///
    /// # Arguments
    ///
    /// * `tasks` - Vector of tasks to process
    /// * `batch_size` - Number of tasks to process simultaneously
    /// * `img` - Optional image paths corresponding to tasks
    ///
    /// # Returns
    ///
    /// `Result<Vec<String>, AgentRearrangeError>` - Vector of results corresponding to input tasks
    pub async fn batch_run(
        &mut self,
        tasks: Vec<String>,
        batch_size: usize,
        img: Option<Vec<String>>,
    ) -> Result<Vec<String>, AgentRearrangeError> {
        if tasks.is_empty() {
            return Err(AgentRearrangeError::EmptyTasksOrAgents);
        }

        let mut results = Vec::with_capacity(tasks.len());

        for chunk in tasks.chunks(batch_size) {
            let mut batch_handles = Vec::new();

            for (i, task) in chunk.iter().enumerate() {
                let img_path = img.as_ref().and_then(|imgs| imgs.get(i)).cloned();
                let task_clone = task.clone();

                // Create a clone of self for each task
                let mut rearrange_clone = self.clone_for_task();

                let handle = tokio::spawn(async move {
                    rearrange_clone
                        .run_internal(task_clone, img_path, None)
                        .await
                });

                batch_handles.push(handle);
            }

            // Wait for batch to complete
            for handle in batch_handles {
                let result = handle.await??;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Process multiple tasks concurrently without batching
    ///
    /// # Arguments
    ///
    /// * `tasks` - Vector of tasks to process concurrently
    /// * `img` - Optional image paths corresponding to tasks
    /// * `max_concurrent` - Maximum number of concurrent tasks (None for unlimited)
    ///
    /// # Returns
    ///
    /// `Result<Vec<String>, AgentRearrangeError>` - Vector of results corresponding to input tasks
    pub async fn concurrent_run(
        &mut self,
        tasks: Vec<String>,
        img: Option<Vec<String>>,
        max_concurrent: Option<usize>,
    ) -> Result<Vec<String>, AgentRearrangeError> {
        if tasks.is_empty() {
            return Err(AgentRearrangeError::EmptyTasksOrAgents);
        }

        let stream = stream::iter(tasks.into_iter().enumerate().map(|(i, task)| {
            let img_path = img.as_ref().and_then(|imgs| imgs.get(i)).cloned();
            let mut rearrange_clone = self.clone_for_task();

            async move { rearrange_clone.run_internal(task, img_path, None).await }
        }));

        let results: Result<Vec<_>, _> = if let Some(max_concurrent) = max_concurrent {
            stream.buffer_unordered(max_concurrent).try_collect().await
        } else {
            stream.buffer_unordered(8).try_collect().await // Default to 8 concurrent tasks
        };

        results
    }

    /// Create a lightweight clone for task execution
    fn clone_for_task(&self) -> Self {
        let mut cloned_agents = HashMap::new();
        for (name, agent) in &self.agents {
            cloned_agents.insert(name.clone(), agent.clone_box());
        }

        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            agents: cloned_agents,
            flow: self.flow.clone(),
            max_loops: self.max_loops,
            verbose: self.verbose,
            output_type: self.output_type.clone(),
            autosave: false, // Disable autosave for clones
            return_json: self.return_json,
            metadata_output_dir: self.metadata_output_dir.clone(),
            conversation: AgentConversation::new(format!("{}-clone", self.name)),
            metadata_map: MetadataSchemaMap::default(),
            tasks: DashSet::new(),
            rules: self.rules.clone(),
            team_awareness: self.team_awareness,
        }
    }

    /// Save metadata to the configured output directory
    async fn save_metadata(&self) -> Result<(), AgentRearrangeError> {
        if !self.metadata_output_dir.is_empty() {
            let metadata = self.to_metadata();
            let path = Path::new(&self.metadata_output_dir).join(format!("{}.json", self.id));
            let json_data = serde_json::to_string_pretty(&metadata)?;
            persistence::save_to_file(json_data.as_bytes(), &path).await?;
        }
        Ok(())
    }

    /// Convert the agent rearrange instance to metadata for persistence
    fn to_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "flow": self.flow,
            "max_loops": self.max_loops,
            "agents": self.agents.keys().collect::<Vec<_>>(),
            "conversation_length": self.conversation.history.len(),
            "timestamp": Local::now().timestamp_millis(),
        })
    }

    /// Get the unique identifier of this agent rearrange instance
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the name of this agent rearrange instance
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the description of this agent rearrange instance
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the current flow pattern
    pub fn flow(&self) -> &str {
        &self.flow
    }

    /// Get the list of agent names
    pub fn agent_names(&self) -> Vec<&String> {
        self.agents.keys().collect()
    }

    /// Get the conversation history
    pub fn conversation(&self) -> &AgentConversation {
        &self.conversation
    }

    /// Get the maximum number of loops
    pub fn max_loops(&self) -> u32 {
        self.max_loops
    }

    /// Get the number of agents in the swarm
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

/// Convenience function to create and run an agent rearrangement task
///
/// # Arguments
///
/// * `name` - Name of the agent rearrange instance
/// * `description` - Description of the purpose
/// * `agents` - Vector of agents to use
/// * `flow` - Flow pattern for execution
/// * `task` - Task to execute
/// * `img` - Optional image path
///
/// # Returns
///
/// `Result<String, AgentRearrangeError>` - The result of executing the task
///
/// # Examples
///
/// ```rust,no_run
/// use swarms_rs::structs::rearrange::rearrange;
///
/// async fn example() {
///     let result = rearrange(
///         "TaskProcessor",
///         "Processes tasks through agents",
///         vec![/* agents */],
///         "agent1 -> agent2",
///         "Analyze this data",
///         None,
///     ).await;
/// }
/// ```
/// Convenience function to create and run an agent rearrangement task
///
/// # Arguments
///
/// * `name` - Name of the agent rearrange instance
/// * `description` - Description of the purpose
/// * `agents` - Vector of agents to use
/// * `flow` - Flow pattern for execution
/// * `task` - Task to execute
/// * `img` - Optional image path
///
/// # Returns
///
/// `Result<String, AgentRearrangeError>` - The result of executing the task
///
/// # Examples
///
/// ```rust,no_run
/// use swarms_rs::structs::rearrange::rearrange;
///
/// async fn example() {
///     let result = rearrange(
///         "TaskProcessor",
///         "Processes tasks through agents",
///         vec![/* agents */],
///         "agent1 -> agent2",
///         "Analyze this data",
///         None,
///     ).await;
/// }
/// ```
pub async fn rearrange(
    name: impl Into<String>,
    description: impl Into<String>,
    agents: Vec<Box<dyn Agent>>,
    flow: impl Into<String>,
    task: impl Into<String>,
    img: Option<String>,
) -> Result<String, AgentRearrangeError> {
    let mut agent_system = AgentRearrange::builder()
        .name(name)
        .description(description)
        .agents(agents)
        .flow(flow)
        .build();

    agent_system.run_internal(task, img, None).await
}

impl Swarm for AgentRearrange {
    fn run(&self, task: String) -> BoxFuture<'_, Result<Box<dyn ErasedSerialize>, SwarmError>> {
        Box::pin(async move {
            // Create a mutable clone to work with
            let mut rearrange_clone = self.clone_for_task();

            match rearrange_clone.run_internal(task, None, None).await {
                Ok(result) => {
                    let serialized: Box<dyn ErasedSerialize> = Box::new(result);
                    Ok(serialized)
                },
                Err(e) => Err(SwarmError::AgentRearrangeError(e)),
            }
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
