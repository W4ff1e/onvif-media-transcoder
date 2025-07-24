use onvif_media_transcoder::{
    setup_signal_handlers, start_onvif_service_with_shutdown, start_ws_discovery_server,
    validate_rtsp_stream_connectivity, Config, ServiceStatus,
};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

/// Main entry point for the ONVIF Media Transcoder
fn main() {
    println!("Starting ONVIF Media Transcoder...");

    // Create shared service status
    let service_status = Arc::new(ServiceStatus::new());

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
        // Do not exit immediately - let the main thread handle the error
    }));

    // Set up signal handlers for graceful shutdown
    let status_for_signal = service_status.clone();
    setup_signal_handlers(status_for_signal);

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

    // Validate RTSP stream connectivity before starting services
    println!("Validating RTSP stream connectivity...");
    match validate_rtsp_stream_connectivity(&config.rtsp_stream_url) {
        Ok(_) => {
            println!("RTSP stream connectivity validation passed");
        }
        Err(e) => {
            eprintln!("RTSP stream connectivity validation failed: {e}");
            eprintln!("The RTSP stream may not be ready yet. This could cause startup issues.");
            eprintln!(
                "Continuing anyway, but the service may not function properly until the stream is available."
            );
        }
    }

    // Create a channel for component-level shutdown coordination
    let (shutdown_tx, shutdown_rx) = channel::<String>();

    // Start WS-Discovery server in a separate thread
    if config.ws_discovery_enabled {
        println!("Initializing WS-Discovery server...");
        let ws_status = service_status.clone();

        if let Err(e) = start_ws_discovery_server(&config) {
            eprintln!("Failed to start WS-Discovery server: {e}");
            println!(
                "Continuing without WS-Discovery (ONVIF service will still work for direct connections)"
            );
            ws_status.set_error(&format!("WS-Discovery startup failed: {}", e));
        } else {
            println!("WS-Discovery server initialization completed");
            ws_status
                .ws_discovery_healthy
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    } else {
        println!("WS-Discovery is disabled - skipping WS-Discovery server initialization");
        println!(
            "Device discovery will not be available, but ONVIF service will work for direct connections"
        );
    }

    // Start ONVIF web service (this will block)
    println!("Starting ONVIF web service...");

    // Start a heartbeat thread to show the service is still alive
    let heartbeat_config = config.clone();
    let heartbeat_status = service_status.clone();
    let heartbeat_handle = thread::spawn(move || {
        let mut counter = 0;
        while !heartbeat_status.is_shutdown_requested() {
            std::thread::sleep(std::time::Duration::from_secs(30));
            counter += 1;
            println!(
                "HEARTBEAT #{}: ONVIF service is running (port: {})",
                counter, heartbeat_config.onvif_port
            );
        }
        println!("Heartbeat thread shutting down");
    });

    // Start the main ONVIF service with shutdown handling
    let onvif_service_status = service_status.clone();
    let onvif_config = config.clone();

    // Start the ONVIF service in the main thread, but monitor for shutdown signals
    let onvif_thread = thread::spawn(move || {
        if let Err(e) =
            start_onvif_service_with_shutdown(&onvif_config, onvif_service_status.clone())
        {
            eprintln!("ONVIF service error: {e}");
            onvif_service_status.set_error(&format!("ONVIF service error: {}", e));
            shutdown_tx
                .send("ONVIF service failed".to_string())
                .unwrap_or_default();
        } else if onvif_service_status.is_shutdown_requested() {
            println!("ONVIF service shutdown completed gracefully");
        } else {
            // If we get here, the service exited without an error but was not asked to shut down
            eprintln!("WARNING: ONVIF service exited unexpectedly");
            onvif_service_status.set_error("ONVIF service exited unexpectedly");
            shutdown_tx
                .send("ONVIF service exited unexpectedly".to_string())
                .unwrap_or_default();
        }
    });

    // Wait for shutdown signal from any component
    let main_status = service_status.clone();
    match shutdown_rx.recv() {
        Ok(component) => {
            println!("Shutdown initiated by component: {}", component);
            main_status.request_shutdown();
        }
        Err(_) => {
            println!("All component channels closed, initiating shutdown");
            main_status.request_shutdown();
        }
    }

    println!("Performing graceful shutdown...");

    // Request shutdown for all components
    service_status.request_shutdown();

    // Wait for heartbeat thread to finish
    if let Err(e) = heartbeat_handle.join() {
        eprintln!("Error joining heartbeat thread: {:?}", e);
    }

    // Wait for ONVIF service thread to finish with timeout
    match onvif_thread.join() {
        Ok(_) => println!("Main ONVIF service thread shut down successfully"),
        Err(e) => eprintln!("Error joining ONVIF service thread: {:?}", e),
    }

    println!("ONVIF Media Transcoder shutdown complete");
}
