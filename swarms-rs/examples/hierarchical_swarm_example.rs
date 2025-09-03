use std::env;
use anyhow::Result;
use swarms_rs::structs::hierarchical_swarm::HierarchicalSwarmBuilder;
use swarms_rs::structs::swarms_client::{AgentSpec, SwarmsClient};
use swarms_rs::utils::formatter::Formatter;

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

    // Create a formatter for the example output (OPTIONAL - for demo purposes)
    let mut formatter = Formatter::auto();
    
    formatter.render_section_header("Post-Civil War Communist Society Planning");
    formatter.render_info("This example demonstrates the hierarchical multi-agent system planning a perfect communist society in modern USA after a 2030 civil war");
    formatter.render_info("The core swarm prioritizes PERFORMANCE, but we'll use optional beautiful formatting for this demo");

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
        .verbose(false) // Disable verbose logging for performance
        .interactive(false) // Disable interactive dashboard
        .sequential_execution(true) // Use sequential execution with memory for better results
        .client(client)
        .build()
        .expect("Failed to create hierarchical swarm");

    formatter.render_success("Post-civil war communist society planning swarm created successfully!");
    formatter.render_info(&format!("Swarm configuration: {}", swarm.get_configuration()));
    formatter.render_info("Note: Core swarm is PERFORMANCE OPTIMIZED - beautiful output is optional for demos");

    // Step 4: Define the User Task
    let task = "Design a perfect communist society for modern USA after a civil war in 2030. The civil war was triggered by the Trump administration declaring an American Empire, leading to widespread social unrest and eventual collapse of the capitalist system. Plan for economic equality, collective ownership, social justice, sustainable infrastructure, and cultural transformation. Provide comprehensive implementation strategies for transitioning from post-war chaos to a functioning communist society.";

    formatter.render_section_header("User Task");
    formatter.render_info(&format!("Task: {}", task));

    // Step 5: Execute the Hierarchical Process 
    formatter.render_section_header("Execution Options");
    formatter.render_info("Option 1: PERFORMANCE MODE (default) - Maximum speed, minimal formatting");
    formatter.render_info("Option 2: DEMO MODE (optional) - Beautiful agent output with markdown");

    // OPTION 1: Performance mode (default) - Maximum speed
    formatter.render_section_header("PERFORMANCE MODE: Maximum Speed Execution");
    let start_time = std::time::Instant::now();
    
    match swarm.run(task, None).await {
        Ok(outputs) => {
            let duration = start_time.elapsed();
            formatter.render_success(&format!("Communist society planning completed in {:?}!", duration));
            formatter.render_info(&format!("Generated {} comprehensive planning documents", outputs.len()));
            formatter.render_info("Core swarm executed at MAXIMUM SPEED with minimal formatting overhead");
        }
        Err(e) => {
            formatter.render_error(&format!("Communist society planning failed: {:?}", e));
        }
    }

    // OPTION 2: Demo mode with beautiful formatting (optional)
    formatter.render_section_header("DEMO MODE: Beautiful Agent Output (Optional)");
    formatter.render_info("Now executing with beautiful agent-level markdown formatting for demonstration...");

    // Create a demo formatter for beautiful output
    let mut demo_formatter = swarm.create_formatter();
    let start_time_demo = std::time::Instant::now();

    match swarm.run_with_formatter(task, None, &mut demo_formatter).await {
        Ok(outputs) => {
            let duration = start_time_demo.elapsed();
            formatter.render_success(&format!("Beautiful demo execution completed in {:?}!", duration));
            formatter.render_info(&format!("Generated {} comprehensive planning documents with beautiful formatting", outputs.len()));
            
            // Show individual agent outputs with beautiful formatting
            for (i, output) in outputs.iter().enumerate() {
                if !output.trim().is_empty() {
                    demo_formatter.render_agent_output(&format!("Planning Agent {}", i + 1), output);
                }
            }
        }
        Err(e) => {
            formatter.render_error(&format!("Beautiful demo execution failed: {:?}", e));
        }
    }

    // Step 6: Show the hierarchical process summary
    formatter.render_section_header("Performance vs Beauty Trade-off Summary");
    formatter.render_info("The system provides two execution modes:");
    formatter.render_info("1. ✓ PERFORMANCE MODE: Maximum speed, minimal formatting (default)");
    formatter.render_info("2. ✓ DEMO MODE: Beautiful agent output with markdown (optional)");
    formatter.render_info("3. ✓ Core swarm prioritizes SPEED over visual formatting");
    formatter.render_info("4. ✓ Beautiful output is available when explicitly requested");
    formatter.render_info("5. ✓ Agent-level markdown control for demos and examples");
    formatter.render_info("6. ✓ Production use gets maximum performance");
    formatter.render_info("7. ✓ Demo use gets beautiful visual separation");

    formatter.render_workflow_completion("Performance vs Beauty demonstration");

    Ok(())
} 
