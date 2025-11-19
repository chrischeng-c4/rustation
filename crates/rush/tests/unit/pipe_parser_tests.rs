// Parser unit tests for pipe operator support

use rush::executor::parser::parse_pipeline;

#[test]
fn test_parse_single_command() {
    let pipeline = parse_pipeline("ls").unwrap();
    assert_eq!(pipeline.len(), 1);
    assert_eq!(pipeline.segments[0].program, "ls");
    assert_eq!(pipeline.segments[0].args.len(), 0);
}

#[test]
fn test_parse_two_command_pipeline() {
    let pipeline = parse_pipeline("ls | grep txt").unwrap();
    assert_eq!(pipeline.len(), 2);
    assert_eq!(pipeline.segments[0].program, "ls");
    assert_eq!(pipeline.segments[0].args.len(), 0);
    assert_eq!(pipeline.segments[1].program, "grep");
    assert_eq!(pipeline.segments[1].args, vec!["txt"]);
}

#[test]
fn test_parse_multi_command_pipeline() {
    let pipeline = parse_pipeline("cat file.txt | grep error | wc -l").unwrap();
    assert_eq!(pipeline.len(), 3);
    assert_eq!(pipeline.segments[0].program, "cat");
    assert_eq!(pipeline.segments[0].args, vec!["file.txt"]);
    assert_eq!(pipeline.segments[1].program, "grep");
    assert_eq!(pipeline.segments[1].args, vec!["error"]);
    assert_eq!(pipeline.segments[2].program, "wc");
    assert_eq!(pipeline.segments[2].args, vec!["-l"]);
}

#[test]
fn test_parse_pipe_in_quotes() {
    // Pipe inside quotes should be treated as literal
    let pipeline = parse_pipeline("echo \"a | b\"").unwrap();
    assert_eq!(pipeline.len(), 1);
    assert_eq!(pipeline.segments[0].program, "echo");
    assert_eq!(pipeline.segments[0].args, vec!["a | b"]);
}

#[test]
fn test_parse_empty_before_pipe() {
    // Should error: no command before pipe
    let result = parse_pipeline("| grep foo");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Empty command before pipe"));
}

#[test]
fn test_parse_empty_after_pipe() {
    // Should error: no command after pipe
    let result = parse_pipeline("ls |");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Empty command after pipe"));
}

#[test]
fn test_parse_double_pipe() {
    // Should error: double pipe creates empty command between
    let result = parse_pipeline("ls | | grep");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Empty command"));
}

#[test]
fn test_parse_with_args_before_pipe() {
    let pipeline = parse_pipeline("ls -la | grep txt").unwrap();
    assert_eq!(pipeline.len(), 2);
    assert_eq!(pipeline.segments[0].program, "ls");
    assert_eq!(pipeline.segments[0].args, vec!["-la"]);
    assert_eq!(pipeline.segments[1].program, "grep");
    assert_eq!(pipeline.segments[1].args, vec!["txt"]);
}

#[test]
fn test_parse_with_multiple_args() {
    let pipeline = parse_pipeline("git commit -m \"test\" | grep success").unwrap();
    assert_eq!(pipeline.len(), 2);
    assert_eq!(pipeline.segments[0].program, "git");
    assert_eq!(pipeline.segments[0].args, vec!["commit", "-m", "test"]);
    assert_eq!(pipeline.segments[1].program, "grep");
    assert_eq!(pipeline.segments[1].args, vec!["success"]);
}

#[test]
fn test_parse_five_command_pipeline() {
    let pipeline = parse_pipeline("a | b | c | d | e").unwrap();
    assert_eq!(pipeline.len(), 5);
    for (i, segment) in pipeline.segments.iter().enumerate() {
        assert_eq!(segment.index, i);
    }
}

#[test]
fn test_segment_is_first() {
    let pipeline = parse_pipeline("a | b | c").unwrap();
    assert!(pipeline.segments[0].is_first());
    assert!(!pipeline.segments[1].is_first());
    assert!(!pipeline.segments[2].is_first());
}

#[test]
fn test_segment_is_last() {
    let pipeline = parse_pipeline("a | b | c").unwrap();
    assert!(!pipeline.segments[0].is_last(pipeline.len()));
    assert!(!pipeline.segments[1].is_last(pipeline.len()));
    assert!(pipeline.segments[2].is_last(pipeline.len()));
}
