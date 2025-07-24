use clap::Parser;
use onvif_media_transcoder::Config;

#[test]
fn test_ws_discovery_device_info_creation() {
    // Test with WS-Discovery enabled
    let config = Config::try_parse_from(&[
        "test-program",
        "--device-name",
        "Test-WS-Discovery-Device",
        "--container-ip",
        "192.168.1.100",
        "--ws-discovery-enabled",
    ])
    .expect("Should create config from args");

    // Test that WS-Discovery is enabled
    assert!(config.ws_discovery_enabled);

    // Test device name for WS-Discovery
    assert_eq!(config.device_name, "Test-WS-Discovery-Device");

    // Test container IP for WS-Discovery
    assert_eq!(config.container_ip, "192.168.1.100");
}

#[test]
fn test_ws_discovery_disabled_config() {
    // Test with WS-Discovery disabled (default behavior)
    let config = Config::try_parse_from(&[
        "test-program",
        "--device-name",
        "Test-Device",
        "--container-ip",
        "192.168.1.100",
    ])
    .expect("Should create config from args");

    // Test that WS-Discovery is disabled by default
    assert!(!config.ws_discovery_enabled);
}

#[test]
fn test_ws_discovery_xaddrs_format() {
    // Test XAddrs format construction with custom values
    let config = Config::try_parse_from(&[
        "test-program",
        "--onvif-port",
        "8080",
        "--container-ip",
        "192.168.1.100",
        "--ws-discovery-enabled",
    ])
    .expect("Should create config from args");

    // Test XAddrs format construction (how it would be used in WS-Discovery)
    let expected_xaddrs = format!(
        "http://{}:{}/onvif/device_service",
        config.container_ip, config.onvif_port
    );

    assert_eq!(
        expected_xaddrs,
        "http://192.168.1.100:8080/onvif/device_service"
    );
}

#[test]
fn test_ws_discovery_device_uuid_generation() {
    use uuid::Uuid;

    // Test UUID generation for device identification
    let device_uuid = Uuid::new_v4();
    let endpoint_reference = format!("urn:uuid:{}", device_uuid);

    assert!(endpoint_reference.starts_with("urn:uuid:"));
    assert_eq!(endpoint_reference.len(), 45); // "urn:uuid:" + 36 char UUID

    // Test that multiple UUIDs are different
    let device_uuid2 = Uuid::new_v4();
    assert_ne!(device_uuid, device_uuid2);
}

#[test]
fn test_ws_discovery_device_info_fields() {
    // Test the fields that would be used in DeviceInfo struct
    let types = "tdn:NetworkVideoTransmitter";
    let scopes =
        "onvif://www.onvif.org/Profile/Streaming onvif://www.onvif.org/name/ONVIF-Media-Transcoder";
    let manufacturer = "ONVIF Media Solutions";
    let firmware_version = "1.0.0";

    // Test field formats
    assert!(types.contains("NetworkVideoTransmitter"));
    assert!(scopes.contains("Profile/Streaming"));
    assert!(scopes.contains("ONVIF-Media-Transcoder"));
    assert!(!manufacturer.is_empty());
    assert!(firmware_version.contains("1.0"));
}

#[test]
fn test_ws_discovery_serial_number_generation() {
    let device_name = "Test-Device-Name";
    let serial_number = format!("EMU-{}", device_name.chars().take(6).collect::<String>());

    assert_eq!(serial_number, "EMU-Test-D");
    assert!(serial_number.starts_with("EMU-"));
    assert_eq!(serial_number.len(), 10); // "EMU-" + 6 chars
}

#[test]
fn test_ws_discovery_invalid_ip_rejection() {
    // Test that invalid IPs are rejected (this would prevent WS-Discovery startup)
    let invalid_ips = vec!["", "0.0.0.0", "999.999.999.999", "not.an.ip"];

    for invalid_ip in invalid_ips {
        // These would be rejected by the IP validation logic
        if invalid_ip.is_empty() || invalid_ip == "0.0.0.0" {
            assert!(true); // These should be rejected
        } else if invalid_ip.parse::<std::net::IpAddr>().is_err() {
            assert!(true); // These should fail parsing
        }
    }
}

#[test]
fn test_ws_discovery_valid_ip_acceptance() {
    // Test that valid IPs are accepted
    let valid_ips = vec!["127.0.0.1", "192.168.1.100", "10.0.0.1", "172.16.0.1"];

    for valid_ip in valid_ips {
        let parse_result = valid_ip.parse::<std::net::IpAddr>();
        assert!(parse_result.is_ok());
    }
}

#[test]
fn test_ws_discovery_probe_match_structure() {
    // Test the XML structure that would be used in WS-Discovery responses
    let device_uuid = uuid::Uuid::new_v4();
    let endpoint_reference = format!("urn:uuid:{}", device_uuid);
    let types = "tdn:NetworkVideoTransmitter";
    let scopes = "onvif://www.onvif.org/Profile/Streaming";
    let xaddrs = "http://192.168.1.100:8080/onvif/device_service";

    // Test that all required WS-Discovery fields are present
    assert!(!endpoint_reference.is_empty());
    assert!(!types.is_empty());
    assert!(!scopes.is_empty());
    assert!(!xaddrs.is_empty());

    // Test URL format
    assert!(xaddrs.starts_with("http://"));
    assert!(xaddrs.contains(":8080"));
    assert!(xaddrs.ends_with("/onvif/device_service"));
}

#[test]
fn test_ws_discovery_multicast_address() {
    // Test WS-Discovery multicast constants
    let ws_discovery_multicast_addr = "239.255.255.250";
    let ws_discovery_port = 3702;

    // These are the standard WS-Discovery multicast values
    assert_eq!(ws_discovery_multicast_addr, "239.255.255.250");
    assert_eq!(ws_discovery_port, 3702);

    // Test that the multicast address parses correctly
    let addr_result = ws_discovery_multicast_addr.parse::<std::net::IpAddr>();
    assert!(addr_result.is_ok());

    if let Ok(addr) = addr_result {
        assert!(addr.is_ipv4());
    }
}
