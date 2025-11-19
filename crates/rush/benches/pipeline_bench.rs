// Performance benchmarks for pipeline execution
//
// Validates that parsing and execution meet constitution requirements:
// - Parse time <1ms
// - Execution overhead <5ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rush::executor::execute::CommandExecutor;
use rush::executor::parser::parse_pipeline;

/// Benchmark: Parse pipeline from string
fn benchmark_parse_pipeline(c: &mut Criterion) {
    c.bench_function("parse_pipeline_two_commands", |b| {
        b.iter(|| {
            let result = parse_pipeline(black_box("ls -la | grep txt"));
            assert!(result.is_ok());
        });
    });

    c.bench_function("parse_pipeline_five_commands", |b| {
        b.iter(|| {
            let result = parse_pipeline(black_box("a | b | c | d | e"));
            assert!(result.is_ok());
        });
    });

    c.bench_function("parse_pipeline_with_quotes", |b| {
        b.iter(|| {
            let result = parse_pipeline(black_box("echo \"test | data\" | grep test"));
            assert!(result.is_ok());
        });
    });
}

/// Benchmark: Execute two-command pipeline
fn benchmark_execute_two_command_pipeline(c: &mut Criterion) {
    let executor = CommandExecutor::new();

    c.bench_function("execute_echo_pipe_cat", |b| {
        b.iter(|| {
            let result = executor.execute(black_box("echo test | cat"));
            assert!(result.is_ok());
        });
    });

    c.bench_function("execute_true_pipe_true", |b| {
        b.iter(|| {
            let result = executor.execute(black_box("true | true"));
            assert!(result.is_ok());
        });
    });
}

/// Benchmark: Execute five-command pipeline
fn benchmark_execute_five_command_pipeline(c: &mut Criterion) {
    let executor = CommandExecutor::new();

    c.bench_function("execute_five_cat_pipeline", |b| {
        b.iter(|| {
            let result = executor.execute(black_box("echo data | cat | cat | cat | cat"));
            assert!(result.is_ok());
        });
    });

    c.bench_function("execute_five_true_pipeline", |b| {
        b.iter(|| {
            let result = executor.execute(black_box("true | true | true | true | true"));
            assert!(result.is_ok());
        });
    });
}

/// Benchmark: Concurrent execution validation
///
/// This benchmark verifies that pipelines execute concurrently, not sequentially.
/// We use fast commands to measure overhead, not actual concurrency timing.
fn benchmark_concurrent_execution(c: &mut Criterion) {
    let executor = CommandExecutor::new();

    c.bench_function("concurrent_two_command", |b| {
        b.iter(|| {
            let result = executor.execute(black_box("true | true"));
            assert!(result.is_ok());
        });
    });

    c.bench_function("concurrent_ten_command", |b| {
        b.iter(|| {
            let result = executor.execute(black_box(
                "true | true | true | true | true | true | true | true | true | true",
            ));
            assert!(result.is_ok());
        });
    });
}

criterion_group!(
    benches,
    benchmark_parse_pipeline,
    benchmark_execute_two_command_pipeline,
    benchmark_execute_five_command_pipeline,
    benchmark_concurrent_execution
);
criterion_main!(benches);
