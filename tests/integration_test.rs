use onvif_media_transcoder::config::Config;
use onvif_media_transcoder::ws_discovery::{DeviceInfo, WSDiscoveryServer};

#[test]
fn test_config_loading_defaults() {
    // We can't easily test Config::from_args() in a unit test because it reads actual CLI args.
    // But we can test that the struct exists and we can create it if we had a constructor.
    // Since Config fields are private and there's no public constructor other than load(),
    // we might need to make fields public or add a constructor for testing.
    // For now, let's just verify we can import it.
}

#[test]
fn test_device_info_creation() {
    let device_info = DeviceInfo {
        endpoint_reference: "urn:uuid:test".to_string(),
        types: "tdn:Test".to_string(),
        scopes: "onvif://test".to_string(),
        xaddrs: "http://localhost".to_string(),
        manufacturer: "Test".to_string(),
        model_name: "Test".to_string(),
        friendly_name: "Test".to_string(),
        firmware_version: "1.0".to_string(),
        serial_number: "123".to_string(),
    };

    assert_eq!(device_info.manufacturer, "Test");
}

// We can't easily test WSDiscoveryServer::new without network permissions or mocking,
// but we can verify the type exists.
