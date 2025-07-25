use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use sha1::Digest;
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

    /// Enable debug mode with verbose request logging (NOT FOR PRODUCTION USE, LOGS SENSITIVE INFORMATION)
    #[arg(short = 'd', long = "debug", action = clap::ArgAction::SetTrue)]
    debug: bool,
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
            "  WS-Discovery: {} (WS-Discovery functionality is currently commented out for simplicity)",
            if self.ws_discovery_enabled {
                "ENABLED (COMMENTED OUT)"
            } else {
                "DISABLED"
            }
        );

        if self.debug {
            println!("  Debug Mode: ENABLED (verbose request logging)");
        } else {
            println!("  Debug Mode: DISABLED");
        }
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
                    eprintln!("Error handling connection #{connection_count}: {e}");
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
    println!("  Authentication required: {requires_auth}");

    if requires_auth && !is_authenticated(&request, &config.onvif_username, &config.onvif_password)
    {
        println!("  Authentication failed - sending 401 response");

        // Debug dump for authentication failures
        dump_headers(&request, size, "AUTH_FAILED", config.debug);

        send_auth_required_response(&mut stream)?;
        return Ok(());
    } else if requires_auth {
        println!("  Authentication successful");
    } else {
        println!("  Public endpoint - no authentication required");
    }

    // Handle ONVIF endpoints
    if request.contains("GetCapabilities") {
        println!("Handling supported endpoint: GetCapabilities");
        dump_headers(&request, size, "GetCapabilities", config.debug);
        send_capabilities_response(&mut stream, &config.container_ip, &config.onvif_port)?;
    } else if request.contains("GetServices") {
        println!("Handling supported endpoint: GetServices");
        dump_headers(&request, size, "GetServices", config.debug);
        send_services_response(&mut stream, &config.container_ip, &config.onvif_port)?;
    } else if request.contains("GetSystemDateAndTime") {
        println!("Handling supported endpoint: GetSystemDateAndTime");
        dump_headers(&request, size, "GetSystemDateAndTime", config.debug);
        send_system_date_time_response(&mut stream)?;
    } else if request.contains("GetProfiles") {
        println!("Handling supported endpoint: GetProfiles");
        dump_headers(&request, size, "GetProfiles", config.debug);
        send_profiles_response(&mut stream, &config.rtsp_stream_url)?;
    } else if request.contains("GetStreamUri") {
        println!("Handling supported endpoint: GetStreamUri");
        dump_headers(&request, size, "GetStreamUri", config.debug);
        send_stream_uri_response(&mut stream, &config.rtsp_stream_url)?;
    } else if request.contains("GetSnapshotUri") {
        println!("Handling supported endpoint: GetSnapshotUri");
        dump_headers(&request, size, "GetSnapshotUri", config.debug);
        send_snapshot_uri_response(&mut stream, &config.container_ip, &config.onvif_port)?;
    } else if request.contains("GetDeviceInformation") {
        println!("Handling supported endpoint: GetDeviceInformation");
        dump_headers(&request, size, "GetDeviceInformation", config.debug);
        send_device_info_response(&mut stream, &config.device_name)?;
    } else if request.contains("GetVideoSources") {
        println!("Handling supported endpoint: GetVideoSources");
        dump_headers(&request, size, "GetVideoSources", config.debug);
        send_video_sources_response(&mut stream)?;
    } else if request.contains("GetVideoSourceConfigurations") {
        println!("Handling supported endpoint: GetVideoSourceConfigurations");
        dump_headers(&request, size, "GetVideoSourceConfigurations", config.debug);
        send_video_source_configurations_response(&mut stream)?;
    } else if request.contains("GetVideoEncoderConfigurations") {
        println!("Handling supported endpoint: GetVideoEncoderConfigurations");
        dump_headers(
            &request,
            size,
            "GetVideoEncoderConfigurations",
            config.debug,
        );
        send_video_encoder_configurations_response(&mut stream)?;
    } else if request.contains("GetAudioSourceConfigurations") {
        println!("Handling supported endpoint: GetAudioSourceConfigurations");
        dump_headers(&request, size, "GetAudioSourceConfigurations", config.debug);
        send_audio_source_configurations_response(&mut stream)?;
    } else if request.contains("GetAudioEncoderConfigurations") {
        println!("Handling supported endpoint: GetAudioEncoderConfigurations");
        dump_headers(
            &request,
            size,
            "GetAudioEncoderConfigurations",
            config.debug,
        );
        send_audio_encoder_configurations_response(&mut stream)?;
    } else if request.contains("GetServiceCapabilities") {
        println!("Handling supported endpoint: GetServiceCapabilities");
        dump_headers(&request, size, "GetServiceCapabilities", config.debug);
        send_service_capabilities_response(&mut stream)?;
    } else if request.contains("GET /snapshot.jpg") {
        println!("Handling snapshot request: GET /snapshot.jpg");
        dump_headers(&request, size, "snapshot.jpg", config.debug);
        send_snapshot_image_response(&mut stream, &config.rtsp_stream_url)?;
    } else {
        // Detect and log unsupported ONVIF endpoints
        let unsupported_endpoint = detect_unsupported_onvif_endpoint(&request);
        if let Some(endpoint) = unsupported_endpoint {
            eprintln!("UNSUPPORTED ONVIF ENDPOINT: {endpoint}");
            dump_headers(
                &request,
                size,
                &format!("UNSUPPORTED_{endpoint}"),
                config.debug,
            );
            send_unsupported_endpoint_response(&mut stream, &endpoint)?;
        } else {
            println!("Unknown request type: {first_line}");
            dump_headers(&request, size, "UNKNOWN", config.debug);
            send_default_response(&mut stream)?;
        }
    }

    Ok(())
}

