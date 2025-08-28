use criterion::{Criterion, black_box, criterion_group, criterion_main, Throughput, BenchmarkId};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

use swarms_rs::{
    agent::SwarmsAgentBuilder,
    llm::provider::openai::OpenAI,
    structs::agent::AgentConfig,
};

fn run_async<F: std::future::Future<Output = T>, T>(future: F) -> T {
    let rt = Runtime::new().unwrap();
    rt.block_on(future)
}

/// Create a simple agent configuration for benchmarking
fn create_simple_config(name: &str) -> AgentConfig {
    AgentConfig::builder()
        .agent_name(name)
        .user_name("User")
        .description("Simple benchmark agent")
        .temperature(0.1)
        .max_loops(1)
        .max_tokens(100)
        .retry_attempts(0) // No retry attempts for faster initialization
        .build()
        .as_ref()
        .clone()
}

/// Create a mock OpenAI provider for testing
fn create_mock_openai() -> OpenAI {
    OpenAI::new("mock-api-key".to_string())
}

/// Benchmark single agent initialization speed
fn bench_single_agent_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_initialization");

    group.bench_function("single_agent_init", |b| {
        b.iter(|| {
            let openai = create_mock_openai();
            let config = create_simple_config("BenchAgent");
            
            let _agent = SwarmsAgentBuilder::new_with_model(openai)
                .config(config)
                .disable_task_complete_tool() // Disable tools for faster init
                .build();
            
            black_box(_agent);
        });
    });

    group.finish();
}

/// Benchmark batch agent initialization with different sizes
fn bench_batch_agent_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_agent_initialization");

    for &batch_size in &[1, 5, 10, 25, 50, 100] {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_size", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut agents = Vec::with_capacity(batch_size);
                    
                    for i in 0..batch_size {
                        let openai = create_mock_openai();
                        let config = create_simple_config(&format!("Agent{}", i));
                        
                        let agent = SwarmsAgentBuilder::new_with_model(openai)
                            .config(config)
                            .disable_task_complete_tool()
                            .build();
                        
                        agents.push(agent);
                    }
                    
                    black_box(agents);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark how many agents can be initialized in one minute
fn bench_agents_per_minute(c: &mut Criterion) {
    let mut group = c.benchmark_group("agents_per_minute");
    
    // Set a longer measurement time for this benchmark
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10); // Fewer samples since each takes a minute
    
    group.bench_function("max_agents_in_60_seconds", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            let mut total_agents = 0;
            
            for _ in 0..iters {
                let init_start = Instant::now();
                let mut count = 0;
                
                // Initialize agents for 60 seconds
                while init_start.elapsed() < Duration::from_secs(60) {
                    let openai = create_mock_openai();
                    let config = create_simple_config(&format!("SpeedAgent{}", count));
                    
                    let _agent = SwarmsAgentBuilder::new_with_model(openai)
                        .config(config)
                        .disable_task_complete_tool()
                        .build();
                    
                    count += 1;
                    black_box(_agent);
                }
                
                total_agents += count;
                println!("Initialized {} agents in 60 seconds (iteration {})", count, total_agents);
            }
            
            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark concurrent agent initialization
fn bench_concurrent_agent_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_agent_initialization");

    for &num_concurrent in &[2, 4, 8, 16, 32] {
        group.throughput(Throughput::Elements(num_concurrent as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent", num_concurrent),
            &num_concurrent,
            |b, &num_concurrent| {
                b.iter(|| {
                    run_async(async {
                        let handles: Vec<_> = (0..num_concurrent)
                            .map(|i| {
                                tokio::spawn(async move {
                                    let openai = create_mock_openai();
                                    let config = create_simple_config(&format!("ConcurrentAgent{}", i));
                                    
                                    SwarmsAgentBuilder::new_with_model(openai)
                                        .config(config)
                                        .disable_task_complete_tool()
                                        .build()
                                })
                            })
                            .collect();

                        let agents = futures::future::join_all(handles).await;
                        black_box(agents);
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark initialization with different configurations
fn bench_different_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_variants");

    // Minimal configuration
    group.bench_function("minimal_config", |b| {
        b.iter(|| {
            let openai = create_mock_openai();
            let config = AgentConfig::builder()
                .agent_name("MinimalAgent")
                .max_loops(1)
                .retry_attempts(0)
                .build()
                .as_ref()
                .clone();
            
            let _agent = SwarmsAgentBuilder::new_with_model(openai)
                .config(config)
                .disable_task_complete_tool()
                .build();
            
            black_box(_agent);
        });
    });

    // Full configuration
    group.bench_function("full_config", |b| {
        b.iter(|| {
            let openai = create_mock_openai();
            let config = AgentConfig::builder()
                .agent_name("FullAgent")
                .user_name("TestUser")
                .description("Full configuration agent for benchmarking")
                .temperature(0.7)
                .max_loops(1)
                .build()
                .as_ref()
                .clone();
            
            let _agent = SwarmsAgentBuilder::new_with_model(openai)
                .config(config)
                .build(); // Keep task evaluator tool for full config
            
            black_box(_agent);
        });
    });

    group.finish();
}

/// Quick test to measure initialization rate
fn quick_initialization_rate_test() {
    println!("\n=== Quick Agent Initialization Rate Test ===");
    
    let start = Instant::now();
    let mut count = 0;
    let test_duration = Duration::from_secs(10); // 10 second test
    
    while start.elapsed() < test_duration {
        let openai = create_mock_openai();
        let config = create_simple_config(&format!("QuickTest{}", count));
        
        let _agent = SwarmsAgentBuilder::new_with_model(openai)
            .config(config)
            .disable_task_complete_tool()
            .build();
        
        count += 1;
    }
    
    let elapsed = start.elapsed();
    let rate = count as f64 / elapsed.as_secs_f64();
    
    println!("Initialized {} agents in {:.2}s", count, elapsed.as_secs_f64());
    println!("Rate: {:.2} agents/second", rate);
    println!("Estimated agents per minute: {:.0}", rate * 60.0);
    println!("=========================================\n");
}

/// Custom benchmark function to print initialization statistics
fn print_initialization_stats() {
    // Run the quick test before benchmarks
    quick_initialization_rate_test();
}

// Call the stats function at the start of benchmarks
#[ctor::ctor]
fn init() {
    print_initialization_stats();
}

criterion_group!(
    benches,
    bench_single_agent_initialization,
    bench_batch_agent_initialization,
    bench_agents_per_minute,
    bench_concurrent_agent_initialization,
    bench_different_configurations,
);
criterion_main!(benches);
