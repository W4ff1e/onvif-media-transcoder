use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

mod onvif_endpoints;
mod onvif_responses;
mod ws_discovery;

use onvif_endpoints::UNSUPPORTED_ENDPOINTS;
use onvif_responses::*;
// use ws_discovery::{DeviceInfo, WSDiscoveryServer};

/// Configuration structure for the ONVIF Media Transcoder
#[derive(Debug, Clone, Parser)]
#[command(name = "onvif-media-transcoder")]
#[command(
    about = "ONVIF Media Transcoder - Converts media streams to ONVIF-compatible RTSP streams"
)]
struct Config {
    /// RTSP stream URL to transcode
    #[arg(short = 'r', long, default_value = "rtsp://127.0.0.1:8554/stream")]
    rtsp_stream_url: String,

    /// Port for the ONVIF service
    #[arg(short = 'P', long, default_value = "8080")]
    onvif_port: String,

    /// Device name for ONVIF identification
    #[arg(short = 'n', long, default_value = "ONVIF-Media-Transcoder")]
    device_name: String,

    /// Username for ONVIF authentication
    #[arg(short = 'u', long, default_value = "admin")]
    onvif_username: String,

    /// Password for ONVIF authentication
    #[arg(short = 'p', long, default_value = "onvif-rust")]
    onvif_password: String,

    /// Container IP address for WS-Discovery
    #[arg(long = "container-ip", short = 'i', default_value = "127.0.0.1")]
    container_ip: String,

    /// Enable WS-Discovery service (currently disabled)
    #[arg(long = "ws-discovery-enabled", short = 'w', action = clap::ArgAction::SetTrue)]
    ws_discovery_enabled: bool,
}

impl Config {
    fn from_args() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Parsing command-line arguments...");
        let config = Config::parse();

        // Validate port number
        println!("Validating port number...");
        let _: u16 = config
            .onvif_port
            .parse()
            .map_err(|_| "ONVIF_PORT must be a valid port number")?;
        println!("Port validation successful");

        // Validate container IP is not empty
        if config.container_ip.is_empty() {
            return Err("CONTAINER_IP cannot be empty".into());
        }

        // Basic IP format validation
        if config.container_ip.parse::<std::net::IpAddr>().is_err() {
            return Err(format!(
                "CONTAINER_IP '{}' is not a valid IP address",
                config.container_ip
            )
            .into());
        }

        // Validate RTSP stream URL format
        if !config.rtsp_stream_url.starts_with("rtsp://") {
            return Err(format!(
                "RTSP_STREAM_URL must start with 'rtsp://', got: {}",
                config.rtsp_stream_url
            )
            .into());
        }

        println!("Configuration creation completed successfully");
        Ok(config)
    }

    fn display(&self) {
        println!("Configuration:");

        // Check if default values are being used and log accordingly
        if self.rtsp_stream_url == "rtsp://127.0.0.1:8554/stream" {
            println!(
                "  RTSP Input Stream: {} (using default)",
                self.rtsp_stream_url
            );
        } else {
            println!("  RTSP Input Stream: {}", self.rtsp_stream_url);
        }

        if self.onvif_port == "8080" {
            println!("  ONVIF Port: {} (using default)", self.onvif_port);
        } else {
            println!("  ONVIF Port: {}", self.onvif_port);
        }

        if self.device_name == "ONVIF-Media-Transcoder" {
            println!("  Device Name: {} (using default)", self.device_name);
        } else {
            println!("  Device Name: {}", self.device_name);
        }

        if self.onvif_username == "admin" {
            println!("  ONVIF Username: {} (using default)", self.onvif_username);
        } else {
            println!("  ONVIF Username: {}", self.onvif_username);
        }

        if self.onvif_password == "onvif-rust" {
            println!("  ONVIF Password: [HIDDEN] (using default)");
        } else {
            println!("  ONVIF Password: [HIDDEN]");
        }

        if self.container_ip == "127.0.0.1" {
            println!("  Container IP: {} (using default)", self.container_ip);
        } else {
            println!("  Container IP: {}", self.container_ip);
        }

        println!(
            "  WS-Discovery: {} {}",
            if self.ws_discovery_enabled {
                "ENABLED (COMMENTED OUT)"
            } else {
                "DISABLED"
            },
            "(WS-Discovery functionality is currently commented out for simplicity)"
        );
    }
}

