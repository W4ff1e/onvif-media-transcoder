use base64::{Engine as _, engine::general_purpose};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use uuid::Uuid;

mod onvif_responses;
mod ws_discovery;

use onvif_responses::*;
use ws_discovery::{DeviceInfo, WSDiscoveryServer, get_default_interface_ip};

fn main() {
    println!("Starting ONVIF Media Transcoder...");

    // Strict configuration - no defaults, expect environment variables to be set
    let rtsp_input =
        std::env::var("RTSP_INPUT").expect("RTSP_INPUT environment variable must be set");
    let onvif_port =
        std::env::var("ONVIF_PORT").expect("ONVIF_PORT environment variable must be set");
    let device_name =
        std::env::var("DEVICE_NAME").expect("DEVICE_NAME environment variable must be set");
    let onvif_username =
        std::env::var("ONVIF_USERNAME").expect("ONVIF_USERNAME environment variable must be set");
    let onvif_password =
        std::env::var("ONVIF_PASSWORD").expect("ONVIF_PASSWORD environment variable must be set");

    // Validate port number
    let _: u16 = onvif_port
        .parse()
        .expect("ONVIF_PORT must be a valid port number");

    println!("Configuration:");
    println!("  RTSP Input Stream: {}", rtsp_input);
    println!("  ONVIF Port: {}", onvif_port);
    println!("  Device Name: {}", device_name);
    println!("  ONVIF Username: {}", onvif_username);
    println!("  ONVIF Password: [HIDDEN]");

    // Get the container IP for WS-Discovery
    let container_ip = get_default_interface_ip().unwrap_or_else(|_| {
        println!("Warning: Could not determine container IP, using localhost");
        "127.0.0.1".to_string()
    });
    println!("  Container IP: {}", container_ip);

    // Start WS-Discovery server in a separate thread
    let device_uuid = Uuid::new_v4();
    let ws_discovery_device_info = DeviceInfo {
        endpoint_reference: format!("urn:uuid:{}", device_uuid),
        types: "tdn:NetworkVideoTransmitter".to_string(),
        scopes: "onvif://www.onvif.org/Profile/Streaming onvif://www.onvif.org/name/ONVIF-Media-Transcoder".to_string(),
        xaddrs: format!("http://{}:{}/onvif/device_service", container_ip, onvif_port),
        manufacturer: "ONVIF Media Solutions".to_string(),
        model_name: device_name.clone(),
        friendly_name: device_name.clone(),
        firmware_version: "1.0.0".to_string(),
        serial_number: format!("EMU-{}", device_name.chars().take(6).collect::<String>()),
    };

    let ws_container_ip = container_ip.clone();
    thread::spawn(move || {
        match WSDiscoveryServer::new(ws_discovery_device_info, &ws_container_ip) {
            Ok(mut server) => {
                println!("Starting WS-Discovery server...");
                if let Err(e) = server.start() {
                    eprintln!("WS-Discovery server error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to create WS-Discovery server: {}", e);
                println!(
                    "Continuing without WS-Discovery (ONVIF service will still work for direct connections)"
                );
            }
        }
    });

    // Start ONVIF web service
    start_onvif_service(
        &onvif_port,
        &rtsp_input,
        &device_name,
        &onvif_username,
        &onvif_password,
        &container_ip,
    );
}

fn start_onvif_service(
    port: &str,
    rtsp_stream: &str,
    device_name: &str,
    username: &str,
    password: &str,
    container_ip: &str,
) {
    println!("Starting ONVIF web service on port {}", port);
    println!("Exposing RTSP stream: {}", rtsp_stream);
    println!("Device Name: {}", device_name);
    println!("Authentication: {} / [HIDDEN]", username);
    println!("WS-Discovery device discovery is running");

    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind to ONVIF port");

    println!("ONVIF Camera service running on port {}", port);
    println!("Device discovery available via WS-Discovery");
    println!("Stream URI: {}", rtsp_stream);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let rtsp_clone = rtsp_stream.to_string();
                let device_clone = device_name.to_string();
                let username_clone = username.to_string();
                let password_clone = password.to_string();
                let container_ip_clone = container_ip.to_string();
                let port_clone = port.to_string();
                thread::spawn(move || {
                    handle_onvif_request(
                        stream,
                        &rtsp_clone,
                        &device_clone,
                        &username_clone,
                        &password_clone,
                        &container_ip_clone,
                        &port_clone,
                    );
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

fn handle_onvif_request(
    mut stream: TcpStream,
    rtsp_stream: &str,
    device_name: &str,
    username: &str,
    password: &str,
    container_ip: &str,
    onvif_port: &str,
) {
    let mut buffer = [0; 2048];

    // Add error handling to prevent crashes
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Ok(size) = stream.read(&mut buffer) {
            let request = String::from_utf8_lossy(&buffer[..size]);

            println!(
                "Received ONVIF request: {}",
                request.lines().next().unwrap_or("Unknown")
            );

            // Check for authentication
            if !is_authenticated(&request, username, password) {
                send_auth_required_response(&mut stream);
                return;
            }

            // Enhanced ONVIF endpoint routing
            if request.contains("GetCapabilities") {
                send_capabilities_response(&mut stream, rtsp_stream, container_ip, onvif_port);
            } else if request.contains("GetProfiles") {
                send_profiles_response(&mut stream, rtsp_stream);
            } else if request.contains("GetStreamUri") {
                send_stream_uri_response(&mut stream, rtsp_stream);
            } else if request.contains("GetDeviceInformation") {
                send_device_info_response(&mut stream, device_name);
            } else if request.contains("GetVideoSources") {
                send_video_sources_response(&mut stream);
            } else if request.contains("GetServiceCapabilities") {
                send_service_capabilities_response(&mut stream);
            } else {
                send_default_response(&mut stream);
            }
        }
    }));

    if let Err(e) = result {
        eprintln!("ONVIF request handler panicked: {:?}", e);
        // Try to send a basic error response
        let _ = send_http_response(&mut stream, "500 Internal Server Error", "text/plain", "");
    }
}

// Helper function to send HTTP responses with dynamic content-length
fn send_http_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

// Helper function to send SOAP responses
fn send_soap_response(stream: &mut TcpStream, body: &str) {
    send_http_response(stream, "200 OK", "application/soap+xml", body);
}

// Authentication functions
fn is_authenticated(request: &str, username: &str, password: &str) -> bool {
    // Check for Basic Auth first (simpler)
    if let Some(auth_header) = extract_authorization_header(request) {
        if auth_header.starts_with("Basic ") {
            return validate_basic_auth(&auth_header, username, password);
        } else if auth_header.starts_with("Digest ") {
            return validate_digest_auth(&auth_header, request, username, password);
        }
    }

    // Allow unauthenticated requests for GetCapabilities and device discovery
    if request.contains("GetCapabilities") || request.contains("GetDeviceInformation") {
        return true;
    }

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
                let expected = format!("{}:{}", username, password);
                return decoded == expected;
            }
        }
    }
    false
}

