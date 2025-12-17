//! Integration tests for MCP server SSE connection

use rstn::tui::mcp_server::{self, McpServerConfig};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

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
    let handle = mcp_server::start_server(config, event_tx)
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

    let handle = mcp_server::start_server(config, event_tx)
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
    let result = timeout(
        Duration::from_secs(5),
        client.get(&url).send()
    ).await;

    // The important thing is that we get SOME response from the server
    // (not a connection refused error), even if it's 404 or other status.
    // This proves the HTTP server is running and listening.
    assert!(result.is_ok(), "Request timed out - server may not be listening");
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

    let handle = mcp_server::start_server(config, event_tx)
        .await
        .expect("Failed to start MCP server");

    // Update state
    handle.update_state(
        Some("060".to_string()),
        Some("mcp-server-infrastructure".to_string()),
        Some("060-mcp-server-infrastructure".to_string()),
        Some("implement".to_string()),
        Some("specs/060-mcp-server-infrastructure".to_string()),
    ).await;

    // State is updated internally - we can't directly verify it here
    // but the test ensures the update_state method works without errors

    // Cleanup
    handle.shutdown().await;
}

#[tokio::test]
async fn test_mcp_config_lifecycle() {
    // Write config
    let config_path = mcp_server::write_mcp_config(19560)
        .expect("Failed to write MCP config");

    // Verify config file exists
    assert!(config_path.exists(), "Config file was not created");

    // Read and verify config content
    let content = std::fs::read_to_string(&config_path)
        .expect("Failed to read config file");

    assert!(content.contains("rstn"), "Config missing rstn server entry");
    assert!(content.contains("sse"), "Config missing SSE transport");
    assert!(content.contains("19560"), "Config missing port");

    // Cleanup config
    mcp_server::cleanup_mcp_config()
        .expect("Failed to cleanup MCP config");

    // Verify config file is removed
    assert!(!config_path.exists(), "Config file was not removed");
}
