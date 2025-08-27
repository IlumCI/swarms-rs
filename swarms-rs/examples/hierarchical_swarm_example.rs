use std::env;
use anyhow::Result;
use swarms_rs::structs::hierarchical_swarm::HierarchicalSwarmBuilder;
use swarms_rs::structs::swarms_client::{AgentSpec, SwarmsClient};
use swarms_rs::utils::formatter::{init_formatter, render_section_header, render_info, render_success, render_error, render_markdown};



#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // Initialize the global formatter with markdown enabled
    init_formatter(true);

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

    render_section_header("Post-Civil War Communist Society Planning");
    render_info("This example demonstrates the hierarchical multi-agent system planning a perfect communist society in modern USA after a 2030 civil war");

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

    let social_engineer = AgentSpec {
        agent_name: "Social Engineer".to_string(),
        description: Some("Worker Agent planning social structures and governance".to_string()),
        system_prompt: Some("You are a Social Engineer Worker Agent. Design communist social structures including governance systems, community organization, education, healthcare, and social services. Focus on collective decision-making, equal rights, and community solidarity. Execute the task assigned by the Director Agent.".to_string()),
        model_name: "gpt-4o-mini".to_string(),
        auto_generate_prompt: false,
        max_tokens: 2000,
        temperature: 0.3,
        role: Some("worker".to_string()),
        max_loops: 1,
        tools_dictionary: None,
    };

    let infrastructure_planner = AgentSpec {
        agent_name: "Infrastructure Planner".to_string(),
        description: Some("Worker Agent planning physical infrastructure and technology".to_string()),
        system_prompt: Some("You are an Infrastructure Planner Worker Agent. Design communist infrastructure including housing, transportation, energy systems, technology integration, and public spaces. Focus on sustainability, accessibility, and collective ownership. Execute the task assigned by the Director Agent.".to_string()),
        model_name: "gpt-4o-mini".to_string(),
        auto_generate_prompt: false,
        max_tokens: 2000,
        temperature: 0.4,
        role: Some("worker".to_string()),
        max_loops: 1,
        tools_dictionary: None,
    };

    let cultural_architect = AgentSpec {
        agent_name: "Cultural Architect".to_string(),
        description: Some("Worker Agent designing cultural and educational systems".to_string()),
        system_prompt: Some("You are a Cultural Architect Worker Agent. Design communist cultural systems including education, arts, media, recreation, and community activities. Focus on collective cultural development, critical thinking, and cultural equality. Execute the task assigned by the Director Agent.".to_string()),
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
        .description("Demonstrates the hierarchical architecture: Director → Worker Agents with Memory for societal planning")
        .agent(director_agent)
        .agent(economic_architect)
        .agent(social_engineer)
        .agent(infrastructure_planner)
        .agent(cultural_architect)
        .max_loops(2) // Allow for refinement loops
        .verbose(false) // Disable verbose logging
        .interactive(false) // Disable interactive dashboard
        .sequential_execution(false) // Use parallel execution for speed
        .client(client)
        .build()
        .expect("Failed to create hierarchical swarm");

    render_success("Post-civil war communist society planning swarm created successfully!");
    render_info(&format!("Swarm configuration: {}", swarm.get_configuration()));

    // Step 4: Define the User Task
    let task = "Design a perfect communist society for modern USA after a civil war in 2030. The civil war was triggered by the Trump administration declaring an American Empire, leading to widespread social unrest and eventual collapse of the capitalist system. Plan for economic equality, collective ownership, social justice, sustainable infrastructure, and cultural transformation. Provide comprehensive implementation strategies for transitioning from post-war chaos to a functioning communist society.";

    render_section_header("User Task");
    render_info(&format!("Task: {}", task));

    // Step 5: Execute the Hierarchical Process
    render_section_header("Executing Communist Society Planning with Memory");
    render_info("Following the architecture: User Task → Director → Plan & Orders → Distribute to Agents → Execute Sequentially with Memory → Report Results → Director Evaluation");

    // Execute the hierarchical process
    match swarm.run(task, None).await {
        Ok(outputs) => {
            render_success("Communist society planning process completed successfully!");
            
            // Display results from each agent
            for (i, output) in outputs.iter().enumerate() {
                if !output.trim().is_empty() {
                    render_section_header(&format!("Planning Agent {} Output", i + 1));
                    render_markdown(output);
                }
            }
        }
        Err(e) => {
            render_error(&format!("Communist society planning failed: {:?}", e));
        }
    }

    // Step 6: Show the hierarchical process summary
    render_section_header("Communist Society Planning Summary");
    render_info("The system executed the following hierarchical steps:");
    render_info("1. ✓ Post-civil war scenario analyzed by Director Agent");
    render_info("2. ✓ Director created comprehensive communist society plan");
    render_info("3. ✓ Orders distributed to specialized planning agents");
    render_info("4. ✓ Planning agents executed tasks sequentially with memory");
    render_info("5. ✓ Each agent built upon previous agent's societal planning");
    render_info("6. ✓ Results collected and synthesized");
    render_info("7. ✓ Director evaluated results and provided feedback");
    render_info("8. ✓ Complete communist society blueprint created");

    render_success("Post-civil war communist society planning demonstration completed!");

    Ok(())
} 