/// Main entry point for the ONVIF Media Transcoder
fn main() {
    println!("Starting ONVIF Media Transcoder (Simplified Version)...");

    // Load configuration from command-line arguments
    println!("Loading configuration from command-line arguments...");
    let config = match Config::from_args() {
        Ok(config) => {
            println!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("Configuration error: {e}");
            std::process::exit(1);
        }
    };

    // Display configuration
    config.display();

    // WS-Discovery is commented out for simplification
    // if config.ws_discovery_enabled {
    //     println!("WS-Discovery is enabled but commented out for simplification");
    // } else {
    println!("WS-Discovery disabled - continuing with direct ONVIF connections only");
    // }

    // Start ONVIF web service (this will block)
    println!("Starting ONVIF web service...");

    if let Err(e) = start_onvif_service(&config) {
        eprintln!("ONVIF service error: {e}");
        std::process::exit(1);
    }
}

fn start_onvif_service(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ONVIF web service on port {}", config.onvif_port);
    println!("Exposing RTSP stream: {}", config.rtsp_stream_url);
    println!("Device Name: {}", config.device_name);
    println!("Authentication: {} / [HIDDEN]", config.onvif_username);

    let bind_addr = format!("0.0.0.0:{}", config.onvif_port);
    println!("Attempting to bind to address: {bind_addr}");

    let listener = match TcpListener::bind(&bind_addr) {
        Ok(listener) => {
            println!("Successfully bound to {bind_addr}");
            listener
        }
        Err(e) => {
            let error_msg = format!("Failed to bind to ONVIF port {}: {}", config.onvif_port, e);
            eprintln!("{error_msg}");
            return Err(error_msg.into());
        }
    };

    println!("ONVIF Camera service running on port {}", config.onvif_port);
    println!("Stream URI: {}", config.rtsp_stream_url);

    let mut connection_count = 0u64;

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                connection_count += 1;
                println!(
                    "Accepted connection #{} from: {:?}",
                    connection_count,
                    stream.peer_addr()
                );

                // Handle request directly in main thread (simplified)
                if let Err(e) = handle_onvif_request(stream, config) {
                    eprintln!("Error handling connection #{}: {}", connection_count, e);
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
                continue;
            }
        }

        // Periodic status update
        if connection_count % 10 == 0 {
            println!("ONVIF service is healthy - processed {connection_count} connections");
        }
    }

    println!("ONVIF service listener loop ended");
    Ok(())
}

