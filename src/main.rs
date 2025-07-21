use base64::{Engine as _, engine::general_purpose};
use sha1::Digest;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use uuid::Uuid;

mod onvif_endpoints;
mod onvif_responses;
mod ws_discovery;

use onvif_endpoints::UNSUPPORTED_ENDPOINTS;
use onvif_responses::*;
use ws_discovery::{DeviceInfo, WSDiscoveryServer, get_default_interface_ip};

/// Configuration structure for the ONVIF Media Transcoder
#[derive(Debug, Clone)]
struct Config {
    rtsp_stream_url: String,
    onvif_port: String,
    device_name: String,
    onvif_username: String,
    onvif_password: String,
    container_ip: String,
}

impl Config {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Reading rtsp_stream_url environment variable...");
        let rtsp_stream_url = std::env::var("rtsp_stream_url")
            .map_err(|_| "rtsp_stream_url environment variable must be set")?;
        println!("rtsp_stream_url: {rtsp_stream_url}");

        println!("Reading ONVIF_PORT environment variable...");
        let onvif_port = std::env::var("ONVIF_PORT")
            .map_err(|_| "ONVIF_PORT environment variable must be set")?;
        println!("ONVIF_PORT: {onvif_port}");

        println!("Reading DEVICE_NAME environment variable...");
        let device_name = std::env::var("DEVICE_NAME")
            .map_err(|_| "DEVICE_NAME environment variable must be set")?;
        println!("DEVICE_NAME: {device_name}");

        println!("Reading ONVIF_USERNAME environment variable...");
        let onvif_username = std::env::var("ONVIF_USERNAME")
            .map_err(|_| "ONVIF_USERNAME environment variable must be set")?;
        println!("ONVIF_USERNAME: {onvif_username}");

        println!("Reading ONVIF_PASSWORD environment variable...");
        let onvif_password = std::env::var("ONVIF_PASSWORD")
            .map_err(|_| "ONVIF_PASSWORD environment variable must be set")?;
        println!(
            "ONVIF_PASSWORD: [HIDDEN] (length: {})",
            onvif_password.len()
        );

        // Validate port number
        println!("Validating port number...");
        let _: u16 = onvif_port
            .parse()
            .map_err(|_| "ONVIF_PORT must be a valid port number")?;
        println!("Port validation successful");

        // Get the container IP for WS-Discovery
        println!("Detecting container IP...");
        let container_ip = match get_default_interface_ip() {
            Ok(ip) => {
                println!("Container IP detected: {ip}");
                ip
            }
            Err(e) => {
                eprintln!("Warning: Could not determine container IP ({e}), using localhost");
                "127.0.0.1".to_string()
            }
        };

        println!("Configuration creation completed successfully");
        Ok(Config {
            rtsp_stream_url: rtsp_stream_url,
            onvif_port,
            device_name,
            onvif_username,
            onvif_password,
            container_ip,
        })
    }

    fn display(&self) {
        println!("Configuration:");
        println!("  RTSP Input Stream: {}", self.rtsp_stream_url);
        println!("  ONVIF Port: {}", self.onvif_port);
        println!("  Device Name: {}", self.device_name);
        println!("  ONVIF Username: {}", self.onvif_username);
        println!("  ONVIF Password: [HIDDEN]");
        println!("  Container IP: {}", self.container_ip);
    }
}

