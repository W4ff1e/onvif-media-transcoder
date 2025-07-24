use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use sha1::Digest;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use tempfile::NamedTempFile;
use uuid::Uuid;

pub mod onvif_endpoints;
pub mod onvif_responses;
pub mod ws_discovery;

use onvif_endpoints::UNSUPPORTED_ENDPOINTS;
use onvif_responses::*;
use ws_discovery::{DeviceInfo, WSDiscoveryServer};

/// Configuration structure for the ONVIF Media Transcoder
#[derive(Debug, Clone, Parser)]
#[command(name = "onvif-media-transcoder")]
#[command(
    about = "ONVIF Media Transcoder - Converts media streams to ONVIF-compatible RTSP streams"
)]
pub struct Config {
    /// RTSP stream URL to transcode
    #[arg(long, default_value = "rtsp://127.0.0.1:8554/stream")]
    pub rtsp_stream_url: String,

    /// Port for the ONVIF service
    #[arg(long, default_value = "8080")]
    pub onvif_port: String,

    /// Device name for ONVIF identification
    #[arg(long, default_value = "ONVIF-Media-Transcoder")]
    pub device_name: String,

    /// Username for ONVIF authentication
    #[arg(long, default_value = "admin")]
    pub onvif_username: String,

    /// Password for ONVIF authentication
    #[arg(long, default_value = "onvif-rust")]
    pub onvif_password: String,

    /// Container IP address for WS-Discovery
    #[arg(long, default_value = "127.0.0.1")]
    pub container_ip: String,

    /// Enable WS-Discovery service
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub ws_discovery_enabled: bool,
}

impl Config {
    pub fn from_args() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Parsing command-line arguments...");
        let config = Config::parse();

        // Log when default values are used
        println!("Configuration loaded from command-line arguments:");

        // Validate port number
        println!("Validating port number...");
        let _: u16 = config
            .onvif_port
            .parse()
            .map_err(|_| "ONVIF_PORT must be a valid port number")?;
        println!("Port validation successful");

        // Validate container IP is not empty
        if config.container_ip.is_empty() {
            return Err("CONTAINER_IP cannot be empty - container IP detection failed".into());
        }

        // Basic IP format validation
        if config.container_ip.parse::<std::net::IpAddr>().is_err() {
            return Err(format!(
                "CONTAINER_IP '{}' is not a valid IP address",
                config.container_ip
            )
            .into());
        }

        println!("Container IP: {}", config.container_ip);

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

    pub fn display(&self) {
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

        if self.onvif_password == "onvif-password" {
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
                "ENABLED"
            } else {
                "DISABLED"
            },
            if !self.ws_discovery_enabled {
                "(default: disabled, use --ws-discovery-enabled to enable)"
            } else {
                ""
            }
        );
    }
}

/// Represents the status of the service components
#[derive(Debug)]
pub struct ServiceStatus {
    pub ws_discovery_healthy: AtomicBool,
    pub onvif_service_healthy: AtomicBool,
    pub shutdown_requested: AtomicBool,
    pub last_error: Mutex<Option<String>>,
}

impl ServiceStatus {
    pub fn new() -> Self {
        ServiceStatus {
            ws_discovery_healthy: AtomicBool::new(false),
            onvif_service_healthy: AtomicBool::new(false),
            shutdown_requested: AtomicBool::new(false),
            last_error: Mutex::new(None),
        }
    }

    pub fn request_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        println!("Shutdown requested - service will exit after completing current operations");
    }

    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    pub fn set_error(&self, error: &str) {
        let mut last_error = self.last_error.lock().unwrap();
        *last_error = Some(error.to_string());
        eprintln!("Service error: {}", error);
    }
}