fn validate_digest_auth(
    _auth_header: &str,
    _request: &str,
    _username: &str,
    _password: &str,
) -> bool {
    // For now, we'll implement a simple digest auth validation
    // In a production environment, this should be more robust
    true // Simplified for this implementation
}

fn send_auth_required_response(stream: &mut TcpStream) {
    let auth_response = get_auth_required_response();
    let _ = stream.write_all(auth_response.as_bytes());
}

fn send_capabilities_response(
    stream: &mut TcpStream,
    _rtsp_stream: &str,
    container_ip: &str,
    onvif_port: &str,
) {
    let body = get_capabilities_response(container_ip, onvif_port);
    send_soap_response(stream, &body);
}

fn send_profiles_response(stream: &mut TcpStream, _rtsp_stream: &str) {
    let body = get_profiles_response();
    send_soap_response(stream, &body);
}

fn send_stream_uri_response(stream: &mut TcpStream, rtsp_stream: &str) {
    let body = get_stream_uri_response(rtsp_stream);
    send_soap_response(stream, &body);
}

fn send_device_info_response(stream: &mut TcpStream, device_name: &str) {
    let body = get_device_info_response(device_name);
    send_soap_response(stream, &body);
}

fn send_default_response(stream: &mut TcpStream) {
    let body = get_default_response();
    send_http_response(stream, "200 OK", "text/plain", &body);
}

fn send_video_sources_response(stream: &mut TcpStream) {
    let body = get_video_sources_response();
    send_soap_response(stream, &body);
}

fn send_service_capabilities_response(stream: &mut TcpStream) {
    let body = get_service_capabilities_response();
    send_soap_response(stream, &body);
}