fn handle_onvif_request(
    mut stream: TcpStream,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set socket timeouts
    let timeout = std::time::Duration::from_secs(30);
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    // Get client info for debugging
    let client_addr = stream
        .peer_addr()
        .unwrap_or_else(|_| "unknown:0".parse().unwrap());

    println!("New connection from: {client_addr}");
    let mut buffer = [0; 4096];

    let size = stream
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read from stream: {e}"))?;

    if size == 0 {
        println!("  Connection closed by client (0 bytes read)");
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..size]);
    let first_line = request.lines().next().unwrap_or("Unknown");
    println!("Received ONVIF request: {first_line}");

    // Check for authentication
    let requires_auth = !is_public_endpoint(&request);

    if requires_auth && !is_authenticated(&request, &config.onvif_username, &config.onvif_password)
    {
        println!("  Authentication required - sending 401 response");
        send_auth_required_response(&mut stream)?;
        return Ok(());
    }

    // Handle ONVIF endpoints
    if request.contains("GetCapabilities") {
        println!("Handling supported endpoint: GetCapabilities");
        send_capabilities_response(&mut stream, &config.container_ip, &config.onvif_port)?;
    } else if request.contains("GetServices") {
        println!("Handling supported endpoint: GetServices");
        send_services_response(&mut stream, &config.container_ip, &config.onvif_port)?;
    } else if request.contains("GetSystemDateAndTime") {
        println!("Handling supported endpoint: GetSystemDateAndTime");
        send_system_date_time_response(&mut stream)?;
    } else if request.contains("GetProfiles") {
        println!("Handling supported endpoint: GetProfiles");
        send_profiles_response(&mut stream, &config.rtsp_stream_url)?;
    } else if request.contains("GetStreamUri") {
        println!("Handling supported endpoint: GetStreamUri");
        send_stream_uri_response(&mut stream, &config.rtsp_stream_url)?;
    } else if request.contains("GetSnapshotUri") {
        println!("Handling supported endpoint: GetSnapshotUri");
        send_snapshot_uri_response(&mut stream, &config.container_ip, &config.onvif_port)?;
    } else if request.contains("GetDeviceInformation") {
        println!("Handling supported endpoint: GetDeviceInformation");
        send_device_info_response(&mut stream, &config.device_name)?;
    } else if request.contains("GetVideoSources") {
        println!("Handling supported endpoint: GetVideoSources");
        send_video_sources_response(&mut stream)?;
    } else if request.contains("GetVideoSourceConfigurations") {
        println!("Handling supported endpoint: GetVideoSourceConfigurations");
        send_video_source_configurations_response(&mut stream)?;
    } else if request.contains("GetVideoEncoderConfigurations") {
        println!("Handling supported endpoint: GetVideoEncoderConfigurations");
        send_video_encoder_configurations_response(&mut stream)?;
    } else if request.contains("GetAudioSourceConfigurations") {
        println!("Handling supported endpoint: GetAudioSourceConfigurations");
        send_audio_source_configurations_response(&mut stream)?;
    } else if request.contains("GetAudioEncoderConfigurations") {
        println!("Handling supported endpoint: GetAudioEncoderConfigurations");
        send_audio_encoder_configurations_response(&mut stream)?;
    } else if request.contains("GetServiceCapabilities") {
        println!("Handling supported endpoint: GetServiceCapabilities");
        send_service_capabilities_response(&mut stream)?;
    } else {
        // Detect and log unsupported ONVIF endpoints
        let unsupported_endpoint = detect_unsupported_onvif_endpoint(&request);
        if let Some(endpoint) = unsupported_endpoint {
            eprintln!("UNSUPPORTED ONVIF ENDPOINT: {endpoint}");
            send_unsupported_endpoint_response(&mut stream, &endpoint)?;
        } else {
            println!("Unknown request type: {first_line}");
            send_default_response(&mut stream)?;
        }
    }

    Ok(())
}

fn send_http_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|e| format!("Failed to send HTTP response: {e}").into())
}

fn send_soap_response(
    stream: &mut TcpStream,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    send_http_response(stream, "200 OK", "application/soap+xml", body)
}

fn is_authenticated(request: &str, username: &str, password: &str) -> bool {
    // Check for Basic Auth first
    if let Some(auth_header) = extract_authorization_header(request) {
        if auth_header.starts_with("Basic ") {
            return validate_basic_auth(&auth_header, username, password);
        }
    }

    // Check for WS-Security Username Token
    if request.contains("<UsernameToken>") && request.contains("<Username>") {
        return validate_ws_security_auth(request, username, password);
    }

    false
}

fn is_public_endpoint(request: &str) -> bool {
    // Allow certain endpoints without authentication for ONVIF discovery
    request.contains("GetCapabilities")
        || request.contains("GetDeviceInformation")
        || request.contains("GetServices")
        || request.contains("GetSystemDateAndTime")
        || request.contains("GetServiceCapabilities")
        || request.contains("<GetCapabilities")
        || request.contains("<GetDeviceInformation")
        || request.contains("<GetServices")
        || request.contains("<GetSystemDateAndTime")
        || request.contains("<GetServiceCapabilities")
        || request.contains(":GetCapabilities")
        || request.contains(":GetDeviceInformation")
        || request.contains(":GetServices")
        || request.contains(":GetSystemDateAndTime")
        || request.contains(":GetServiceCapabilities")
}

fn extract_authorization_header(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("authorization:") {
            if let Some(auth_value) = line.split(':').nth(1) {
                return Some(auth_value.trim().to_string());
            }
        }
    }
    None
}

