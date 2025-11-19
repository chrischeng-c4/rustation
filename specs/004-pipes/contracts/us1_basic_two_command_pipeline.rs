// Contract Test: User Story 1 - Basic Two-Command Pipeline
//
// Tests the fundamental pipe operator functionality for connecting
// two commands where stdout of first becomes stdin of second.
//
// Specification: specs/004-pipes/spec.md (Lines 10-25)
// Priority: P1 (MVP)

use rush::executor::parser::parse_pipeline;
use rush::executor::pipeline::PipelineExecutor;
use std::fs;
use std::process::Command;

/// US1-AS1: Given user types `ls | grep txt`, When command executes,
/// Then only lines containing "txt" from ls output are displayed
#[test]
fn test_ls_pipe_grep_txt() {
    // Setup: Create test files
    let test_dir = std::env::temp_dir().join("rush_pipe_test_us1_as1");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(test_dir.join("file1.txt"), "content").unwrap();
    fs::write(test_dir.join("file2.md"), "content").unwrap();
    fs::write(test_dir.join("file3.txt"), "content").unwrap();

    // Execute pipeline: ls test_dir | grep txt
    let pipeline_input = format!("ls {} | grep txt", test_dir.display());
    let pipeline = parse_pipeline(&pipeline_input).unwrap();
    let executor = PipelineExecutor::new();

    // Capture output
    // Note: This test validates exit code. Output validation requires
    // capturing stdout, which is added in implementation phase.
    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded (grep found matches)
    assert_eq!(exit_code, 0, "Pipeline should succeed when grep finds matches");

    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}

/// US1-AS2: Given user types `echo "test" | wc -l`, When command executes,
/// Then output shows "1" (one line counted)
#[test]
fn test_echo_pipe_wc() {
    let pipeline_input = "echo \"test\" | wc -l";
    let pipeline = parse_pipeline(pipeline_input).unwrap();
    let executor = PipelineExecutor::new();

    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded
    assert_eq!(exit_code, 0, "Pipeline should succeed");

    // Expected output: "       1" (wc -l format)
    // Output validation deferred to implementation phase
}

/// US1-AS3: Given user types `cat README.md | head -5`, When command executes,
/// Then first 5 lines of README.md are displayed
#[test]
fn test_cat_pipe_head() {
    // Setup: Create test file with 10 lines
    let test_file = std::env::temp_dir().join("rush_pipe_test_us1_as3.txt");
    let content = (1..=10).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
    fs::write(&test_file, content).unwrap();

    // Execute pipeline: cat test_file | head -5
    let pipeline_input = format!("cat {} | head -5", test_file.display());
    let pipeline = parse_pipeline(&pipeline_input).unwrap();
    let executor = PipelineExecutor::new();

    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded
    assert_eq!(exit_code, 0, "Pipeline should succeed");

    // Expected output: Lines 1-5
    // Output validation deferred to implementation phase

    // Cleanup
    fs::remove_file(&test_file).unwrap();
}

/// US1-AS4: Given first command produces no output, When pipeline executes,
/// Then second command receives empty input and completes successfully
#[test]
fn test_empty_output_pipe() {
    // Pipeline: true produces no output, cat receives empty stdin
    let pipeline_input = "true | cat";
    let pipeline = parse_pipeline(pipeline_input).unwrap();
    let executor = PipelineExecutor::new();

    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Pipeline succeeds (cat handles empty input gracefully)
    assert_eq!(exit_code, 0, "Pipeline with empty input should succeed");
}

/// US1-AS5: Given user types `date | cat`, When command executes,
/// Then current date/time is displayed (verifying stdout piping works)
#[test]
fn test_date_pipe_cat() {
    let pipeline_input = "date | cat";
    let pipeline = parse_pipeline(pipeline_input).unwrap();
    let executor = PipelineExecutor::new();

    let exit_code = executor.execute(&pipeline).unwrap();

    // Verify: Command succeeded
    assert_eq!(exit_code, 0, "Pipeline should succeed");

    // Expected output: Current date/time from date command
    // Output validation deferred to implementation phase
}

/// Additional Test: Verify pipeline parsing for basic two-command case
#[test]
fn test_parse_basic_pipeline() {
    let pipeline = parse_pipeline("ls | grep txt").unwrap();

    assert_eq!(pipeline.len(), 2, "Should have 2 segments");
    assert_eq!(pipeline.segments[0].program, "ls");
    assert_eq!(pipeline.segments[0].args.len(), 0);
    assert_eq!(pipeline.segments[1].program, "grep");
    assert_eq!(pipeline.segments[1].args, vec!["txt"]);
}

/// Additional Test: Verify single-command optimization path
#[test]
fn test_single_command_no_pipe() {
    let pipeline = parse_pipeline("echo hello").unwrap();

    assert_eq!(pipeline.len(), 1, "Should have 1 segment");
    assert_eq!(pipeline.segments[0].program, "echo");
    assert_eq!(pipeline.segments[0].args, vec!["hello"]);

    let executor = PipelineExecutor::new();
    let exit_code = executor.execute(&pipeline).unwrap();
    assert_eq!(exit_code, 0);
}