/// Debug function to dump request headers and content for troubleshooting
fn dump_headers(request: &str, size: usize, endpoint_name: &str, debug_enabled: bool) {
    if !debug_enabled {
        return;
    }

    println!(
        "=== DEBUG REQUEST DUMP FOR {} ===",
        endpoint_name.to_uppercase()
    );
    println!("Request size: {size} bytes");
    println!("Raw request:");
    println!("{}", "=".repeat(50));
    println!("{request}");
    println!("{}", "=".repeat(50));

    // Parse and display headers separately for easier reading
    println!("Parsed headers:");
    for (i, line) in request.lines().enumerate() {
        if line.is_empty() {
            println!("  [{}]: <EMPTY LINE - Headers end here>", i + 1);
            break;
        }
        println!("  [{}]: {}", i + 1, line);
    }
    println!(
        "=== END DEBUG REQUEST DUMP FOR {} ===",
        endpoint_name.to_uppercase()
    );
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
    println!("  Starting authentication validation...");

    // Check for Basic Auth first (simpler)
    if let Some(auth_header) = extract_authorization_header(request) {
        if auth_header.starts_with("Basic ") {
            println!("  Attempting Basic Auth validation...");
            return validate_basic_auth(&auth_header, username, password);
        } else if auth_header.starts_with("Digest ") {
            println!("  Attempting Digest Auth validation...");
            return validate_digest_auth(&auth_header, request, username, password);
        }
    }

    // Check for WS-Security Username Token (Digest)
    if request.contains("<UsernameToken>") && request.contains("<Username>") {
        println!("  Found WS-Security UsernameToken, attempting validation...");
        return validate_ws_security_auth(request, username, password);
    }

    println!("  No valid authentication method found");
    false
}

