use swarms_rs::{
    agent::SwarmsAgentBuilder,
    llm::provider::openai::OpenAI,
    structs::{
        agent::AgentConfig,
        execute_agent_batch::{AgentBatchExecutor, BatchConfigBuilder},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create OpenAI model
    let openai = OpenAI::new("gpt-4o-mini".to_string());

    // Create multiple agents with different roles
    let researcher_agent = SwarmsAgentBuilder::new_with_model(openai.clone())
        .agent_name("Researcher")
        .description("A researcher agent that analyzes and gathers information")
        .config(AgentConfig::default())
        .build();

    let writer_agent = SwarmsAgentBuilder::new_with_model(openai.clone())
        .agent_name("Writer")
        .description("A writer agent that creates content")
        .config(AgentConfig::default())
        .build();

    let editor_agent = SwarmsAgentBuilder::new_with_model(openai)
        .agent_name("Editor")
        .description("An editor agent that reviews and refines content")
        .config(AgentConfig::default())
        .build();

    // Create batch configuration
    let batch_config = BatchConfigBuilder::default()
        .max_concurrent_tasks(3)
        .auto_cpu_optimization(true)
        .build();

    // Create batch executor
    let batch_executor = AgentBatchExecutor::builder()
        .add_agent(Box::new(researcher_agent))
        .add_agent(Box::new(writer_agent))
        .add_agent(Box::new(editor_agent))
        .config(batch_config)
        .build();

    // Define tasks
    let tasks = vec![
        "Research the impact of artificial intelligence on healthcare".to_string(),
        "Write an article about sustainable energy solutions".to_string(),
        "Analyze the future of remote work".to_string(),
    ];

    // Execute batch
    let results = batch_executor.execute_batch(tasks).await?;

    // Process results
    for (task, conversation) in results {
        println!("\nTask: {}", task);
        println!("Conversation history:");
        for message in &conversation.history {
            println!("{}: {}", message.role, message.content);
        }
    }

    Ok(())
}
