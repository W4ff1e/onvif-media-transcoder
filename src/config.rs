use clap::Parser;
use std::net::IpAddr;

/// Configuration structure for the ONVIF Media Transcoder
#[derive(Debug, Clone, Parser)]
#[command(name = "onvif-media-transcoder")]
#[command(
    about = "ONVIF Media Transcoder - Converts media streams to ONVIF-compatible RTSP streams"
)]
pub struct Config {
    /// RTSP stream URL to transcode
    #[arg(short = 'r', long, default_value = "rtsp://127.0.0.1:8554/stream")]
    pub rtsp_stream_url: String,

    /// Port for the ONVIF service
    #[arg(short = 'P', long, default_value = "8080")]
    pub onvif_port: String,

    /// Device name for ONVIF identification
    #[arg(short = 'n', long, default_value = "ONVIF-Media-Transcoder")]
    pub device_name: String,

    /// Username for ONVIF authentication
    #[arg(short = 'u', long, default_value = "admin")]
    pub onvif_username: String,

    /// Password for ONVIF authentication
    #[arg(short = 'p', long, default_value = "onvif-rust")]
    pub onvif_password: String,

    /// Container IP address for WS-Discovery
    #[arg(long = "container-ip", short = 'i', default_value = "127.0.0.1")]
    pub container_ip: String,

    /// Enable WS-Discovery service for automatic device discovery
    #[arg(long = "ws-discovery-enabled", short = 'w', action = clap::ArgAction::SetTrue)]
    pub ws_discovery_enabled: bool,

    /// Enable debug mode with verbose request logging (NOT FOR PRODUCTION USE, LOGS SENSITIVE INFORMATION)
    #[arg(short = 'd', long = "debug", action = clap::ArgAction::SetTrue)]
    pub debug: bool,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
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
        if config.container_ip.parse::<IpAddr>().is_err() {
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
            "  WS-Discovery: {}",
            if self.ws_discovery_enabled {
                "ENABLED"
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
