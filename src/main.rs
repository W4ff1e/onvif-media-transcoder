use onvif_media_transcoder::config::Config;
use onvif_media_transcoder::onvif::OnvifService;
use onvif_media_transcoder::ws_discovery::{DeviceInfo, WSDiscoveryServer};
use std::sync::Arc;
use std::thread;
use tracing::{error, info};

/// Number of threads serving ONVIF HTTP requests.
const HTTP_WORKERS: usize = 4;

fn main() {
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {e}");
            std::process::exit(1);
        }
    };

    init_logging(config.debug);
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "starting ONVIF Media Transcoder"
    );
    config.display();

    let ws_discovery = if config.ws_discovery_enabled {
        match start_ws_discovery(&config) {
            Ok(handle) => Some(handle),
            Err(e) => {
                error!(error = %e, "failed to start WS-Discovery");
                std::process::exit(1);
            }
        }
    } else {
        info!("WS-Discovery disabled");
        None
    };

    let bind_addr = format!("0.0.0.0:{}", config.onvif_port);
    info!(
        bind = %bind_addr,
        stream = %config.rtsp_stream_url,
        device = %config.device_name,
        "starting ONVIF HTTP service"
    );
    let service = Arc::new(OnvifService::new(config));
    let server = match service.serve(&bind_addr, HTTP_WORKERS) {
        Ok(server) => server,
        Err(e) => {
            error!(error = %e, bind = %bind_addr, "failed to bind ONVIF port");
            std::process::exit(1);
        }
    };
    info!(bind = %bind_addr, "ONVIF HTTP service ready");

    // The HTTP workers run until the process is terminated.
    server.join();
    error!("ONVIF HTTP service stopped unexpectedly");
    drop(ws_discovery);
    std::process::exit(1);
}

fn init_logging(debug: bool) {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(if debug { "debug" } else { "info" }));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn start_ws_discovery(
    config: &Config,
) -> Result<thread::JoinHandle<()>, Box<dyn std::error::Error>> {
    let device_info = DeviceInfo {
        endpoint_reference: format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        types: "tdn:NetworkVideoTransmitter".to_string(),
        scopes: format!(
            "onvif://www.onvif.org/type/NetworkVideoTransmitter onvif://www.onvif.org/name/{} onvif://www.onvif.org/hardware/{} onvif://www.onvif.org/location/Unknown",
            config.device_name, config.device_name
        ),
        xaddrs: format!(
            "http://{}:{}/onvif/device_service",
            config.container_ip, config.onvif_port
        ),
        manufacturer: "ONVIF Media Solutions".to_string(),
        model_name: config.device_name.clone(),
        friendly_name: config.device_name.clone(),
        firmware_version: env!("CARGO_PKG_VERSION").to_string(),
        serial_number: format!(
            "EMU-{}",
            config.device_name.chars().take(6).collect::<String>()
        ),
    };

    let mut server = WSDiscoveryServer::new(device_info, &config.container_ip, config.debug)?;
    let handle = thread::Builder::new()
        .name("ws-discovery".to_string())
        .spawn(move || {
            if let Err(e) = server.start() {
                error!(error = %e, "WS-Discovery service stopped");
            }
        })?;
    Ok(handle)
}