fn is_public_endpoint(request: &str) -> bool {
    // Allow certain endpoints without authentication for ONVIF discovery
    let public_endpoints = [
        "GetCapabilities",
        "GetDeviceInformation",
        "GetServices",
        "GetSystemDateAndTime",
        "GetServiceCapabilities",
        "snapshot.jpg",
    ];

    for endpoint in &public_endpoints {
        // Check various patterns where the endpoint might appear
        if request.contains(endpoint)
            || request.contains(&format!("<{endpoint}>"))
            || request.contains(&format!("<{endpoint}/>"))
            || request.contains(&format!(":{endpoint}"))
            || request.contains(&format!("<{endpoint} "))
            || request.contains(&format!("tds:{endpoint}"))
            || request.contains(&format!("trt:{endpoint}"))
            || request.contains(&format!("soap:{endpoint}"))
        {
            println!("  Detected public endpoint: {endpoint}");
            return true;
        }
    }

    println!("  Request does not match any public endpoint patterns");
    false
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

fn validate_digest_auth(auth_header: &str, request: &str, username: &str, password: &str) -> bool {
    // Parse Digest authentication header
    // Format: Digest username="user", realm="realm", nonce="nonce", uri="/path", response="hash"
    let mut auth_params = std::collections::HashMap::new();

    // Remove "Digest " prefix and split by comma
    if let Some(digest_part) = auth_header.strip_prefix("Digest ") {
        for param in digest_part.split(',') {
            let param = param.trim();
            if let Some(eq_pos) = param.find('=') {
                let key = param[..eq_pos].trim();
                let value = param[eq_pos + 1..].trim().trim_matches('"');
                auth_params.insert(key, value);
            }
        }
    }

    // Extract required parameters
    let auth_username = auth_params.get("username").unwrap_or(&"");
    let realm = auth_params.get("realm").unwrap_or(&"");
    let nonce = auth_params.get("nonce").unwrap_or(&"");
    let uri = auth_params.get("uri").unwrap_or(&"");
    let response = auth_params.get("response").unwrap_or(&"");

    let method = request
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .next()
        .unwrap_or("GET");

    println!("Digest Auth validation:");
    println!("  Username: {auth_username}");
    println!("  Realm: {realm}");
    println!("  Method: {method}");
    println!("  URI: {uri}");

    // Check username
    if auth_username != &username {
        println!("Digest Auth: Username mismatch");
        return false;
    }

    // Calculate expected response: MD5(HA1:nonce:HA2)
    // where HA1 = MD5(username:realm:password)
    // and HA2 = MD5(method:uri)

    let ha1 = format!("{username}:{realm}:{password}");
    let ha1_hash = format!("{:x}", md5::compute(ha1.as_bytes()));

    let ha2 = format!("{method}:{uri}");
    let ha2_hash = format!("{:x}", md5::compute(ha2.as_bytes()));

    let expected_response_str = format!("{ha1_hash}:{nonce}:{ha2_hash}");
    let expected_response = format!("{:x}", md5::compute(expected_response_str.as_bytes()));

    println!("  Expected response: {expected_response}");
    println!("  Provided response: {response}");

    if response == &expected_response {
        println!("Digest Auth: Authentication successful");
        true
    } else {
        println!("Digest Auth: Authentication failed");
        false
    }
}

fn validate_ws_security_auth(request: &str, username: &str, password: &str) -> bool {
    println!("  WS-Security validation starting...");

    // Parse WS-Security UsernameToken
    if let (Some(user_start), Some(user_end)) =
        (request.find("<Username>"), request.find("</Username>"))
    {
        let provided_username = &request[user_start + 10..user_end];
        if provided_username != username {
            println!(
                "  WS-Security: Username mismatch. Expected: {username}, Got: {provided_username}"
            );
            return false;
        }
    } else {
        println!("  WS-Security: No username found in request");
        return false;
    }

    // Look for different password element patterns
    if let Some(password_start) = request.find("<Password") {
        // Find the end of the opening tag
        if let Some(tag_end) = request[password_start..].find('>') {
            let tag_content = &request[password_start..password_start + tag_end + 1];

            // Find the password value
            if let Some(pwd_end) = request[password_start + tag_end + 1..].find("</Password>") {
                let password_value =
                    &request[password_start + tag_end + 1..password_start + tag_end + 1 + pwd_end];

                // Check what type of password authentication is being used
                if tag_content.contains("PasswordDigest") {
                    println!("  WS-Security: Found PasswordDigest type");

                    // Extract nonce - look for various nonce patterns
                    let nonce = extract_ws_security_element(request, "Nonce");

                    // Extract created timestamp - look for various created patterns
                    let created = extract_ws_security_element(request, "Created");

                    // If either is None, we can't validate
                    if nonce.is_none() || created.is_none() {
                        println!("  WS-Security: Missing nonce or created timestamp");
                        return false;
                    }

                    let nonce = nonce.unwrap();
                    let created = created.unwrap();

                    // Decode the nonce from base64
                    let nonce_bytes = match general_purpose::STANDARD.decode(nonce) {
                        Ok(bytes) => bytes,
                        Err(_) => {
                            println!("  WS-Security: Failed to decode nonce");
                            return false;
                        }
                    };

                    // Calculate expected password digest
                    // PasswordDigest = Base64(SHA1(Nonce + Created + Password))
                    let mut hasher = sha1::Sha1::new();
                    hasher.update(&nonce_bytes);
                    hasher.update(created.as_bytes());
                    hasher.update(password.as_bytes());
                    let digest = hasher.finalize();
                    let expected_digest = general_purpose::STANDARD.encode(digest);

                    println!("  Expected digest: {expected_digest}");
                    println!("  Provided digest: {password_value}");

                    if password_value == expected_digest {
                        println!("  WS-Security: Authentication successful");
                        true
                    } else {
                        println!("  WS-Security: Authentication failed - digest mismatch");
                        false
                    }
                } else {
                    println!("  WS-Security: Using plain text password");
                    if password_value == password {
                        println!("  WS-Security: Authentication successful");
                        true
                    } else {
                        println!("  WS-Security: Authentication failed - password mismatch");
                        false
                    }
                }
            } else {
                println!("  WS-Security: Malformed Password element - no closing tag");
                false
            }
        } else {
            println!("  WS-Security: Malformed Password element - no closing >");
            false
        }
    } else {
        println!("  WS-Security: No Password element found");
        false
    }
}

fn extract_ws_security_element(request: &str, element_name: &str) -> Option<String> {
    // Look for opening tag with various prefixes and potential attributes
    for prefix in ["", "wsu:", "wsse:", "s:", "soap:"] {
        let tag_start = format!("<{prefix}{element_name}");

        if let Some(open_pos) = request.find(&tag_start) {
            // Find the end of the opening tag (either > or space)
            let content_start = if let Some(gt_pos) = request[open_pos..].find('>') {
                open_pos + gt_pos + 1
            } else {
                continue;
            };

            // Look for the closing tag
            let close_tag = format!("</{prefix}{element_name}>");
            if let Some(close_pos) = request[content_start..].find(&close_tag) {
                let content_end = content_start + close_pos;
                let content = request[content_start..content_end].trim();

                println!("  Found {element_name}: '{content}'");
                return Some(content.to_string());
            }
        }
    }

    println!("  Could not find element: {element_name}");
    None
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

fn send_snapshot_image_response(
    stream: &mut TcpStream,
    rtsp_stream_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Capturing snapshot from RTSP stream: {rtsp_stream_url}");

    match capture_rtsp_snapshot(rtsp_stream_url) {
        Ok(image_data) => {
            println!(
                "Successfully captured snapshot ({} bytes)",
                image_data.len()
            );
            send_image_response(stream, &image_data)
        }
        Err(e) => {
            eprintln!("Failed to capture snapshot: {e}");
            send_error_response(stream, "Failed to capture snapshot")
        }
    }
}

fn capture_rtsp_snapshot(rtsp_url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::process::Command;
    use tempfile::NamedTempFile;

    // Create a temporary file for the snapshot
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    println!("Using temporary file: {temp_path:?}");

    // Use FFmpeg to capture a single frame from the RTSP stream
    let output = Command::new("ffmpeg")
        .args([
            "-rtsp_transport",
            "tcp", // Use TCP for more reliable connection
            "-i",
            rtsp_url, // Input RTSP stream
            "-vframes",
            "1", // Capture only 1 frame
            "-q:v",
            "2", // High quality JPEG
            "-f",
            "image2",                    // Force image2 format
            "-y",                        // Overwrite output file
            temp_path.to_str().unwrap(), // Output file
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg failed: {stderr}").into());
    }

    // Read the captured image file
    let image_data = std::fs::read(temp_path)?;

    if image_data.is_empty() {
        return Err("Captured image is empty".into());
    }

    println!("FFmpeg capture successful");
    Ok(image_data)
}

fn send_image_response(
    stream: &mut TcpStream,
    image_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\nAccept-Ranges: bytes\r\nContent-Disposition: inline; filename=\"snapshot.jpg\"\r\n\r\n",
        image_data.len()
    );

    // Send HTTP headers
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    // Send image data
    stream.write_all(image_data)?;
    stream.flush()?;

    // Properly shutdown the connection to signal end of response
    let _ = stream.shutdown(std::net::Shutdown::Write);

    println!(
        "Sent snapshot response with {} bytes of image data",
        image_data.len()
    );
    Ok(())
}

fn send_error_response(
    stream: &mut TcpStream,
    error_message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = format!("Error: {error_message}");
    send_http_response(stream, "500 Internal Server Error", "text/plain", &body)
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