/// Main entry point for the ONVIF Media Transcoder
fn main() {
    println!("Starting ONVIF Media Transcoder...");

    // Set up panic hook for better crash reporting
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC in ONVIF service: {panic_info}");
        if let Some(location) = panic_info.location() {
            eprintln!(
                "  Location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
        eprintln!("  Thread: {:?}", std::thread::current().id());
        // Don't exit immediately - let the main thread handle the error
        // std::process::exit(1);
    }));

    // Load configuration from environment variables
    println!("Loading configuration from environment variables...");
    let config = match Config::from_env() {
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

    // Start WS-Discovery server in a separate thread
    println!("Initializing WS-Discovery server...");
    if let Err(e) = start_ws_discovery_server(&config) {
        eprintln!("Failed to start WS-Discovery server: {e}");
        println!(
            "Continuing without WS-Discovery (ONVIF service will still work for direct connections)"
        );
    } else {
        println!("WS-Discovery server initialization completed");
    }

    // Start ONVIF web service (this will block)
    println!("Starting ONVIF web service...");

    // Start a heartbeat thread to show the service is still alive
    let heartbeat_config = config.clone();
    thread::spawn(move || {
        let mut counter = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            counter += 1;
            println!(
                "HEARTBEAT #{}: ONVIF service is running (port: {})",
                counter, heartbeat_config.onvif_port
            );
        }
    });

    if let Err(e) = start_onvif_service(&config) {
        eprintln!("ONVIF service error: {e}");
        std::process::exit(1);
    }
}

