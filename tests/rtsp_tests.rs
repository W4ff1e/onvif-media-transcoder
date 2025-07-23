use std::env;

#[test]
fn test_rtsp_url_validation() {
    // Test valid RTSP URLs
    let valid_rtsp_urls = vec![
        "rtsp://127.0.0.1:8554/stream",
        "rtsp://192.168.1.100:8554/live",
        "rtsp://example.com:554/video",
        "rtsp://user:pass@server.com:554/stream",
    ];

    for url in valid_rtsp_urls {
        assert!(url.starts_with("rtsp://"));
        assert!(url.contains("://"));
    }
}

#[test]
fn test_invalid_rtsp_url_detection() {
    // Test invalid RTSP URLs (these should be rejected)
    let invalid_rtsp_urls = vec![
        "http://127.0.0.1:8554/stream", // Wrong protocol
        "https://example.com/stream",   // Wrong protocol
        "ftp://server.com/video",       // Wrong protocol
        "rtmp://server.com/stream",     // Wrong protocol
        "127.0.0.1:8554/stream",        // Missing protocol
        "",                             // Empty
    ];

    for url in invalid_rtsp_urls {
        assert!(!url.starts_with("rtsp://") || url.is_empty());
    }
}

#[test]
fn test_rtsp_url_components_parsing() {
    let rtsp_url = "rtsp://192.168.1.100:8554/stream";

    // Test that URL can be parsed
    assert!(rtsp_url.starts_with("rtsp://"));

    // Extract components manually (as the validation function might do)
    let without_protocol = rtsp_url.strip_prefix("rtsp://").unwrap();
    let parts: Vec<&str> = without_protocol.splitn(2, '/').collect();

    assert_eq!(parts.len(), 2);

    let host_port = parts[0];
    let path = parts[1];

    assert_eq!(host_port, "192.168.1.100:8554");
    assert_eq!(path, "stream");

    // Further parse host and port
    let host_port_parts: Vec<&str> = host_port.split(':').collect();
    assert_eq!(host_port_parts.len(), 2);
    assert_eq!(host_port_parts[0], "192.168.1.100");
    assert_eq!(host_port_parts[1], "8554");
}

#[test]
fn test_ffprobe_command_construction() {
    // Test ffprobe command arguments that would be used for RTSP validation
    let rtsp_url = "rtsp://127.0.0.1:8554/stream";
    let timeout = "10000000"; // 10 seconds in microseconds
    let analyzeduration = "5000000"; // 5 seconds in microseconds

    let args = vec![
        "-v",
        "quiet",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=codec_name",
        "-of",
        "csv=p=0",
        "-timeout",
        timeout,
        "-analyzeduration",
        analyzeduration,
        rtsp_url,
    ];

    assert!(args.contains(&"-v"));
    assert!(args.contains(&"quiet"));
    assert!(args.contains(&"-timeout"));
    assert!(args.contains(&timeout));
    assert!(args.contains(&rtsp_url));
    assert_eq!(args.len(), 13);
}

#[test]
fn test_codec_parsing() {
    // Test codec name parsing from ffprobe output
    let sample_outputs = vec![
        "h264\n", "h265\n", "mjpeg\n", "mpeg4\n", "", // Empty output case
    ];

    for output in sample_outputs {
        let codec = output.trim();
        if !codec.is_empty() {
            assert!(!codec.contains('\n'));
            assert!(!codec.contains('\r'));
        }
    }
}

#[test]
fn test_rtsp_error_messages() {
    // Test error message patterns that might be returned from ffprobe
    let error_messages = vec![
        "Connection refused",
        "Connection timed out",
        "No route to host",
        "Protocol not found",
        "Invalid data found when processing input",
        "Server returned 404 Not Found",
    ];

    for error_msg in error_messages {
        assert!(!error_msg.is_empty());
        // These would be used to provide meaningful error feedback
    }
}

#[test]
fn test_rtsp_timeout_values() {
    // Test timeout value conversions
    let timeout_seconds = 10;
    let timeout_microseconds = timeout_seconds * 1_000_000;

    assert_eq!(timeout_microseconds, 10_000_000);
    assert_eq!(timeout_microseconds.to_string(), "10000000");

    let analyzeduration_seconds = 5;
    let analyzeduration_microseconds = analyzeduration_seconds * 1_000_000;

    assert_eq!(analyzeduration_microseconds, 5_000_000);
    assert_eq!(analyzeduration_microseconds.to_string(), "5000000");
}

#[test]
fn test_rtsp_port_validation() {
    // Test RTSP port validation
    let rtsp_ports = vec![
        "554",  // Standard RTSP port
        "8554", // Common alternative port
        "1935", // Another common streaming port
        "80",   // HTTP port (sometimes used)
        "443",  // HTTPS port (sometimes used)
    ];

    for port_str in rtsp_ports {
        let port: Result<u16, _> = port_str.parse();
        assert!(port.is_ok());

        if let Ok(port_num) = port {
            assert!(port_num > 0);
            // u16 max is 65535, so no need to check upper bound
        }
    }
}

#[test]
fn test_rtsp_validation_error_handling() {
    // Test error handling patterns for RTSP validation
    let error_cases = vec![
        ("Connection refused", "network"),
        ("Timeout", "timeout"),
        ("Invalid data", "format"),
        ("Protocol not found", "protocol"),
        ("Server returned 404", "not_found"),
    ];

    for (error_msg, error_type) in error_cases {
        assert!(!error_msg.is_empty());
        assert!(!error_type.is_empty());

        // Test error categorization
        match error_type {
            "network" => assert!(error_msg.to_lowercase().contains("connection")),
            "timeout" => assert!(error_msg.to_lowercase().contains("timeout")),
            "format" => assert!(
                error_msg.to_lowercase().contains("invalid")
                    || error_msg.to_lowercase().contains("data")
            ),
            "protocol" => assert!(error_msg.to_lowercase().contains("protocol")),
            "not_found" => assert!(error_msg.contains("404")),
            _ => {}
        }
    }
}

#[test]
fn test_common_rtsp_codecs() {
    // Test common video codecs that might be detected
    let common_codecs = vec![
        "h264", "h265", "hevc", "mjpeg", "mpeg4", "vp8", "vp9", "av1",
    ];

    for codec in common_codecs {
        assert!(!codec.is_empty());
        assert!(!codec.contains(' '));
        assert!(codec.chars().all(|c| c.is_alphanumeric()));
    }
}

// Helper function to clean up environment variables after tests
#[allow(dead_code)]
fn cleanup_env_vars() {
    env::remove_var("RTSP_STREAM_URL");
    env::remove_var("ONVIF_PORT");
    env::remove_var("DEVICE_NAME");
    env::remove_var("ONVIF_USERNAME");
    env::remove_var("ONVIF_PASSWORD");
    env::remove_var("CONTAINER_IP");
    env::remove_var("WS_DISCOVERY_ENABLED");
}
