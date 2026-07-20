use std::env;
use swarms_rs::llm::provider::openai::OpenAI;
use swarms_rs::logging::init_logger;
use swarms_rs::prompts::multi_agent_collab_prompt_new::MULTI_AGENT_COLLAB_PROMPT_NEW;
use swarms_rs::structs::agent::AgentConfig;
use swarms_rs::structs::rearrange::{AgentRearrange, OutputType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Example 1: Using AgentRearrange directly
    println!("=== Example 1: Direct AgentRearrange Usage ===");

    // Create OpenAI provider with API key from environment
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  OPENAI_API_KEY not set, using demo key (will show API errors)");
        "demo-key".to_string()
    });
    let openai = OpenAI::new(api_key);

    // Note: In a real implementation, you would configure agents with specific system prompts
    // For this example, we use the default configuration

    let researcher_config = AgentConfig::builder()
        .agent_name("Researcher")
        .user_name("User")
        .description("Conducts research on given topics")
        .temperature(0.3)
        .max_loops(1)
        .build();

    let analyst_config = AgentConfig::builder()
        .agent_name("Analyst")
        .user_name("User")
        .description("Analyzes data and provides insights")
        .temperature(0.5)
        .max_loops(1)
        .build();

    let reviewer_config = AgentConfig::builder()
        .agent_name("Reviewer")
        .user_name("User")
        .description("R eviews and validates analysis")
        .temperature(0.2)
        .max_loops(1)
        .build();

    let summarizer_config = AgentConfig::builder()
        .agent_name("Summarizer")
        .user_name("User")
        .description("Creates comprehensive summaries")
        .temperature(0.4)
        .max_loops(1)
        .build();

    // Create SwarmsAgent instances using builder pattern with proper configs
    // Disable task_evaluator tool to avoid JSON parsing issues in this example
    let researcher = swarms_rs::agent::SwarmsAgentBuilder::new_with_model(openai.clone())
        .config((*researcher_config).clone())
        .system_prompt(MULTI_AGENT_COLLAB_PROMPT_NEW)
        .disable_task_complete_tool()
        .build();
    let analyst = swarms_rs::agent::SwarmsAgentBuilder::new_with_model(openai.clone())
        .config((*analyst_config).clone())
        .system_prompt(MULTI_AGENT_COLLAB_PROMPT_NEW)
        .disable_task_complete_tool()
        .build();
    let reviewer = swarms_rs::agent::SwarmsAgentBuilder::new_with_model(openai.clone())
        .config((*reviewer_config).clone())
        .system_prompt(MULTI_AGENT_COLLAB_PROMPT_NEW)
        .disable_task_complete_tool()
        .build();
    let summarizer = swarms_rs::agent::SwarmsAgentBuilder::new_with_model(openai.clone())
        .config((*summarizer_config).clone())
        .system_prompt(MULTI_AGENT_COLLAB_PROMPT_NEW)
        .disable_task_complete_tool()
        .build();

    // Convert to boxed agents
    let agents: Vec<Box<dyn swarms_rs::structs::agent::Agent>> = vec![
        Box::new(researcher),
        Box::new(analyst),
        Box::new(reviewer),
        Box::new(summarizer),
    ];

    // Create AgentRearrange with sequential flow
    let mut rearrange = AgentRearrange::builder()
        .name("Research Pipeline")
        .description("A comprehensive research and analysis pipeline")
        .agents(agents)
        .flow("Researcher -> Analyst, Reviewer, Summarizer")
        .max_loops(1)
        .output_type(OutputType::Final)
        .verbose(true)
        .rules("Always provide detailed and accurate information")
        .build();

    let result = rearrange.run("Analyze the current state of the cryptocurrency market, focusing on Bitcoin and Ethereum").await?;

    println!("Result: {}", result);

    Ok(())
}

// Now run the file
// cargo run --example agent_rearrange_example