fn start_ws_discovery_server(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating WS-Discovery device info...");
    let device_uuid = Uuid::new_v4();
    let ws_discovery_device_info = DeviceInfo {
        endpoint_reference: format!("urn:uuid:{device_uuid}"),
        types: "tdn:NetworkVideoTransmitter".to_string(),
        scopes: "onvif://www.onvif.org/Profile/Streaming onvif://www.onvif.org/name/ONVIF-Media-Transcoder".to_string(),
        xaddrs: format!("http://{}:{}/onvif/device_service", config.container_ip, config.onvif_port),
        manufacturer: "ONVIF Media Solutions".to_string(),
        model_name: config.device_name.clone(),
        friendly_name: config.device_name.clone(),
        firmware_version: "1.0.0".to_string(),
        serial_number: format!("EMU-{}", config.device_name.chars().take(6).collect::<String>()),
    };

    let ws_container_ip = config.container_ip.clone();
    println!("Creating WS-Discovery thread with IP: {ws_container_ip}");
    println!("Device UUID: {device_uuid}");
    println!("XAddrs: {}", ws_discovery_device_info.xaddrs);

    let spawn_result = std::panic::catch_unwind(|| {
        thread::spawn(move || {
            println!("WS-Discovery thread started");

            // Add error handling and retry logic
            let mut attempts = 0;
            let max_attempts = 3;

            while attempts < max_attempts {
                attempts += 1;
                println!("WS-Discovery attempt {attempts} of {max_attempts}");

                match WSDiscoveryServer::new(ws_discovery_device_info.clone(), &ws_container_ip) {
                    Ok(mut server) => {
                        println!("WS-Discovery server created successfully on attempt {attempts}");
                        match server.start() {
                            Ok(_) => {
                                println!("WS-Discovery server completed normally");
                                return; // Exit thread normally
                            }
                            Err(e) => {
                                eprintln!("WS-Discovery server error on attempt {attempts}: {e}");
                                if attempts < max_attempts {
                                    println!("Retrying WS-Discovery in 2 seconds...");
                                    std::thread::sleep(std::time::Duration::from_secs(2));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to create WS-Discovery server on attempt {attempts}: {e}"
                        );
                        if attempts < max_attempts {
                            println!("Retrying WS-Discovery creation in 2 seconds...");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                    }
                }
            }

            eprintln!(
                "WS-Discovery failed after {max_attempts} attempts - continuing without device discovery"
            );
            println!("WS-Discovery thread ending");
        })
    });

    match spawn_result {
        Ok(handle) => {
            println!("WS-Discovery server thread started successfully");
            std::mem::drop(handle); // Let it run independently
        }
        Err(_) => {
            eprintln!("Failed to start WS-Discovery thread - panic occurred");
            return Err("WS-Discovery thread spawn failed".into());
        }
    }

    // Give the thread a moment to start
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!("WS-Discovery server initialization completed");
    Ok(())
}

fn start_onvif_service(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ONVIF web service on port {}", config.onvif_port);
    println!("Exposing RTSP stream: {}", config.rtsp_stream_url);
    println!("Device Name: {}", config.device_name);
    println!("Authentication: {} / [HIDDEN]", config.onvif_username);
    println!("WS-Discovery device discovery is running");

    let bind_addr = format!("0.0.0.0:{}", config.onvif_port);
    println!("Attempting to bind to address: {bind_addr}");

    // Add more detailed error handling for port binding
    let listener = match TcpListener::bind(&bind_addr) {
        Ok(listener) => {
            println!("Successfully bound to {bind_addr}");
            listener
        }
        Err(e) => {
            let error_msg = format!("Failed to bind to ONVIF port {}: {}", config.onvif_port, e);
            eprintln!("{error_msg}");

            // Check if port is already in use
            if e.kind() == std::io::ErrorKind::AddrInUse {
                eprintln!(
                    "Port {} is already in use. Please check if another service is running on this port.",
                    config.onvif_port
                );
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                eprintln!(
                    "Permission denied to bind to port {}. May need elevated privileges for ports < 1024.",
                    config.onvif_port
                );
            }

            return Err(error_msg.into());
        }
    };

    println!("Successfully bound to {bind_addr}");
    println!("ONVIF Camera service running on port {}", config.onvif_port);
    println!("Device discovery available via WS-Discovery");
    println!("Stream URI: {}", config.rtsp_stream_url);

    // Add a keepalive mechanism to detect if the service is still running
    let mut connection_count = 0u64;

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                // Set TCP socket options for better WiFi performance
                if let Err(e) = stream.set_nodelay(true) {
                    eprintln!("Warning: Failed to set TCP_NODELAY: {e}");
                }

                connection_count += 1;
                println!(
                    "Accepted connection #{} from: {:?}",
                    connection_count,
                    stream.peer_addr()
                );

                let config_clone = config.clone();
                let conn_id = connection_count;

                let thread_result = std::panic::catch_unwind(|| {
                    thread::spawn(move || {
                        println!("Thread started for connection #{conn_id}");
                        match handle_onvif_request(stream, &config_clone) {
                            Ok(_) => {
                                println!("Successfully handled request #{conn_id}");
                            }
                            Err(e) => {
                                eprintln!("Error handling ONVIF request #{conn_id}: {e}");
                                // Don't panic on request handling errors
                            }
                        }
                        println!("Thread completed for connection #{conn_id}");
                    })
                });

                match thread_result {
                    Ok(handle) => {
                        println!("Successfully spawned thread for connection #{connection_count}");
                        // We could store the handle if we wanted to join later
                        std::mem::drop(handle);
                    }
                    Err(_) => {
                        eprintln!(
                            "Failed to spawn thread for connection #{connection_count} - panic occurred"
                        );
                        // Continue serving other connections even if one thread panics
                    }
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
                // Don't exit on connection errors, just log and continue
                std::thread::sleep(std::time::Duration::from_millis(100));
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
    // Set socket timeouts for better WiFi performance
    let timeout = std::time::Duration::from_secs(30);
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    // Get client info for debugging
    let client_addr = stream
        .peer_addr()
        .unwrap_or_else(|_| "unknown:0".parse().unwrap());
    let connection_start = std::time::Instant::now();

    println!("New connection from: {client_addr}");
    let mut buffer = [0; 4096]; // Increased buffer size for larger SOAP requests

    // Use proper error handling instead of panic catching
    let size = stream
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read from stream: {e}"))?;

    if size == 0 {
        println!("  Connection closed by client (0 bytes read)");
        return Ok(()); // Connection closed by client
    }

    if size >= 4096 {
        println!("  Request may be truncated (reached buffer limit of 4096 bytes)");
    }

    let request = String::from_utf8_lossy(&buffer[..size]);

    // Enhanced logging for debugging
    let first_line = request.lines().next().unwrap_or("Unknown");
    println!("Received ONVIF request: {first_line}");

    // Debug: Check if we have SOAP content
    if request.contains("Envelope") && (request.contains("soap:") || request.contains("v:")) {
        println!("  Contains SOAP envelope - valid ONVIF request");
    } else {
        println!("  No SOAP envelope detected - possibly incomplete request");
        println!("  Request size: {size} bytes");
        if size < 2048 {
            println!(
                "  Full request: {}",
                request.replace('\r', "\\r").replace('\n', "\\n")
            );
        }
    }

    // Log the WS-Security section if present for debugging
    if request.contains("<UsernameToken>") {
        if let Some(start) = request.find("<UsernameToken>") {
            if let Some(end) = request.find("</UsernameToken>") {
                let token_section = &request[start..end + 16]; // +16 for closing tag
                println!("  WS-Security UsernameToken section:");
                for line in token_section.lines() {
                    println!("    {}", line.trim());
                }
            }
        }
    }

    // Check for authentication - but allow some endpoints without auth
    let requires_auth = !is_public_endpoint(&request);

    println!("  Authentication analysis:");
    println!("    - Endpoint requires auth: {requires_auth}");

    // Log what authentication headers/tokens we found
    if let Some(auth_header) = extract_authorization_header(&request) {
        if auth_header.starts_with("Basic ") {
            println!("    - Found HTTP Basic Auth header");
        } else if auth_header.starts_with("Digest ") {
            println!("    - Found HTTP Digest Auth header");
        } else {
            println!("    - Found unknown Authorization header");
        }
    } else {
        println!("    - No Authorization header found");
    }

    if request.contains("<UsernameToken>") {
        println!("    - Found WS-Security UsernameToken");
    } else {
        println!("    - No WS-Security UsernameToken found");
    }

    if requires_auth && !is_authenticated(&request, &config.onvif_username, &config.onvif_password)
    {
        println!("  Authentication required - sending 401 response");

        // Check if this is a SOAP request that might benefit from WS-Security auth
        if request.contains("soap:Envelope") || request.contains("<soap:") {
            println!("  Detected SOAP request - sending WS-Security auth fault");
            send_ws_security_auth_fault(&mut stream)?;
        } else {
            println!("  Sending standard HTTP auth challenge");
            send_auth_required_response(&mut stream)?;
        }
        return Ok(());
    } else if requires_auth {
        println!("  Authentication successful");
    } else {
        println!("  Public endpoint - no authentication required");
    }

    // Enhanced ONVIF endpoint routing with comprehensive error handling
    if request.contains("GetCapabilities") {
        println!("Handling supported endpoint: GetCapabilities");
        send_capabilities_response(
            &mut stream,
            &config.rtsp_stream_url,
            &config.container_ip,
            &config.onvif_port,
        )?;
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
            eprintln!("   Request details: {first_line}");
            eprintln!("   This endpoint is not implemented in this ONVIF transcoder");
            eprintln!("   Consider adding support for this endpoint if needed");
            send_unsupported_endpoint_response(&mut stream, &endpoint)?;
        } else {
            println!("Unknown request type (not ONVIF SOAP): {first_line}");
            send_default_response(&mut stream)?;
        }
    }

    let duration = connection_start.elapsed();
    println!("Connection handled in {duration:?} for {client_addr}");

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
    println!("    Starting authentication validation...");

    // Check for Basic Auth first (simpler)
    if let Some(auth_header) = extract_authorization_header(request) {
        if auth_header.starts_with("Basic ") {
            println!("    Attempting Basic Auth validation...");
            return validate_basic_auth(&auth_header, username, password);
        } else if auth_header.starts_with("Digest ") {
            println!("    Attempting Digest Auth validation...");
            return validate_digest_auth(&auth_header, request, username, password);
        }
    }

    // Check for WS-Security Username Token (Digest)
    if request.contains("<UsernameToken>") && request.contains("<Username>") {
        println!("    Found WS-Security UsernameToken, attempting validation...");
        return validate_ws_security_auth(request, username, password);
    }

    println!("    No valid authentication method found");
    false
}

fn is_public_endpoint(request: &str) -> bool {
    // Allow certain endpoints without authentication for ONVIF discovery
    request.contains("GetCapabilities") || 
    request.contains("GetDeviceInformation") || 
    request.contains("GetServices") ||
    request.contains("GetSystemDateAndTime") ||
    request.contains("GetServiceCapabilities") ||
    // Also check for these endpoints in SOAP body format
    request.contains("<GetCapabilities") ||
    request.contains("<GetDeviceInformation") ||
    request.contains("<GetServices") ||
    request.contains("<GetSystemDateAndTime") ||
    request.contains("<GetServiceCapabilities") ||
    // Check for namespaced versions
    request.contains(":GetCapabilities") ||
    request.contains(":GetDeviceInformation") ||
    request.contains(":GetServices") ||
    request.contains(":GetSystemDateAndTime") ||
    request.contains(":GetServiceCapabilities")
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
    println!("    WS-Security validation starting...");

    // Parse WS-Security UsernameToken
    if let (Some(user_start), Some(user_end)) =
        (request.find("<Username>"), request.find("</Username>"))
    {
        let provided_username = &request[user_start + 10..user_end];
        if provided_username != username {
            println!(
                "    WS-Security: Username mismatch. Expected: {username}, Got: {provided_username}"
            );
            return false;
        }
    } else {
        println!("    WS-Security: No username found in request");
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
                    println!("    WS-Security: Found PasswordDigest type");

                    // Extract nonce - look for various nonce patterns
                    let nonce = if request.contains("<Nonce") && request.contains("</Nonce>") {
                        // Simple approach: extract everything between > and </Nonce>
                        if let Some(nonce_start) = request.find("<Nonce") {
                            if let Some(nonce_tag_end) = request[nonce_start..].find('>') {
                                let content_start = nonce_start + nonce_tag_end + 1;
                                if let Some(nonce_end_pos) =
                                    request[content_start..].find("</Nonce>")
                                {
                                    &request[content_start..content_start + nonce_end_pos]
                                } else {
                                    println!("    WS-Security: Found <Nonce but no </Nonce>");
                                    ""
                                }
                            } else {
                                println!("    WS-Security: Found <Nonce but no closing >");
                                ""
                            }
                        } else {
                            ""
                        }
                    } else {
                        println!("    WS-Security: No nonce found");
                        ""
                    };

                    // Extract created timestamp - look for various created patterns
                    let created = if request.contains("<Created") && request.contains("</Created>")
                    {
                        // Simple approach: extract everything between > and </Created>
                        if let Some(created_start) = request.find("<Created") {
                            if let Some(created_tag_end) = request[created_start..].find('>') {
                                let content_start = created_start + created_tag_end + 1;
                                if let Some(created_end_pos) =
                                    request[content_start..].find("</Created>")
                                {
                                    &request[content_start..content_start + created_end_pos]
                                } else {
                                    println!("    WS-Security: Found <Created but no </Created>");
                                    ""
                                }
                            } else {
                                println!("    WS-Security: Found <Created but no closing >");
                                ""
                            }
                        } else {
                            ""
                        }
                    } else {
                        println!("    WS-Security: No created timestamp found");
                        ""
                    };

                    if nonce.is_empty() {
                        println!(
                            "    WS-Security: No nonce found - PasswordDigest requires nonce and created timestamp"
                        );
                        // Some broken clients send PasswordDigest type but with plain password
                        if password_value == password {
                            println!("    WS-Security: Plain password fallback successful");
                            return true;
                        } else {
                            println!("    WS-Security: Plain password fallback failed");
                            return false;
                        }
                    }

                    if created.is_empty() {
                        println!(
                            "    WS-Security: No created timestamp found - PasswordDigest requires both nonce and created timestamp"
                        );
                        return false;
                    }

                    // Validate timestamp (should be within reasonable time window)
                    if let Ok(created_time) = chrono::DateTime::parse_from_rfc3339(created) {
                        let now = chrono::Utc::now();
                        let time_diff =
                            now.signed_duration_since(created_time.with_timezone(&chrono::Utc));

                        // Allow 5 minutes before and after for clock skew
                        if time_diff.num_seconds().abs() > 300 {
                            println!(
                                "    WS-Security: Timestamp validation failed. Time difference: {} seconds",
                                time_diff.num_seconds()
                            );
                            return false;
                        }
                    } else {
                        println!("    WS-Security: Invalid timestamp format: {created}");
                        return false;
                    }

                    // Decode the nonce from base64
                    let nonce_bytes = match general_purpose::STANDARD.decode(nonce) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            println!("    WS-Security: Failed to decode nonce: {e}");
                            return false;
                        }
                    };

                    // Calculate expected digest: Base64(SHA1(nonce + created + password))
                    let mut hasher = sha1::Sha1::new();
                    hasher.update(&nonce_bytes);
                    hasher.update(created.as_bytes());
                    hasher.update(password.as_bytes());
                    let hash_result = hasher.finalize();

                    let expected_digest = general_purpose::STANDARD.encode(hash_result);

                    if password_value == expected_digest {
                        println!("    WS-Security: Authentication successful");
                        return true;
                    } else {
                        println!("    WS-Security: Digest validation failed");
                        return false;
                    }
                } else if tag_content.contains("PasswordText") {
                    println!("    WS-Security: Using PasswordText authentication");
                    if password_value == password {
                        println!("    WS-Security: Authentication successful");
                        return true;
                    } else {
                        println!("    WS-Security: Authentication failed");
                        return false;
                    }
                } else {
                    // Handle simple password element without type attribute
                    println!("    WS-Security: Using simple password authentication");
                    if password_value == password {
                        println!("    WS-Security: Authentication successful");
                        return true;
                    } else {
                        println!("    WS-Security: Authentication failed");
                        return false;
                    }
                }
            } else {
                println!("    WS-Security: Could not find closing </Password> tag");
            }
        } else {
            println!("    WS-Security: Malformed Password element - no closing >");
        }
    } else {
        println!("    WS-Security: No Password element found");
    }

    false
}

