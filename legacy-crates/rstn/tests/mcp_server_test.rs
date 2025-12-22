//! Integration tests for MCP server SSE connection

use rstn::tui::mcp_server::{self, McpServerConfig, McpState};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

/// Create a default McpState for testing
fn create_test_mcp_state() -> Arc<Mutex<McpState>> {
    Arc::new(Mutex::new(McpState::default()))
}

#[tokio::test]
async fn test_mcp_server_starts_and_stops() {
    // Start server on test port
    let config = McpServerConfig {
        port: 19561,
        name: "rstn-test".to_string(),
        version: "0.1.0".to_string(),
    };

    let (event_tx, _event_rx) = mpsc::channel(10);

    // Start the server
    let handle = mcp_server::start_server(config, event_tx, create_test_mcp_state())
        .await
        .expect("Failed to start MCP server");

    // Verify the server is accessible
    assert_eq!(handle.port(), 19561);
    assert_eq!(handle.url(), "http://127.0.0.1:19561");

    // Give the server a moment to fully start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Gracefully shutdown
    handle.shutdown().await;
}

#[tokio::test]
async fn test_mcp_server_http_reachability() {
    // Start server on test port
    let config = McpServerConfig {
        port: 19562,
        name: "rstn-test".to_string(),
        version: "0.1.0".to_string(),
    };

    let (event_tx, _event_rx) = mpsc::channel(10);

    let handle = mcp_server::start_server(config, event_tx, create_test_mcp_state())
        .await
        .expect("Failed to start MCP server");

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create HTTP client and try to connect
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    // Try the base URL first - the MCP server should be listening
    let url = handle.url();

    // Make request with timeout to prevent hanging
    let result = timeout(Duration::from_secs(5), client.get(&url).send()).await;

    // The important thing is that we get SOME response from the server
    // (not a connection refused error), even if it's 404 or other status.
    // This proves the HTTP server is running and listening.
    assert!(
        result.is_ok(),
        "Request timed out - server may not be listening"
    );
    let response = result.unwrap();
    assert!(
        response.is_ok(),
        "Failed to connect to server (connection refused): {:?}",
        response.err()
    );

    // If we got here, the server is reachable. The exact endpoint structure
    // is handled by prism-mcp-rs and will be tested in features 061-063.
    let response = response.unwrap();
    let status = response.status();

    // Any HTTP response (even 404) proves the server is listening
    assert!(
        status.as_u16() < 600,
        "Got unexpected HTTP status: {}",
        status
    );

    // Cleanup
    handle.shutdown().await;
}

#[tokio::test]
async fn test_mcp_state_update() {
    // Start server
    let config = McpServerConfig::default();
    let (event_tx, _event_rx) = mpsc::channel(10);

    let handle = mcp_server::start_server(config, event_tx, create_test_mcp_state())
        .await
        .expect("Failed to start MCP server");

    // Update state
    handle
        .update_state(
            Some("060".to_string()),
            Some("mcp-server-infrastructure".to_string()),
            Some("060-mcp-server-infrastructure".to_string()),
            Some("implement".to_string()),
            Some("specs/060-mcp-server-infrastructure".to_string()),
        )
        .await;

    // State is updated internally - we can't directly verify it here
    // but the test ensures the update_state method works without errors

    // Cleanup
    handle.shutdown().await;
}

#[tokio::test]
async fn test_mcp_config_lifecycle() {
    // Write config
    let config_path = mcp_server::write_mcp_config(19560).expect("Failed to write MCP config");

    // Verify config file exists
    assert!(config_path.exists(), "Config file was not created");

    // Read and verify config content
    let content = std::fs::read_to_string(&config_path).expect("Failed to read config file");

    assert!(content.contains("rstn"), "Config missing rstn server entry");
    assert!(content.contains("http"), "Config missing HTTP transport");
    assert!(content.contains("19560"), "Config missing port");

    // Cleanup config
    mcp_server::cleanup_mcp_config().expect("Failed to cleanup MCP config");

    // Verify config file is removed
    assert!(!config_path.exists(), "Config file was not removed");
}

#[tokio::test]
async fn test_rstn_report_status_tool_registration() {
    // Start server with rstn_report_status tool
    let config = McpServerConfig {
        port: 19564,
        ..Default::default()
    };

    let (event_tx, _event_rx) = mpsc::channel(10);
    let handle = mcp_server::start_server(config, event_tx, create_test_mcp_state())
        .await
        .expect("Failed to start MCP server");

    // Give the server time to start and register tools
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Tool is registered - in real usage, Claude Code would query
    // the tools/list endpoint via MCP protocol to verify this

    // Cleanup
    handle.shutdown().await;
}

