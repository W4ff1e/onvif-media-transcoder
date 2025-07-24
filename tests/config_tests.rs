use clap::Parser;
use onvif_media_transcoder::{Config, ServiceStatus};
use std::sync::Arc;

#[test]
fn test_config_with_defaults() {
    // Test with default values by parsing empty args (just program name)
    let config = Config::try_parse_from(&["test-program"]).unwrap();

    assert_eq!(config.rtsp_stream_url, "rtsp://127.0.0.1:8554/stream");
    assert_eq!(config.onvif_port, "8080");
    assert_eq!(config.device_name, "ONVIF-Media-Transcoder");
    assert_eq!(config.onvif_username, "admin");
    assert_eq!(config.onvif_password, "onvif-rust");
    assert_eq!(config.container_ip, "127.0.0.1");
    assert_eq!(config.ws_discovery_enabled, false);
}

#[test]
fn test_config_with_custom_values() {
    // Test with custom values
    let config = Config::try_parse_from(&[
        "test-program",
        "--rtsp-stream-url",
        "rtsp://192.168.1.100:554/stream",
        "--onvif-port",
        "9090",
        "--device-name",
        "Custom-Camera",
        "--onvif-username",
        "testuser",
        "--onvif-password",
        "testpass",
        "--container-ip",
        "192.168.1.50",
        "--ws-discovery-enabled",
    ])
    .unwrap();

    assert_eq!(config.rtsp_stream_url, "rtsp://192.168.1.100:554/stream");
    assert_eq!(config.onvif_port, "9090");
    assert_eq!(config.device_name, "Custom-Camera");
    assert_eq!(config.onvif_username, "testuser");
    assert_eq!(config.onvif_password, "testpass");
    assert_eq!(config.container_ip, "192.168.1.50");
    assert_eq!(config.ws_discovery_enabled, true);
}

#[test]
fn test_config_validation_invalid_port() {
    // Test with invalid port
    let result = Config::try_parse_from(&["test-program", "--onvif-port", "99999"]);

    // Should parse successfully (clap doesn't validate the port format)
    assert!(result.is_ok());

    // But validation should fail when we try to validate
    let config = result.unwrap();
    let validation_result: Result<u16, _> = config.onvif_port.parse();
    assert!(validation_result.is_err());
}

#[test]
fn test_config_validation_invalid_ip() {
    // Test with invalid IP
    let config =
        Config::try_parse_from(&["test-program", "--container-ip", "invalid.ip.address"]).unwrap();

    // IP validation should fail
    let validation_result = config.container_ip.parse::<std::net::IpAddr>();
    assert!(validation_result.is_err());
}

#[test]
fn test_config_validation_invalid_rtsp_url() {
    // Test with invalid RTSP URL (should start with rtsp://)
    let config = Config::try_parse_from(&[
        "test-program",
        "--rtsp-stream-url",
        "http://127.0.0.1:8554/test",
    ])
    .unwrap();

    // URL validation should fail (doesn't start with rtsp://)
    assert!(!config.rtsp_stream_url.starts_with("rtsp://"));
}

#[test]
fn test_config_with_partial_custom_values() {
    // Test with only some custom values, others should be defaults
    let config = Config::try_parse_from(&[
        "test-program",
        "--onvif-port",
        "9090",
        "--device-name",
        "Custom-Camera",
        "--ws-discovery-enabled",
    ])
    .unwrap();

    // Custom values
    assert_eq!(config.onvif_port, "9090");
    assert_eq!(config.device_name, "Custom-Camera");
    assert_eq!(config.ws_discovery_enabled, true);

    // Default values
    assert_eq!(config.rtsp_stream_url, "rtsp://127.0.0.1:8554/stream");
    assert_eq!(config.onvif_username, "admin");
    assert_eq!(config.onvif_password, "onvif-rust");
    assert_eq!(config.container_ip, "127.0.0.1");
}

#[test]
fn test_config_display() {
    // Test that display() doesn't panic
    let config = Config::try_parse_from(&["test-program", "--device-name", "Test-Device"]).unwrap();

    // This test mainly ensures display() doesn't panic
    config.display();
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
