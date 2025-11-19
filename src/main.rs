use onvif_media_transcoder::config::Config;
use onvif_media_transcoder::onvif::handle_onvif_request;
use onvif_media_transcoder::ws_discovery::{DeviceInfo, WSDiscoveryServer};
use std::net::TcpListener;
use std::thread;

fn main() {
    println!("Starting ONVIF Media Transcoder...");

    // Load configuration
    let config = match Config::load() {
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

    // Start WS-Discovery if enabled
    if config.ws_discovery_enabled {
        println!("WS-Discovery is enabled - starting discovery service alongside ONVIF...");

        // Start both WS-Discovery and ONVIF services concurrently
        if let Err(e) = start_services_with_ws_discovery(&config) {
            eprintln!("Service startup error: {e}");
            std::process::exit(1);
        }
    } else {
        println!("WS-Discovery disabled - continuing with direct ONVIF connections only");

        // Start ONVIF web service only (this will block)
        println!("Starting ONVIF web service...");
        if let Err(e) = start_onvif_service(&config) {
            eprintln!("ONVIF service error: {e}");
            std::process::exit(1);
        }
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

fn start_services_with_ws_discovery(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting services with WS-Discovery enabled...");

    // Create device info for WS-Discovery
    let device_info = DeviceInfo {
        endpoint_reference: format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        types: "tdn:NetworkVideoTransmitter".to_string(),
        scopes: format!(
            "onvif://www.onvif.org/type/NetworkVideoTransmitter onvif://www.onvif.org/name/{} onvif://www.onvif.org/hardware/{} onvif://www.onvif.org/location/Unknown",
            config.device_name,
            config.device_name
        ),
        xaddrs: format!("http://{}:{}/onvif/device_service", config.container_ip, config.onvif_port),
        manufacturer: "ONVIF Media Solutions".to_string(),
        model_name: config.device_name.clone(),
        friendly_name: config.device_name.clone(),
        firmware_version: "1.0.0".to_string(),
        serial_number: format!("EMU-{}", config.device_name.chars().take(6).collect::<String>()),
    };

    // Start WS-Discovery server
    println!("Creating WS-Discovery server...");
    let mut ws_discovery_server =
        WSDiscoveryServer::new(device_info, &config.container_ip, config.debug)?;

    let config_clone = config.clone();
    let onvif_handle = thread::spawn(move || {
        println!("Starting ONVIF service thread...");
        if let Err(e) = start_onvif_service(&config_clone) {
            eprintln!("ONVIF service error: {e}");
        }
    });

    let ws_handle = thread::spawn(move || {
        println!("Starting WS-Discovery service thread...");
        if let Err(e) = ws_discovery_server.start() {
            eprintln!("WS-Discovery service error: {e}");
        }
    });

    println!("Both services started successfully!");
    println!("WS-Discovery: Listening on {}:3702", config.container_ip);
    println!(
        "ONVIF HTTP: Listening on {}:{}",
        config.container_ip, config.onvif_port
    );

    // Wait for both threads to complete (they should run indefinitely)
    if let Err(e) = onvif_handle.join() {
        eprintln!("ONVIF thread panicked: {e:?}");
    }
    if let Err(e) = ws_handle.join() {
        eprintln!("WS-Discovery thread panicked: {e:?}");
    }

    Ok(())
}
