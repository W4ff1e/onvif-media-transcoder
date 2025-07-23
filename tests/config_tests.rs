use onvif_media_transcoder::{Config, ServiceStatus};
use serial_test::serial;
use std::env;
use std::sync::Arc;

#[test]
#[serial]
fn test_config_from_env_with_valid_variables() {
    // Clean up first
    cleanup_env_vars();

    // Set up environment variables
    env::set_var("RTSP_STREAM_URL", "rtsp://127.0.0.1:8554/test");
    env::set_var("ONVIF_PORT", "8080");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::set_var("CONTAINER_IP", "192.168.1.100");
    env::set_var("WS_DISCOVERY_ENABLED", "true");

    let config = Config::from_env().expect("Should create config from environment");

    assert_eq!(config.rtsp_stream_url, "rtsp://127.0.0.1:8554/test");
    assert_eq!(config.onvif_port, "8080");
    assert_eq!(config.device_name, "Test-Device");
    assert_eq!(config.onvif_username, "testuser");
    assert_eq!(config.onvif_password, "testpass");
    assert_eq!(config.container_ip, "192.168.1.100");
    assert_eq!(config.ws_discovery_enabled, true);

    // Clean up
    cleanup_env_vars();
}

#[test]
#[serial]
fn test_config_from_env_with_disabled_ws_discovery() {
    // Clean up first
    cleanup_env_vars();

    // Set up environment variables
    env::set_var("RTSP_STREAM_URL", "rtsp://127.0.0.1:8554/test");
    env::set_var("ONVIF_PORT", "8080");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::set_var("CONTAINER_IP", "192.168.1.100");
    env::set_var("WS_DISCOVERY_ENABLED", "false");

    let config = Config::from_env().expect("Should create config from environment");

    assert_eq!(config.ws_discovery_enabled, false);

    // Clean up
    cleanup_env_vars();
}

#[test]
#[serial]
fn test_config_from_env_missing_required_variable() {
    // Don't set RTSP_STREAM_URL
    env::set_var("ONVIF_PORT", "8080");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::set_var("CONTAINER_IP", "192.168.1.100");
    env::set_var("WS_DISCOVERY_ENABLED", "true");

    let result = Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("RTSP_STREAM_URL environment variable must be set"));

    // Clean up
    cleanup_env_vars();
}

#[test]
#[serial]
fn test_config_from_env_invalid_port() {
    // Clean up first
    cleanup_env_vars();

    // Set up environment variables with invalid port
    env::set_var("RTSP_STREAM_URL", "rtsp://127.0.0.1:8554/test");
    env::set_var("ONVIF_PORT", "invalid_port");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::set_var("CONTAINER_IP", "192.168.1.100");
    env::set_var("WS_DISCOVERY_ENABLED", "true");

    let result = Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("ONVIF_PORT must be a valid port number"));

    // Clean up
    cleanup_env_vars();
}

#[test]
#[serial]
fn test_config_from_env_invalid_ip() {
    // Clean up first
    cleanup_env_vars();

    // Set up environment variables with invalid IP
    env::set_var("RTSP_STREAM_URL", "rtsp://127.0.0.1:8554/test");
    env::set_var("ONVIF_PORT", "8080");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::set_var("CONTAINER_IP", "invalid.ip.address");
    env::set_var("WS_DISCOVERY_ENABLED", "true");

    let result = Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("is not a valid IP address"));

    // Clean up
    cleanup_env_vars();
}

#[test]
#[serial]
fn test_config_from_env_invalid_rtsp_url() {
    // Clean up first
    cleanup_env_vars();

    // Set up environment variables with invalid RTSP URL
    env::set_var("RTSP_STREAM_URL", "http://127.0.0.1:8554/test"); // Should be rtsp://
    env::set_var("ONVIF_PORT", "8080");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::set_var("CONTAINER_IP", "192.168.1.100");
    env::set_var("WS_DISCOVERY_ENABLED", "true");

    let result = Config::from_env();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("RTSP_STREAM_URL must start with 'rtsp://'"));

    // Clean up
    cleanup_env_vars();
}

#[test]
#[serial]
fn test_config_from_env_with_default_container_ip() {
    // Clean up first
    cleanup_env_vars();

    // Set up environment variables without CONTAINER_IP
    env::set_var("RTSP_STREAM_URL", "rtsp://127.0.0.1:8554/test");
    env::set_var("ONVIF_PORT", "8080");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::remove_var("CONTAINER_IP"); // Remove if exists
    env::set_var("WS_DISCOVERY_ENABLED", "true");

    let config = Config::from_env().expect("Should create config from environment");

    assert_eq!(config.container_ip, "127.0.0.1"); // Should default to localhost

    // Clean up
    cleanup_env_vars();
}

#[test]
#[serial]
fn test_config_display() {
    // Clean up first
    cleanup_env_vars();

    // Set up environment variables
    env::set_var("RTSP_STREAM_URL", "rtsp://127.0.0.1:8554/test");
    env::set_var("ONVIF_PORT", "8080");
    env::set_var("DEVICE_NAME", "Test-Device");
    env::set_var("ONVIF_USERNAME", "testuser");
    env::set_var("ONVIF_PASSWORD", "testpass");
    env::set_var("CONTAINER_IP", "192.168.1.100");
    env::set_var("WS_DISCOVERY_ENABLED", "true");

    let config = Config::from_env().expect("Should create config from environment");

    // This test mainly ensures display() doesn't panic
    config.display();

    // Clean up
    cleanup_env_vars();
}

#[test]
fn test_service_status_creation() {
    let status = ServiceStatus::new();

    assert!(!status
        .ws_discovery_healthy
        .load(std::sync::atomic::Ordering::SeqCst));
    assert!(!status
        .onvif_service_healthy
        .load(std::sync::atomic::Ordering::SeqCst));
    assert!(!status
        .shutdown_requested
        .load(std::sync::atomic::Ordering::SeqCst));

    let last_error = status.last_error.lock().unwrap();
    assert!(last_error.is_none());
}

#[test]
fn test_service_status_shutdown() {
    let status = ServiceStatus::new();

    // Initially should not be shutdown
    assert!(!status.is_shutdown_requested());

    // Request shutdown
    status.request_shutdown();

    // Should now be shutdown
    assert!(status.is_shutdown_requested());
}

#[test]
fn test_service_status_error() {
    let status = ServiceStatus::new();

    // Set an error
    status.set_error("Test error message");

    // Check that error was set
    let last_error = status.last_error.lock().unwrap();
    assert_eq!(last_error.as_ref().unwrap(), "Test error message");
}

#[test]
fn test_service_status_arc_sharing() {
    let status = Arc::new(ServiceStatus::new());
    let status_clone = status.clone();

    // Modify through clone
    status_clone.request_shutdown();

    // Should be visible through original
    assert!(status.is_shutdown_requested());
}

// Helper function to clean up environment variables after tests
fn cleanup_env_vars() {
    let vars_to_remove = [
        "RTSP_STREAM_URL",
        "ONVIF_PORT",
        "DEVICE_NAME",
        "ONVIF_USERNAME",
        "ONVIF_PASSWORD",
        "CONTAINER_IP",
        "WS_DISCOVERY_ENABLED",
    ];

    for var in &vars_to_remove {
        env::remove_var(var);
    }
}