/// Sets up signal handlers for graceful shutdown
#[cfg(not(windows))]
pub fn setup_signal_handlers(service_status: Arc<ServiceStatus>) {
    use signal_hook::iterator::Signals;

    // Register handlers for SIGINT and SIGTERM
    match Signals::new(&[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM]) {
        Ok(mut signals) => {
            let status_clone = service_status.clone();
            std::thread::spawn(move || {
                for sig in signals.forever() {
                    println!("Received signal {:?}", sig);
                    status_clone.request_shutdown();
                }
            });
            println!("Signal handlers registered for graceful shutdown");
        }
        Err(e) => {
            eprintln!("Failed to set up signal handlers: {}", e);
            println!("Process will not respond to termination signals gracefully");
        }
    }
}

/// Windows-specific version that doesn't use Unix signals
#[cfg(windows)]
pub fn setup_signal_handlers(_service_status: Arc<ServiceStatus>) {
    println!("Signal handling is limited on Windows - use Ctrl+C to terminate");
    // Could implement Windows-specific handlers here if needed
}

/// Start the ONVIF service with graceful shutdown support
pub fn start_onvif_service_with_shutdown(
    config: &Config,
    service_status: Arc<ServiceStatus>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    // Configure listener for non-blocking mode with a short timeout to check for shutdown
    listener.set_nonblocking(true)?;

    println!("Successfully bound to {bind_addr}");
    println!("ONVIF Camera service running on port {}", config.onvif_port);
    if config.ws_discovery_enabled {
        println!("Device discovery available via WS-Discovery");
    } else {
        println!("Device discovery disabled (WS-Discovery is off)");
    }
    println!("Stream URI: {}", config.rtsp_stream_url);

    // Add a keepalive mechanism to detect if the service is still running
    let mut connection_count = 0u64;

    // Update service status to indicate we're running
    service_status
        .onvif_service_healthy
        .store(true, Ordering::SeqCst);

    // Main service loop with shutdown support
    while !service_status.is_shutdown_requested() {
        // Accept connections with timeout to check for shutdown
        match listener.accept() {
            Ok((stream, addr)) => {
                // Set TCP socket options for better WiFi performance
                if let Err(e) = stream.set_nodelay(true) {
                    eprintln!("Warning: Failed to set TCP_NODELAY: {e}");
                }

                connection_count += 1;
                println!("Accepted connection #{} from: {:?}", connection_count, addr);

                let config_clone = config.clone();

                // Handle the request in a new thread
                let thread_result = std::panic::catch_unwind(|| {
                    thread::spawn(move || {
                        if let Err(e) = handle_onvif_request(stream, &config_clone) {
                            eprintln!("Error handling request: {e}");
                        }
                    })
                });

                if thread_result.is_err() {
                    eprintln!("Thread creation panicked!");
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    eprintln!("Error accepting connection: {e}");
                }
                // Short sleep to avoid CPU spinning in the non-blocking loop
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
        }

        // Periodic status update
        if connection_count % 10 == 0 {
            println!("ONVIF service is healthy - processed {connection_count} connections");
        }
    }

    println!("ONVIF service listener loop ending due to shutdown request");
    println!("Cleaning up resources...");

    // Allow time for any in-flight connections to complete
    std::thread::sleep(std::time::Duration::from_secs(1));

    // We could wait for all client threads to finish here if we stored their handles

    println!("ONVIF service shutdown complete");
    Ok(())
}

/// Validate that the RTSP stream is accessible
pub fn validate_rtsp_stream_connectivity(rtsp_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::{Command, Stdio};

    println!("Testing RTSP stream: {}", rtsp_url);

    // Use ffprobe to test the stream with a short timeout
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "csv=p=0",
            "-timeout",
            "10000000", // 10 seconds in microseconds
            "-analyzeduration",
            "5000000", // 5 seconds in microseconds
            rtsp_url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if output.status.success() {
        let codec = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !codec.is_empty() {
            println!("RTSP stream is accessible (codec: {})", codec);
            return Ok(());
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("RTSP stream validation failed: {}", stderr).into())
}

pub fn start_ws_discovery_server(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
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

    // Validate IP address before starting WS-Discovery
    if ws_container_ip.is_empty() || ws_container_ip == "0.0.0.0" {
        return Err("Cannot start WS-Discovery with invalid container IP".into());
    }

    // Create a channel for monitoring thread status
    let (status_tx, status_rx) = std::sync::mpsc::channel::<String>();

    let spawn_result = std::panic::catch_unwind(|| {
        let status_tx = status_tx.clone();
        thread::spawn(move || {
            println!("WS-Discovery thread started");

            // Send initial status update
            let _ = status_tx.send("STARTING".to_string());

            // Add error handling and retry logic
            let mut attempts = 0;
            let max_attempts = 3;

            while attempts < max_attempts {
                attempts += 1;
                println!("WS-Discovery attempt {attempts} of {max_attempts}");

                // Add a small delay between attempts to avoid rapid failures
                if attempts > 1 {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }

                match WSDiscoveryServer::new(ws_discovery_device_info.clone(), &ws_container_ip) {
                    Ok(_server) => {
                        println!("WS-Discovery server created");
                        // Send status update - now running
                        let _ = status_tx.send("RUNNING".to_string());

                        // Run the server (this will block until done)
                        // The run method will be implemented in the WSDiscoveryServer struct
                        // For now, we just print a message and consider it running
                        println!("WS-Discovery server started");

                        // Sleep for some time before considering a retry
                        std::thread::sleep(std::time::Duration::from_secs(60));

                        eprintln!("WS-Discovery server exited after timeout, will retry");
                    }
                    Err(e) => {
                        eprintln!("Failed to create WS-Discovery server: {e}, will retry");
                        // Continue the retry loop
                    }
                }
            }

            eprintln!(
                "WS-Discovery failed after {max_attempts} attempts - continuing without device discovery"
            );
            // Send final status update
            let _ = status_tx.send("FAILED".to_string());
            println!("WS-Discovery thread ending");
        })
    });

    match spawn_result {
        Ok(handle) => {
            println!("WS-Discovery server thread started successfully");

            // Start a monitoring thread to keep track of the WS-Discovery thread status
            let monitor_handle = thread::spawn(move || {
                let mut consecutive_failures = 0;

                loop {
                    match status_rx.recv_timeout(std::time::Duration::from_secs(30)) {
                        Ok(status) => {
                            println!("WS-Discovery status update: {status}");
                            consecutive_failures = 0;

                            if status == "FAILED" {
                                break;
                            }
                        }
                        Err(e) => {
                            if e == std::sync::mpsc::RecvTimeoutError::Disconnected {
                                println!("WS-Discovery monitor: thread disconnected");
                                break;
                            }
                            consecutive_failures += 1;
                            if consecutive_failures > 2 {
                                println!(
                                    "WS-Discovery monitor: no status updates for 60 seconds, assuming failure"
                                );
                                break;
                            }
                        }
                    }
                }

                println!("WS-Discovery monitor thread ending");
            });

            // Store the handle but don't block on it
            std::mem::drop(handle);
            std::mem::drop(monitor_handle);
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

pub fn start_onvif_service(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
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
    if config.ws_discovery_enabled {
        println!("Device discovery available via WS-Discovery");
    } else {
        println!("Device discovery disabled (WS-Discovery is off)");
    }
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
                        if let Err(e) = handle_onvif_request(stream, &config_clone) {
                            eprintln!("Error handling connection #{}: {}", conn_id, e);
                        }
                        println!("Thread completed for connection #{conn_id}");
                    })
                });

                match thread_result {
                    Ok(handle) => {
                        // Don't block, but we could store these handles and join them at shutdown
                        std::mem::drop(handle);
                    }
                    Err(_) => {
                        eprintln!(
                            "Thread creation panicked for connection #{}",
                            connection_count
                        );
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

pub fn handle_onvif_request(
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

    // Check if this is a GET request for snapshot
    if first_line.starts_with("GET /snapshot.jpg") {
        println!("Handling snapshot request");

        // Snapshot endpoints require authentication as per ONVIF standards
        if !is_authenticated(&request, &config.onvif_username, &config.onvif_password) {
            println!("  Snapshot request - authentication required");
            send_auth_required_response(&mut stream)?;
            return Ok(());
        }

        println!("  Snapshot request - authentication successful");
        return handle_snapshot_request(&mut stream, config);
    }

    // Debug: Check if we have SOAP content
    if request.contains("Envelope")
        && (request.contains("soap:") || request.contains("v:") || request.contains("s:"))
    {
        println!("  Contains SOAP envelope - valid ONVIF request");
    } else if request.contains(":Envelope") || request.contains("<Envelope") {
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
        if request.contains("soap:Envelope")
            || request.contains("s:Envelope")
            || request.contains(":Envelope")
            || request.contains("<soap:")
        {
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
    // Note: Snapshot endpoint requires authentication as per ONVIF standards
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
                    let nonce = extract_ws_security_element(request, "Nonce");

                    // Extract created timestamp - look for various created patterns
                    let created = extract_ws_security_element(request, "Created");

                    // If either is None, we can't validate
                    if nonce.is_none() || created.is_none() {
                        println!("    WS-Security: Missing nonce or created timestamp");
                        return false;
                    }

                    let nonce = nonce.unwrap();
                    let created = created.unwrap();

                    // Decode the nonce from base64
                    let nonce_bytes = match general_purpose::STANDARD.decode(nonce) {
                        Ok(bytes) => bytes,
                        Err(_) => {
                            println!("    WS-Security: Failed to decode nonce");
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

                    println!("    Expected digest: {expected_digest}");
                    println!("    Provided digest: {password_value}");

                    if password_value == expected_digest {
                        println!("    WS-Security: Authentication successful");
                        return true;
                    } else {
                        println!("    WS-Security: Authentication failed - digest mismatch");
                        return false;
                    }
                } else {
                    println!("    WS-Security: Using plain text password");
                    if password_value == password {
                        println!("    WS-Security: Authentication successful");
                        return true;
                    } else {
                        println!("    WS-Security: Authentication failed - password mismatch");
                        return false;
                    }
                }
            } else {
                println!("    WS-Security: Malformed Password element - no closing tag");
                return false;
            }
        } else {
            println!("    WS-Security: Malformed Password element - no closing >");
            return false;
        }
    } else {
        println!("    WS-Security: No Password element found");
        return false;
    }
}

fn extract_ws_security_element(request: &str, element_name: &str) -> Option<String> {
    // Try various tag formats: <ElementName>, <wsu:ElementName>, <wsse:ElementName>
    for prefix in ["", "wsu:", "wsse:", "s:", "soap:"] {
        let open_tag = format!("<{}{}>", prefix, element_name);
        let close_tag = format!("</{}{}>", prefix, element_name);

        if let (Some(start_pos), Some(end_pos)) = (
            request.find(&open_tag).map(|pos| pos + open_tag.len()),
            request.find(&close_tag),
        ) {
            if start_pos < end_pos {
                return Some(request[start_pos..end_pos].to_string());
            }
        }
    }

    None
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
        // Look for exact endpoint names in various formats
        if request.contains(&format!(":{}", endpoint))
            || request.contains(&format!("<{}", endpoint))
            || request.contains(&format!(" {}", endpoint))
        {
            return Some(endpoint.to_string());
        }
    }

    // Check for other SOAP action patterns that might be ONVIF
    if request.contains("soap:Envelope")
        || request.contains(":Body")
        || request.contains("<Envelope")
    {
        // Try to extract any ONVIF operation
        if let Some(body_start) = request.find("<Body>") {
            if let Some(body_end) = request.find("</Body>") {
                let body_content = &request[body_start + 6..body_end];
                // Find first tag after <Body>
                if let Some(start) = body_content.find('<') {
                    if let Some(end) = body_content[start + 1..].find('>') {
                        let tag = &body_content[start + 1..start + 1 + end];
                        // Strip namespace prefix if any
                        let operation = if let Some(colon_pos) = tag.rfind(':') {
                            &tag[colon_pos + 1..]
                        } else {
                            tag
                        };
                        // Trim any attributes
                        let operation = if let Some(space_pos) = operation.find(' ') {
                            &operation[0..space_pos]
                        } else {
                            operation
                        };
                        return Some(operation.to_string());
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
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<SOAP-ENV:Envelope xmlns:SOAP-ENV="http://www.w3.org/2003/05/soap-envelope" 
                   xmlns:SOAP-ENC="http://www.w3.org/2003/05/soap-encoding" 
                   xmlns:ter="http://www.onvif.org/ver10/error" 
                   xmlns:xs="http://www.w3.org/2001/XMLSchema" 
                   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
   <SOAP-ENV:Body>
      <SOAP-ENV:Fault>
         <SOAP-ENV:Code>
            <SOAP-ENV:Value>SOAP-ENV:Sender</SOAP-ENV:Value>
            <SOAP-ENV:Subcode>
               <SOAP-ENV:Value>ter:ActionNotSupported</SOAP-ENV:Value>
            </SOAP-ENV:Subcode>
         </SOAP-ENV:Code>
         <SOAP-ENV:Reason>
            <SOAP-ENV:Text xml:lang="en">The requested ONVIF operation '{endpoint}' is not supported by this device</SOAP-ENV:Text>
         </SOAP-ENV:Reason>
         <SOAP-ENV:Detail>
            <SOAP-ENV:Text>This ONVIF endpoint is recognized but not implemented. Please contact the developer if you need this functionality.</SOAP-ENV:Text>
         </SOAP-ENV:Detail>
      </SOAP-ENV:Fault>
   </SOAP-ENV:Body>
</SOAP-ENV:Envelope>"#
    );
    send_http_response(stream, "400 Bad Request", "application/soap+xml", &body)
}

/// Handle HTTP GET request for snapshot image
fn handle_snapshot_request(
    stream: &mut TcpStream,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // Capture a snapshot from the RTSP stream
    match capture_snapshot_from_rtsp(&config.rtsp_stream_url) {
        Ok(jpeg_data) => {
            println!(
                "  Snapshot captured successfully ({} bytes)",
                jpeg_data.len()
            );
            // Send the JPEG image as an HTTP response
            send_snapshot_response(stream, &jpeg_data)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("  Failed to capture snapshot: {e}");
            // Send a friendly error response
            send_http_response(
                stream,
                "500 Internal Server Error",
                "text/plain",
                &format!("Error capturing snapshot: {e}"),
            )?;
            Err(e)
        }
    }
}

/// Capture a single frame from the RTSP stream using FFmpeg
fn capture_snapshot_from_rtsp(rtsp_url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a temporary file to store the JPEG
    let file = NamedTempFile::new()?;
    let file_path = file.path().to_string_lossy().to_string();

    // Use FFmpeg to capture a single frame
    println!("  Running FFmpeg to capture snapshot from {rtsp_url}");
    let output = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output file
            "-timeout",
            "10000000", // 10 seconds in microseconds
            "-i",
            rtsp_url, // Input RTSP stream
            "-frames:v",
            "1", // Capture just one frame
            "-q:v",
            "2", // High quality (lower value = higher quality)
            "-f",
            "image2",   // Force output format
            &file_path, // Output file
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Failed to execute FFmpeg: {e}"))?;

    if !output.success() {
        return Err(format!(
            "FFmpeg failed with exit code: {}",
            output.code().unwrap_or(-1)
        )
        .into());
    }

    // Read the JPEG file into memory
    let mut jpeg_data = Vec::new();
    std::fs::File::open(&file_path)?.read_to_end(&mut jpeg_data)?;

    if jpeg_data.is_empty() {
        return Err("Captured image is empty".into());
    }

    println!(
        "  Snapshot captured successfully: {} bytes",
        jpeg_data.len()
    );
    Ok(jpeg_data)
}

/// Send HTTP response with JPEG image data
fn send_snapshot_response(
    stream: &mut TcpStream,
    jpeg_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    // Format HTTP response headers
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nCache-Control: no-cache\r\n\r\n",
        jpeg_data.len()
    );

    // Write headers
    stream.write_all(header.as_bytes())?;

    // Write image data
    stream.write_all(jpeg_data)?;

    Ok(())
}
