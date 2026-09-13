use onvif_media_transcoder::config::Config;
use onvif_media_transcoder::onvif::OnvifService;
use onvif_media_transcoder::ws_discovery::{DeviceInfo, WSDiscoveryServer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
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

    // Set on SIGTERM/SIGINT; every long-running loop polls it so that the
    // WS-Discovery Bye is sent and sockets are closed before exit.
    let shutdown = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
        if let Err(e) = signal_hook::flag::register(signal, Arc::clone(&shutdown)) {
            error!(error = %e, signal, "failed to install signal handler");
            std::process::exit(1);
        }
    }

    let ws_discovery = if config.ws_discovery_enabled {
        match start_ws_discovery(&config, Arc::clone(&shutdown)) {
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
            shutdown.store(true, Ordering::Relaxed);
            if let Some(handle) = ws_discovery {
                let _ = handle.join();
            }
            std::process::exit(1);
        }
    };
    info!(bind = %bind_addr, "ONVIF HTTP service ready");

    while !shutdown.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(200));
    }

    info!("shutdown requested");
    server.shutdown();
    if let Some(handle) = ws_discovery {
        let _ = handle.join();
    }
    info!("stopped");
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
    shutdown: Arc<AtomicBool>,
) -> Result<thread::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
    let interface: std::net::Ipv4Addr = config.container_ip.parse().map_err(|_| {
        format!(
            "CONTAINER_IP must be an IPv4 address for WS-Discovery, got '{}'",
            config.container_ip
        )
    })?;

    let device_info = DeviceInfo {
        endpoint_reference: format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        scopes: format!(
            "onvif://www.onvif.org/type/NetworkVideoTransmitter onvif://www.onvif.org/Profile/Streaming onvif://www.onvif.org/name/{name} onvif://www.onvif.org/hardware/{name} onvif://www.onvif.org/location/Unknown",
            name = config.device_name
        ),
        xaddrs: format!(
            "http://{}:{}/onvif/device_service",
            config.container_ip, config.onvif_port
        ),
    };

    let server = WSDiscoveryServer::new(device_info, interface)?;
    let handle = thread::Builder::new()
        .name("ws-discovery".to_string())
        .spawn(move || server.run(&shutdown))?;
    Ok(handle)
}