fn send_auth_required_response(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let auth_response = get_auth_required_response();
    stream
        .write_all(auth_response.as_bytes())
        .map_err(|e| format!("Failed to send auth required response: {e}").into())
}

fn send_ws_security_auth_fault(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let auth_fault = get_ws_security_auth_fault();
    stream
        .write_all(auth_fault.as_bytes())
        .map_err(|e| format!("Failed to send WS-Security auth fault: {e}").into())
}

fn send_capabilities_response(
    stream: &mut TcpStream,
    _rtsp_stream: &str,
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

    // Check for other SOAP action patterns that might be ONVIF
    if request.contains("soap:Envelope") || request.contains("soap:Body") {
        // Extract potential action from SOAPAction header or request body
        for line in request.lines() {
            if line.to_lowercase().contains("soapaction:") {
                if let Some(action) = line.split('"').nth(1) {
                    if action.contains("onvif.org") {
                        let action_name = action.split('/').next_back().unwrap_or("UnknownAction");
                        return Some(format!("Unknown ONVIF Action: {action_name}"));
                    }
                }
            }
        }

        // Try to extract action from SOAP body
        if let Some(body_start) = request.find("<soap:Body>") {
            if let Some(body_end) = request.find("</soap:Body>") {
                let body_content = &request[body_start..body_end];
                if let Some(action_start) = body_content.find('<') {
                    if let Some(action_end) = body_content[action_start + 1..].find('>') {
                        let action_tag =
                            &body_content[action_start + 1..action_start + 1 + action_end];
                        if action_tag.contains(':') && !action_tag.contains("soap:") {
                            let action_name =
                                action_tag.split(':').next_back().unwrap_or("UnknownAction");
                            return Some(format!("Unknown ONVIF Action: {action_name}"));
                        }
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_public_endpoint() {
        // Test public endpoints that don't require authentication
        assert!(is_public_endpoint("<GetCapabilities/>"));
        assert!(is_public_endpoint(
            "<soap:Body><GetCapabilities/></soap:Body>"
        ));
        assert!(is_public_endpoint(
            "<?xml version=\"1.0\"?><soap:Envelope><soap:Body><GetDeviceInformation/></soap:Body></soap:Envelope>"
        ));
        assert!(is_public_endpoint("<tds:GetServices/>"));
        assert!(is_public_endpoint("GetSystemDateAndTime"));

        // Test endpoints that require authentication
        assert!(!is_public_endpoint("<GetProfiles/>"));
        assert!(!is_public_endpoint("GetStreamUri"));
        assert!(!is_public_endpoint(
            "<soap:Body><GetSnapshotUri/></soap:Body>"
        ));
    }

    #[test]
    fn test_validate_basic_auth() {
        // Valid Basic Auth: admin:onvif-rust encoded as base64
        let valid_auth = "Basic YWRtaW46b252aWYtcnVzdA==";
        assert!(validate_basic_auth(valid_auth, "admin", "onvif-rust"));

        // Invalid credentials
        assert!(!validate_basic_auth(valid_auth, "admin", "wrong-password"));
        assert!(!validate_basic_auth(valid_auth, "wrong-user", "onvif-rust"));

        // Malformed auth header
        assert!(!validate_basic_auth(
            "Basic invalid-base64",
            "admin",
            "onvif-rust"
        ));
        assert!(!validate_basic_auth("Bearer token", "admin", "onvif-rust"));
    }

    #[test]
    fn test_extract_authorization_header() {
        let request_with_basic = "POST /onvif/device_service HTTP/1.1\r\nAuthorization: Basic YWRtaW46b252aWYtcnVzdA==\r\nContent-Type: application/soap+xml\r\n\r\n";
        assert_eq!(
            extract_authorization_header(request_with_basic),
            Some("Basic YWRtaW46b252aWYtcnVzdA==".to_string())
        );

        let request_with_digest = "POST /onvif/media_service HTTP/1.1\r\nAuthorization: Digest username=\"admin\", realm=\"ONVIF\"\r\n\r\n";
        assert_eq!(
            extract_authorization_header(request_with_digest),
            Some("Digest username=\"admin\", realm=\"ONVIF\"".to_string())
        );

        let request_without_auth =
            "POST /onvif/device_service HTTP/1.1\r\nContent-Type: application/soap+xml\r\n\r\n";
        assert_eq!(extract_authorization_header(request_without_auth), None);
    }

    #[test]
    fn test_validate_ws_security_auth() {
        // Test PasswordText authentication
        let ws_security_request = r#"
            <soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
                <soap:Header>
                    <Security xmlns="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd">
                        <UsernameToken>
                            <Username>admin</Username>
                            <Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordText">onvif-rust</Password>
                        </UsernameToken>
                    </Security>
                </soap:Header>
                <soap:Body><GetProfiles/></soap:Body>
            </soap:Envelope>
        "#;

        assert!(validate_ws_security_auth(
            ws_security_request,
            "admin",
            "onvif-rust"
        ));
        assert!(!validate_ws_security_auth(
            ws_security_request,
            "admin",
            "wrong-password"
        ));
        assert!(!validate_ws_security_auth(
            ws_security_request,
            "wrong-user",
            "onvif-rust"
        ));

        // Test simple password element without type
        let simple_ws_security = r#"
            <soap:Envelope>
                <soap:Header>
                    <UsernameToken>
                        <Username>admin</Username>
                        <Password>onvif-rust</Password>
                    </UsernameToken>
                </soap:Header>
                <soap:Body><GetProfiles/></soap:Body>
            </soap:Envelope>
        "#;

        assert!(validate_ws_security_auth(
            simple_ws_security,
            "admin",
            "onvif-rust"
        ));
        assert!(!validate_ws_security_auth(
            simple_ws_security,
            "admin",
            "wrong"
        ));
    }

    #[test]
    fn test_detect_unsupported_onvif_endpoint() {
        // Test known unsupported endpoints
        let ptz_request = r#"
            <soap:Envelope>
                <soap:Body><GetPresets/></soap:Body>
            </soap:Envelope>
        "#;

        let result = detect_unsupported_onvif_endpoint(ptz_request);
        assert!(result.is_some());
        assert!(result.unwrap().contains("GetPresets"));

        // Test supported endpoint (should return None)
        let supported_request = r#"
            <soap:Envelope>
                <soap:Body><GetCapabilities/></soap:Body>
            </soap:Envelope>
        "#;

        assert!(detect_unsupported_onvif_endpoint(supported_request).is_none());

        // Test non-ONVIF request
        let non_onvif_request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        assert!(detect_unsupported_onvif_endpoint(non_onvif_request).is_none());
    }

    #[test]
    fn test_config_from_env() {
        // Save original environment
        let original_vars: Vec<_> = [
            "rtsp_stream_url",
            "ONVIF_PORT",
            "DEVICE_NAME",
            "ONVIF_USERNAME",
            "ONVIF_PASSWORD",
        ]
        .iter()
        .map(|var| (var, std::env::var(var).ok()))
        .collect();

        // Set test environment variables
        unsafe {
            std::env::set_var("rtsp_stream_url", "rtsp://test:8554/stream");
            std::env::set_var("ONVIF_PORT", "8080");
            std::env::set_var("DEVICE_NAME", "Test-Camera");
            std::env::set_var("ONVIF_USERNAME", "testuser");
            std::env::set_var("ONVIF_PASSWORD", "testpass");
        }

        // Test successful config creation
        let config = Config::from_env();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.rtsp_stream_url, "rtsp://test:8554/stream");
        assert_eq!(config.onvif_port, "8080");
        assert_eq!(config.device_name, "Test-Camera");
        assert_eq!(config.onvif_username, "testuser");
        assert_eq!(config.onvif_password, "testpass");

        // Test invalid port
        unsafe {
            std::env::set_var("ONVIF_PORT", "invalid-port");
        }
        let config = Config::from_env();
        assert!(config.is_err());

        // Test missing environment variable
        unsafe {
            std::env::remove_var("rtsp_stream_url");
        }
        let config = Config::from_env();
        assert!(config.is_err());

        // Restore original environment
        unsafe {
            for (var, value) in original_vars {
                match value {
                    Some(val) => std::env::set_var(var, val),
                    None => std::env::remove_var(var),
                }
            }
        }
    }

    #[test]
    fn test_authentication_integration() {
        let username = "admin";
        let password = "onvif-rust";

        // Test public endpoint (no auth required)
        let public_request = r#"POST /onvif/device_service HTTP/1.1
Content-Type: application/soap+xml

<soap:Envelope><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>"#;

        assert!(is_public_endpoint(public_request));

        // Test protected endpoint with valid Basic Auth (proper HTTP format)
        let protected_request_with_auth = "POST /onvif/media_service HTTP/1.1\r\nAuthorization: Basic YWRtaW46b252aWYtcnVzdA==\r\nContent-Type: application/soap+xml\r\n\r\n<soap:Envelope><soap:Body><GetProfiles/></soap:Body></soap:Envelope>";

        assert!(!is_public_endpoint(protected_request_with_auth));
        assert!(is_authenticated(
            protected_request_with_auth,
            username,
            password
        ));

        // Test protected endpoint without auth
        let protected_request_no_auth = r#"POST /onvif/media_service HTTP/1.1
Content-Type: application/soap+xml

<soap:Envelope><soap:Body><GetProfiles/></soap:Body></soap:Envelope>"#;

        assert!(!is_public_endpoint(protected_request_no_auth));
        assert!(!is_authenticated(
            protected_request_no_auth,
            username,
            password
        ));
    }
}
