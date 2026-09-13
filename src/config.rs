//! Runtime configuration.
//!
//! Every option can be given as a command-line flag or as an environment
//! variable, so the container entrypoint only needs to export variables and
//! local development can use flags.

use clap::builder::BoolishValueParser;
use clap::{ArgAction, Parser};
use std::net::Ipv4Addr;
use tracing::info;

/// Configuration for the ONVIF Media Transcoder.
#[derive(Debug, Clone, Parser)]
#[command(name = "onvif-media-transcoder", version)]
#[command(about = "Exposes an RTSP stream as an ONVIF Profile S camera with WS-Discovery")]
pub struct Config {
    /// RTSP URL of the stream served by MediaMTX that clients will play
    #[arg(
        short = 'r',
        long,
        env = "RTSP_STREAM_URL",
        default_value = "rtsp://127.0.0.1:8554/stream"
    )]
    pub rtsp_stream_url: String,

    /// TCP port for the ONVIF HTTP service
    #[arg(short = 'P', long, env = "ONVIF_PORT", default_value_t = 8080)]
    pub onvif_port: u16,

    /// Device name reported to ONVIF clients and used in discovery scopes
    #[arg(
        short = 'n',
        long,
        env = "DEVICE_NAME",
        default_value = "ONVIF-Media-Transcoder"
    )]
    pub device_name: String,

    /// Username for ONVIF authentication
    #[arg(short = 'u', long, env = "ONVIF_USERNAME", default_value = "admin")]
    pub onvif_username: String,

    /// Password for ONVIF authentication
    #[arg(
        short = 'p',
        long,
        env = "ONVIF_PASSWORD",
        default_value = "onvif-rust",
        hide_env_values = true
    )]
    pub onvif_password: String,

    /// IPv4 address clients use to reach this device; advertised in
    /// discovery and service addresses
    #[arg(
        short = 'i',
        long,
        env = "CONTAINER_IP",
        default_value_t = Ipv4Addr::LOCALHOST
    )]
    pub container_ip: Ipv4Addr,

    /// Enable the WS-Discovery responder (true/false, yes/no, 1/0)
    #[arg(
        short = 'w',
        long,
        env = "WS_DISCOVERY_ENABLED",
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        value_parser = BoolishValueParser::new()
    )]
    pub ws_discovery_enabled: bool,

    /// Enable debug logging. Logs full requests, including credentials.
    #[arg(
        short = 'd',
        long,
        env = "DEBUG_LOGGING",
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        value_parser = BoolishValueParser::new()
    )]
    pub debug: bool,
}

impl Config {
    /// Parses flags and environment variables and validates them.
    pub fn load() -> Result<Self, String> {
        let config = Config::parse();
        config.validate()?;
        Ok(config)
    }

    /// Checks constraints that clap cannot express.
    pub fn validate(&self) -> Result<(), String> {
        if !self.rtsp_stream_url.starts_with("rtsp://") {
            return Err(format!(
                "RTSP_STREAM_URL must start with 'rtsp://', got '{}'",
                self.rtsp_stream_url
            ));
        }
        if self.device_name.trim().is_empty() {
            return Err("DEVICE_NAME must not be empty".to_string());
        }
        if self.onvif_username.is_empty() || self.onvif_password.is_empty() {
            return Err("ONVIF_USERNAME and ONVIF_PASSWORD must not be empty".to_string());
        }
        Ok(())
    }

    /// Logs the effective configuration without the password.
    pub fn display(&self) {
        info!(
            rtsp_stream_url = %self.rtsp_stream_url,
            onvif_port = self.onvif_port,
            device_name = %self.device_name,
            onvif_username = %self.onvif_username,
            container_ip = %self.container_ip,
            ws_discovery_enabled = self.ws_discovery_enabled,
            debug = self.debug,
            "configuration"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Config, clap::Error> {
        let mut full = vec!["onvif-media-transcoder"];
        full.extend_from_slice(args);
        Config::try_parse_from(full)
    }

    #[test]
    fn defaults() {
        let c = parse(&[]).unwrap();
        assert_eq!(c.onvif_port, 8080);
        assert_eq!(c.container_ip, Ipv4Addr::LOCALHOST);
        assert!(!c.ws_discovery_enabled);
        assert!(!c.debug);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn flags_and_boolish_values() {
        let c = parse(&["-w", "-d", "-P", "9000", "-i", "192.0.2.5"]).unwrap();
        assert!(c.ws_discovery_enabled);
        assert!(c.debug);
        assert_eq!(c.onvif_port, 9000);
        assert_eq!(c.container_ip, Ipv4Addr::new(192, 0, 2, 5));

        let c = parse(&["--ws-discovery-enabled", "no", "--debug=yes"]).unwrap();
        assert!(!c.ws_discovery_enabled);
        assert!(c.debug);
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(parse(&["-P", "70000"]).is_err());
        assert!(parse(&["-P", "abc"]).is_err());
        assert!(parse(&["-i", "fd00::1"]).is_err());
        assert!(parse(&["-i", "not-an-ip"]).is_err());
        assert!(parse(&["-w", "maybe"]).is_err());

        let c = parse(&["-r", "http://x/stream"]).unwrap();
        assert!(c.validate().is_err());
        let c = parse(&["-n", "  "]).unwrap();
        assert!(c.validate().is_err());
    }
}
