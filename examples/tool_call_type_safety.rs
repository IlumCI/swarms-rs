use swarms_rs::{
    structs::conversation::{AgentShortMemory, Role},
    agent::swarms_agent::ToolCallOutput,
};

/// Example demonstrating type-safe tool call storage and retrieval
fn main() {
    println!("=== Tool Call Type Safety Example ===\n");

    // Create a new agent memory
    let memory = AgentShortMemory::new();

    // Simulate a conversation with tool calls
    let task_id = "calculate_budget";

    // Add user message
    memory.add(
        task_id,
        "financial_agent",
        Role::User("user".to_string()),
        "Calculate my monthly budget for groceries and utilities",
    );

    // Simulate tool calls with type-safe storage
    let calculator_tool_calls = vec![
        ToolCallOutput {
            name: "calculator".to_string(),
            args: r#"{"operation": "add", "values": [500, 200, 150]}"#.to_string(),
            result: "850".to_string(),
        },
        ToolCallOutput {
            name: "currency_converter".to_string(),
            args: r#"{"amount": 850, "from": "USD", "to": "EUR"}"#.to_string(),
            result: "780.50".to_string(),
        },
    ];

    // Store tool calls with type safety
    memory.add_tool_calls(
        task_id,
        "financial_agent",
        Role::Assistant("financial_agent".to_string()),
        calculator_tool_calls,
    );

    // Add another tool call later
    let budget_analyzer_calls = vec![
        ToolCallOutput {
            name: "budget_analyzer".to_string(),
            args: r#"{"total": 850, "income": 3000, "category": "essentials"}"#.to_string(),
            result: r#"{"percentage": 28.33, "status": "reasonable", "recommendation": "within normal range"}"#.to_string(),
        },
    ];

    memory.add_tool_calls(
        task_id,
        "financial_agent",
        Role::Assistant("financial_agent".to_string()),
        budget_analyzer_calls,
    );

    // Retrieve and display typed tool call data
    if let Some(conversation) = memory.0.get(task_id) {
        println!("Conversation for task: {}", task_id);
        println!("Total messages: {}\n", conversation.history.len());

        // Get all tool calls
        let all_tool_calls = conversation.get_tool_calls();
        println!("All tool calls ({}):", all_tool_calls.len());
        for (i, tool_call) in all_tool_calls.iter().enumerate() {
            println!("  {}. {}: {} -> {}", i + 1, tool_call.name, tool_call.args, tool_call.result);
        }

        // Get tool calls by specific name
        let calculator_calls = conversation.get_tool_calls_by_name("calculator");
        println!("\nCalculator tool calls ({}):", calculator_calls.len());
        for call in calculator_calls {
            println!("  - Args: {}, Result: {}", call.args, call.result);
        }

        let analyzer_calls = conversation.get_tool_calls_by_name("budget_analyzer");
        println!("\nBudget analyzer calls ({}):", analyzer_calls.len());
        for call in analyzer_calls {
            println!("  - Args: {}, Result: {}", call.args, call.result);
        }

        // Get latest tool calls
        if let Some(latest_calls) = conversation.get_latest_tool_calls() {
            println!("\nLatest tool calls ({}):", latest_calls.len());
            for call in latest_calls {
                println!("  - {}: {} -> {}", call.name, call.args, call.result);
            }
        }

        // Demonstrate parsing typed data
        println!("\n=== Parsing Typed Data ===");
        for tool_call in all_tool_calls {
            match tool_call.name.as_str() {
                "calculator" => {
                    if let Ok(result) = tool_call.result.parse::<f64>() {
                        println!("Calculator result (parsed): ${:.2}", result);
                    }
                }
                "budget_analyzer" => {
                    if let Ok(analysis) = serde_json::from_str::<serde_json::Value>(&tool_call.result) {
                        if let Some(percentage) = analysis["percentage"].as_f64() {
                            println!("Budget percentage: {:.1}%", percentage);
                        }
                        if let Some(status) = analysis["status"].as_str() {
                            println!("Budget status: {}", status);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    println!("\n=== Benefits of Type-Safe Storage ===");
    println!("1. No need to parse formatted strings");
    println!("2. Direct access to structured tool call data");
    println!("3. Type-safe retrieval by tool name");
    println!("4. Easy integration with workflows");
    println!("5. Maintains backward compatibility with text storage");
} 