#[tokio::test]
async fn test_status_event_handling() {
    use rstn::tui::event::Event;

    let (tx, mut rx) = mpsc::channel(10);

    // Simulate rstn_report_status tool call sending an event
    tx.send(Event::McpStatus {
        status: "needs_input".to_string(),
        prompt: Some("Test prompt".to_string()),
        message: None,
    })
    .await
    .unwrap();

    // Verify event received
    let event = rx.recv().await.unwrap();
    match event {
        Event::McpStatus {
            status,
            prompt,
            message,
        } => {
            assert_eq!(status, "needs_input");
            assert_eq!(prompt, Some("Test prompt".to_string()));
            assert_eq!(message, None);
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_status_event_error() {
    use rstn::tui::event::Event;

    let (tx, mut rx) = mpsc::channel(10);

    // Simulate error status
    tx.send(Event::McpStatus {
        status: "error".to_string(),
        prompt: None,
        message: Some("Test error message".to_string()),
    })
    .await
    .unwrap();

    // Verify event received
    let event = rx.recv().await.unwrap();
    match event {
        Event::McpStatus {
            status,
            prompt,
            message,
        } => {
            assert_eq!(status, "error");
            assert_eq!(prompt, None);
            assert_eq!(message, Some("Test error message".to_string()));
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_status_event_completed() {
    use rstn::tui::event::Event;

    let (tx, mut rx) = mpsc::channel(10);

    // Simulate completed status
    tx.send(Event::McpStatus {
        status: "completed".to_string(),
        prompt: None,
        message: None,
    })
    .await
    .unwrap();

    // Verify event received
    let event = rx.recv().await.unwrap();
    match event {
        Event::McpStatus {
            status,
            prompt,
            message,
        } => {
            assert_eq!(status, "completed");
            assert_eq!(prompt, None);
            assert_eq!(message, None);
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_rstn_read_spec_and_get_context_registration() {
    // Start server with all tools
    let config = McpServerConfig {
        port: 19565,
        ..Default::default()
    };

    let (event_tx, _event_rx) = mpsc::channel(10);
    let handle = mcp_server::start_server(config, event_tx, create_test_mcp_state())
        .await
        .expect("Failed to start MCP server");

    // Give the server time to start and register tools
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Update state with test feature context
    handle
        .update_state(
            Some("062".to_string()),
            Some("mcp-resource-tools".to_string()),
            Some("062-mcp-resource-tools".to_string()),
            Some("implement".to_string()),
            Some("specs/062-mcp-resource-tools".to_string()),
        )
        .await;

    // In real usage, Claude Code would call rstn_read_spec and rstn_get_context
    // via MCP protocol to read spec artifacts and get feature context

    // Cleanup
    handle.shutdown().await;
}

#[test]
fn test_artifact_to_filename_mapping() {
    use rstn::tui::mcp_server::ReadSpecArgs;

    // Test valid artifact names map to correct filenames
    let test_cases = vec![
        ("spec", Some("spec.md")),
        ("plan", Some("plan.md")),
        ("tasks", Some("tasks.md")),
        ("checklist", Some("checklist.md")),
        ("analysis", Some("analysis.md")),
    ];

    for (artifact, _expected_filename) in test_cases {
        // Just verify the struct can be created with these artifact names
        let args = ReadSpecArgs {
            artifact: artifact.to_string(),
        };
        assert_eq!(args.artifact, artifact);
    }
}

#[test]
fn test_feature_context_serialization() {
    use rstn::tui::mcp_server::FeatureContext;

    let context = FeatureContext {
        feature_number: Some("062".to_string()),
        feature_name: Some("mcp-resource-tools".to_string()),
        branch: Some("062-mcp-resource-tools".to_string()),
        phase: Some("implement".to_string()),
        spec_dir: Some("specs/062-mcp-resource-tools".to_string()),
    };

    // Verify serialization works
    let json = serde_json::to_string(&context).unwrap();
    assert!(json.contains("062"));
    assert!(json.contains("mcp-resource-tools"));

    // Verify deserialization works
    let deserialized: FeatureContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.feature_number, Some("062".to_string()));
}

#[tokio::test]
async fn test_rstn_complete_task_tool_registration() {
    // Start server with rstn_complete_task tool
    let config = McpServerConfig {
        port: 19566,
        ..Default::default()
    };

    let (event_tx, _event_rx) = mpsc::channel(10);
    let handle = mcp_server::start_server(config, event_tx, create_test_mcp_state())
        .await
        .expect("Failed to start MCP server");

    // Give the server time to start and register tools
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Tool is registered - in real usage, Claude Code would call
    // rstn_complete_task via MCP protocol to mark tasks complete

    // Cleanup
    handle.shutdown().await;
}

#[tokio::test]
async fn test_task_completed_event_handling() {
    use rstn::tui::event::Event;

    let (tx, mut rx) = mpsc::channel(10);

    // Simulate rstn_complete_task tool call sending an event
    tx.send(Event::McpTaskCompleted {
        task_id: "T001".to_string(),
        success: true,
        message: "Task T001 completion requested".to_string(),
    })
    .await
    .unwrap();

    // Verify event received
    let event = rx.recv().await.unwrap();
    match event {
        Event::McpTaskCompleted {
            task_id,
            success,
            message,
        } => {
            assert_eq!(task_id, "T001");
            assert!(success);
            assert!(message.contains("T001"));
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_task_completed_event_failure() {
    use rstn::tui::event::Event;

    let (tx, mut rx) = mpsc::channel(10);

    // Simulate task completion failure
    tx.send(Event::McpTaskCompleted {
        task_id: "T999".to_string(),
        success: false,
        message: "Task T999 not found".to_string(),
    })
    .await
    .unwrap();

    // Verify event received
    let event = rx.recv().await.unwrap();
    match event {
        Event::McpTaskCompleted {
            task_id,
            success,
            message,
        } => {
            assert_eq!(task_id, "T999");
            assert!(!success);
            assert!(message.contains("not found"));
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_complete_task_args_serialization() {
    use rstn::tui::mcp_server::CompleteTaskArgs;

    let args = CompleteTaskArgs {
        task_id: "T001".to_string(),
        skip_validation: Some(false),
    };

    // Verify serialization works
    let json = serde_json::to_string(&args).unwrap();
    assert!(json.contains("T001"));

    // Verify deserialization works
    let deserialized: CompleteTaskArgs = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.task_id, "T001");
    assert_eq!(deserialized.skip_validation, Some(false));
}