fn validate_basic_auth(auth_header: &str, username: &str, password: &str) -> bool {
    if let Some(encoded) = auth_header.strip_prefix("Basic ") {
        if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(encoded.trim()) {
            if let Ok(decoded) = String::from_utf8(decoded_bytes) {
                let expected = format!("{username}:{password}");
                return decoded == expected;
            }
        }
    }
    false
}

fn validate_ws_security_auth(request: &str, username: &str, password: &str) -> bool {
    // Parse WS-Security UsernameToken
    if let (Some(user_start), Some(user_end)) =
        (request.find("<Username>"), request.find("</Username>"))
    {
        let provided_username = &request[user_start + 10..user_end];
        if provided_username != username {
            return false;
        }
    } else {
        return false;
    }

    // Look for password element
    if let Some(password_start) = request.find("<Password") {
        if let Some(tag_end) = request[password_start..].find('>') {
            let tag_content = &request[password_start..password_start + tag_end + 1];

            if let Some(pwd_end) = request[password_start + tag_end + 1..].find("</Password>") {
                let password_value =
                    &request[password_start + tag_end + 1..password_start + tag_end + 1 + pwd_end];

                if tag_content.contains("PasswordDigest") {
                    // For PasswordDigest, we'd need to extract nonce and created timestamp
                    // For simplicity, we'll just check if it's plain text password
                    return password_value == password;
                } else {
                    // Plain text password
                    return password_value == password;
                }
            }
        }
    }

    false
}

fn send_auth_required_response(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let auth_response = get_auth_required_response();
    stream
        .write_all(auth_response.as_bytes())
        .map_err(|e| format!("Failed to send auth required response: {e}").into())
}

fn send_capabilities_response(
    stream: &mut TcpStream,
    container_ip: &str,
    onvif_port: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_capabilities_response(container_ip, onvif_port);
    send_soap_response(stream, &body)
}

fn send_services_response(
    stream: &mut TcpStream,
    container_ip: &str,
    onvif_port: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_services_response(container_ip, onvif_port);
    send_soap_response(stream, &body)
}

fn send_system_date_time_response(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_system_date_time_response();
    send_soap_response(stream, &body)
}

fn send_profiles_response(
    stream: &mut TcpStream,
    _rtsp_stream: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_profiles_response();
    send_soap_response(stream, &body)
}

fn send_stream_uri_response(
    stream: &mut TcpStream,
    rtsp_stream: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_stream_uri_response(rtsp_stream);
    send_soap_response(stream, &body)
}

fn send_snapshot_uri_response(
    stream: &mut TcpStream,
    container_ip: &str,
    onvif_port: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_snapshot_uri_response(container_ip, onvif_port);
    send_soap_response(stream, &body)
}

fn send_device_info_response(
    stream: &mut TcpStream,
    device_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_device_info_response(device_name);
    send_soap_response(stream, &body)
}

fn send_default_response(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_default_response();
    send_http_response(stream, "200 OK", "text/plain", &body)
}

fn send_video_sources_response(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_video_sources_response();
    send_soap_response(stream, &body)
}

fn send_service_capabilities_response(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_service_capabilities_response();
    send_soap_response(stream, &body)
}

fn send_video_source_configurations_response(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_video_source_configurations_response();
    send_soap_response(stream, &body)
}

fn send_video_encoder_configurations_response(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_video_encoder_configurations_response();
    send_soap_response(stream, &body)
}

fn send_audio_source_configurations_response(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_audio_source_configurations_response();
    send_soap_response(stream, &body)
}

fn send_audio_encoder_configurations_response(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_audio_encoder_configurations_response();
    send_soap_response(stream, &body)
}

fn detect_unsupported_onvif_endpoint(request: &str) -> Option<String> {
    // Check if the request contains any unsupported endpoints
    for endpoint in UNSUPPORTED_ENDPOINTS {
        if request.contains(endpoint) {
            return Some(endpoint.to_string());
        }
    }

    None
}

fn send_unsupported_endpoint_response(
    stream: &mut TcpStream,
    endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = get_unsupported_endpoint_response(endpoint);
    send_soap_response(stream, &body)
}
