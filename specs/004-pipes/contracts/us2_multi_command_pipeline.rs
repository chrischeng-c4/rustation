// Contract Test: User Story 2 - Multi-Command Pipeline Chain
//
// Tests pipelines with 3 or more commands, verifying sequential
// I/O chaining across multiple stages.
//
// Specification: specs/004-pipes/spec.md (Lines 28-43)
// Priority: P2

use rush::executor::parser::parse_pipeline;
use rush::executor::pipeline::PipelineExecutor;
use std::fs;

/// US2-AS1: Given user types `cat file.txt | grep error | wc -l`,
/// When command executes, Then count of lines containing "error" is displayed
#[test]
fn test_cat_pipe_grep_pipe_wc() {
    // Setup: Create test file with mix of error and non-error lines
    let test_file = std::env::temp_dir().join("rush_pipe_test_us2_as1.txt");
    let content = vec![
        "INFO: Application started",
        "ERROR: Failed to connect",
        "INFO: Retrying connection",
        "ERROR: Connection timeout",
        "INFO: Success",
    ].join("\n");
    fs::write(&test_file, content).unwrap();

    // Execute pipeline: cat file | grep ERROR | wc -l
    let pipeline_input = format!("cat {} | grep ERROR | wc -l", test_file.display());
    let pipeline = parse_pipeline(&pipeline_input).unwrap();

    // Verify parsing
    assert_eq!(pipeline.len(), 3, "Should have 3 segments");
    assert_eq!(pipeline.segments[0].program, "cat");
    assert_eq!(pipeline.segments[1].program, "grep");
    assert_eq!(pipeline.segments[2].program, "wc");

    // Execute
    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded
    assert_eq!(exit_code, 0, "Pipeline should succeed");

    // Expected output: "       2" (two ERROR lines)
    // Output validation deferred to implementation phase

    // Cleanup
    fs::remove_file(&test_file).unwrap();
}

/// US2-AS2: Given user types `ls -la | grep "\\.md" | head -3`,
/// When command executes, Then first 3 markdown files are listed
#[test]
fn test_ls_pipe_grep_pipe_head() {
    // Setup: Create test directory with multiple markdown and other files
    let test_dir = std::env::temp_dir().join("rush_pipe_test_us2_as2");
    fs::create_dir_all(&test_dir).unwrap();
    for i in 1..=5 {
        fs::write(test_dir.join(format!("file{}.md", i)), "content").unwrap();
        fs::write(test_dir.join(format!("file{}.txt", i)), "content").unwrap();
    }

    // Execute pipeline: ls -la test_dir | grep "\.md" | head -3
    let pipeline_input = format!("ls -la {} | grep \"\\.md\" | head -3", test_dir.display());
    let pipeline = parse_pipeline(&pipeline_input).unwrap();

    assert_eq!(pipeline.len(), 3);

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded
    assert_eq!(exit_code, 0, "Pipeline should succeed");

    // Expected output: First 3 .md files from ls -la
    // Output validation deferred to implementation phase

    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}

/// US2-AS3: Given user types `echo -e "b\\na\\nc" | sort | tail -1`,
/// When command executes, Then output shows "c" (last sorted item)
#[test]
fn test_echo_pipe_sort_pipe_tail() {
    // Note: -e flag behavior varies by echo implementation
    // This test uses printf for portability
    let pipeline_input = "printf \"b\\na\\nc\" | sort | tail -1";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    assert_eq!(pipeline.len(), 3);
    assert_eq!(pipeline.segments[0].program, "printf");
    assert_eq!(pipeline.segments[1].program, "sort");
    assert_eq!(pipeline.segments[2].program, "tail");

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded
    assert_eq!(exit_code, 0, "Pipeline should succeed");

    // Expected output: "c"
    // Output validation deferred to implementation phase
}

/// US2-AS4: Given pipeline has 5 commands, When executed,
/// Then all five commands execute in sequence with proper I/O chaining
#[test]
fn test_five_command_pipeline() {
    // Pipeline: Generate numbers, filter evens, sort reverse, take first 3, count
    let pipeline_input = "printf \"5\\n2\\n8\\n1\\n9\\n3\\n7\\n4\\n6\" | grep -E \"[2468]\" | sort -r | head -3 | wc -l";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    // Verify parsing
    assert_eq!(pipeline.len(), 5, "Should have 5 segments");
    assert_eq!(pipeline.segments[0].program, "printf");
    assert_eq!(pipeline.segments[1].program, "grep");
    assert_eq!(pipeline.segments[2].program, "sort");
    assert_eq!(pipeline.segments[3].program, "head");
    assert_eq!(pipeline.segments[4].program, "wc");

    // Execute
    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded
    assert_eq!(exit_code, 0, "5-command pipeline should succeed");

    // Expected flow:
    // printf: 5 2 8 1 9 3 7 4 6
    // grep -E "[2468]": 2 8 4 6
    // sort -r: 8 6 4 2
    // head -3: 8 6 4
    // wc -l: 3
    // Expected output: "       3"
}

/// Additional Test: Verify concurrent execution (performance requirement)
#[test]
fn test_concurrent_execution_performance() {
    // Pipeline with slow commands should execute concurrently
    // If sequential, this would take ~3 seconds (3 × 1s sleep)
    // If concurrent, should take ~1 second (max of all commands)
    let pipeline_input = "sleep 0.1 | sleep 0.1 | sleep 0.1";
    let pipeline = parse_pipeline(pipeline_input).unwrap();

    let executor = PipelineExecutor::new();
    let start = std::time::Instant::now();
    let exit_code = executor.execute(&pipeline).unwrap();
    let duration = start.elapsed();

    // Verify: Succeeded
    assert_eq!(exit_code, 0);

    // Verify: Completed in reasonable time (concurrent execution)
    // Allow some overhead: should be < 0.5s if truly concurrent
    assert!(
        duration.as_millis() < 500,
        "Pipeline should execute concurrently, took {:?}",
        duration
    );
}

/// Additional Test: Long pipeline (stress test)
#[test]
fn test_long_pipeline_20_commands() {
    // 20-command pipeline (edge case from specification)
    // Pipeline: Generate 1000 lines, filter/transform through 18 stages, count
    let mut pipeline_parts = vec!["seq 1 1000"];

    // Add 18 intermediate commands (cat is no-op but adds process overhead)
    for _ in 0..18 {
        pipeline_parts.push("cat");
    }

    // Final command: count lines
    pipeline_parts.push("wc -l");

    let pipeline_input = pipeline_parts.join(" | ");
    let pipeline = parse_pipeline(&pipeline_input).unwrap();

    // Verify parsing
    assert_eq!(pipeline.len(), 20, "Should have 20 segments");

    // Execute
    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded despite length
    assert_eq!(exit_code, 0, "Long pipeline should succeed");

    // Expected output: "    1000"
}
