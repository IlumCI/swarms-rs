use std::env;

use anyhow::Result;
use swarms_rs::llm::provider::openai::OpenAI;
use swarms_rs::structs::hierarchical_swarm::{HierarchicalSwarmBuilder, HierarchicalSwarm};
use swarms_rs::structs::swarms_client::{AgentSpec, SwarmsClient};

/// **SIMPLE MARKDOWN USAGE EXAMPLE**
/// 
/// Enable beautiful markdown output with just one line:
/// ```rust
/// let swarm = HierarchicalSwarmBuilder::new()
///     .name("My Swarm")
///     .md(true)  // 🎨 That's it! Everything renders automatically
///     .build()?;
/// 
/// // Execute - all agents get beautiful output automatically!
/// let results = swarm.run("My task", None).await?;
/// ```
/// 
/// **What happens automatically when .md(true):**
/// - Director planning gets beautiful borders and markdown
/// - Each worker agent gets bordered output with markdown
/// - Feedback and evaluation get professional formatting
/// - Workflow completion gets summary with markdown
/// - No manual formatter management needed!
/// - Everything renders across the whole swarm automatically!

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_line_number(true)
        .with_file(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Set up API keys - prioritize OpenAI for fallback
    let _api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        env::var("SWARMS_API_KEY").unwrap_or_else(|_| {
            "API_KEY_HERE".to_string()
        })
    });

    // Create Swarms client with OpenAI fallback
    let client = SwarmsClient::builder()
        .unwrap()
        .api_key("dummy-swarms-key") // Force OpenAI fallback
        .openai_api_key("API_KEY_HERE")
        .enable_openai_fallback(true)
        .max_concurrent_requests(1000)
        .circuit_breaker_threshold(1000)
        .timeout(std::time::Duration::from_secs(60))
        .max_retries(3)
        .build()
        .expect("Failed to create Swarms client");

    // Step 1: Create the Director Agent (coordinates everything according to architecture)
    let director_agent = AgentSpec {
        agent_name: "Director".to_string(),
        description: Some("Director Agent that coordinates and orchestrates the entire societal planning process".to_string()),
        system_prompt: Some(r#"You are a Director Agent responsible for coordinating the planning of a perfect communist society in post-civil war USA. Your role is to:

1. ANALYZE the post-war situation and societal needs
2. CREATE a comprehensive plan for communist society implementation
3. GENERATE specific orders for specialized planning agents
4. EVALUATE results from agents and provide feedback
5. DECIDE if more iterations are needed for refinement

Always respond with a JSON structure containing:
- "plan": A detailed step-by-step plan for communist society implementation
- "orders": Array of orders for specific worker agents with clear tasks

Example format:
{
  "plan": "Step 1: Analyze post-war conditions... Step 2: Design economic system...",
  "orders": [
    {"agent_name": "Economic Architect", "task": "Design the communist economic system..."},
    {"agent_name": "Social Engineer", "task": "Plan social structures and governance..."}
  ]
}"#.to_string()),
        model_name: "gpt-4o-mini".to_string(),
        auto_generate_prompt: false,
        max_tokens: 2000,
        temperature: 0.3,
        role: Some("director".to_string()),
        max_loops: 1,
        tools_dictionary: None,
    };

    // Step 2: Create specialized Worker Agents (execute specific tasks)
    let economic_architect = AgentSpec {
        agent_name: "Economic Architect".to_string(),
        description: Some("Worker Agent designing the communist economic system".to_string()),
        system_prompt: Some("You are an Economic Architect Worker Agent. Design comprehensive communist economic systems including production, distribution, resource allocation, and worker ownership. Focus on eliminating class distinctions, ensuring equal access to resources, and creating sustainable economic structures. Execute the task assigned by the Director Agent.".to_string()),
        model_name: "gpt-4o-mini".to_string(),
        auto_generate_prompt: false,
        max_tokens: 2000,
        temperature: 0.4,
        role: Some("worker".to_string()),
        max_loops: 1,
        tools_dictionary: None,
    };

    let social_planner = AgentSpec {
        agent_name: "Social Planner".to_string(),
        description: Some("Worker Agent planning social structures and governance".to_string()),
        system_prompt: Some("You are a Social Planner Worker Agent. Design communist social structures including governance systems, community organization, education, healthcare, and social services. Focus on collective decision-making, equal rights, and community solidarity. Execute the task assigned by the Director Agent.".to_string()),
        model_name: "gpt-4o-mini".to_string(),
        auto_generate_prompt: false,
        max_tokens: 2000,
        temperature: 0.3,
        role: Some("worker".to_string()),
        max_loops: 1,
        tools_dictionary: None,
    };

    let infrastructure_engineer = AgentSpec {
        agent_name: "Infrastructure Engineer".to_string(),
        description: Some("Worker Agent planning physical infrastructure and technology".to_string()),
        system_prompt: Some("You are an Infrastructure Engineer Worker Agent. Design communist infrastructure including housing, transportation, energy systems, technology integration, and public spaces. Focus on sustainability, accessibility, and collective ownership. Execute the task assigned by the Director Agent.".to_string()),
        model_name: "gpt-4o-mini".to_string(),
        auto_generate_prompt: false,
        max_tokens: 2000,
        temperature: 0.4,
        role: Some("worker".to_string()),
        max_loops: 1,
        tools_dictionary: None,
    };

    let governance_specialist = AgentSpec {
        agent_name: "Governance Specialist".to_string(),
        description: Some("Worker Agent designing cultural and educational systems".to_string()),
        system_prompt: Some("You are a Governance Specialist Worker Agent. Design communist cultural systems including education, arts, media, recreation, and community activities. Focus on collective cultural development, critical thinking, and cultural equality. Execute the task assigned by the Director Agent.".to_string()),
        model_name: "gpt-4o-mini".to_string(),
        auto_generate_prompt: false,
        max_tokens: 2000,
        temperature: 0.5,
        role: Some("worker".to_string()),
        max_loops: 1,
        tools_dictionary: None,
    };

    // Step 3: Build the Hierarchical Swarm following the architecture
    let swarm = HierarchicalSwarmBuilder::new()
        .name("Post-Civil War Communist Society Planning Swarm")
        .description("A hierarchical swarm that plans a post-civil war communist society")
        .agent(director_agent)
        .agent(economic_architect)
        .agent(social_planner)
        .agent(infrastructure_engineer)
        .agent(governance_specialist)
        .max_loops(2)
        .sequential_execution(true) // Use sequential execution with memory for better results
        .md(true) // 🎨 ENABLE BEAUTIFUL MARKDOWN OUTPUT - Simple one-liner!
        .client(client)
        .build()
        .expect("Failed to create hierarchical swarm");

    // Define the task
    let task = "Design a perfect communist society for modern USA after a civil war in 2030. The civil war was triggered by the Trump administration declaring an American Empire, leading to widespread social unrest and eventual collapse of the capitalist system. Plan for economic equality, collective ownership, social justice, sustainable infrastructure, and cultural transformation. Provide comprehensive implementation strategies for transitioning from post-war chaos to a functioning communist society.";

    // Execute - everything renders automatically with beautiful markdown!
    // No manual formatter management needed - just .md(true) and go!
    match swarm.run(task, None).await {
        Ok(outputs) => {
            // Beautiful markdown output was automatically rendered for:
            // - Director planning and coordination
            // - Each worker agent execution
            // - Feedback and evaluation
            // - Workflow completion
            println!("✅ Communist society planning process completed successfully!");
            println!("📊 Generated {} comprehensive planning documents", outputs.len());
        }
        Err(e) => {
            println!("❌ Communist society planning failed: {:?}", e);
        }
    }

    println!("🎉 Post-civil war communist society planning demonstration completed!");
    println!("💡 Tip: Use .md(true) to enable beautiful markdown output for any swarm!");
    Ok(())
} 
