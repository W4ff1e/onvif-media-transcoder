//! The identity this device presents to ONVIF clients, shared by
//! WS-Discovery announcements and the GetDeviceInformation response.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use std::net::Ipv4Addr;
use uuid::Uuid;

/// Characters that must be escaped inside an ONVIF scope path segment.
const SCOPE_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

/// Manufacturer reported to clients.
pub const MANUFACTURER: &str = "ONVIF Media Transcoder";
/// Hardware identifier reported to clients.
pub const HARDWARE_ID: &str = "onvif-media-transcoder";

/// Static device description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIdentity {
    /// Human readable device name; also used as the ONVIF model.
    pub name: String,
    /// Stable endpoint reference derived from the device name.
    pub endpoint_reference: String,
    /// Firmware version, taken from the crate version.
    pub firmware_version: String,
    /// Serial number derived from the device name.
    pub serial_number: String,
}

impl DeviceIdentity {
    pub fn new(name: &str) -> Self {
        // A stable UUID lets clients recognise the same device across
        // restarts instead of adopting a "new" camera every time.
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes());
        let serial: String = uuid.simple().to_string()[..12].to_ascii_uppercase();
        Self {
            name: name.to_string(),
            endpoint_reference: format!("urn:uuid:{uuid}"),
            firmware_version: env!("CARGO_PKG_VERSION").to_string(),
            serial_number: format!("OMT-{serial}"),
        }
    }

    pub fn manufacturer(&self) -> &'static str {
        MANUFACTURER
    }

    pub fn hardware_id(&self) -> &'static str {
        HARDWARE_ID
    }

    /// Space separated WS-Discovery scopes.
    pub fn scopes(&self) -> String {
        let name = utf8_percent_encode(&self.name, SCOPE_SEGMENT);
        format!(
            "onvif://www.onvif.org/type/NetworkVideoTransmitter onvif://www.onvif.org/type/video_encoder onvif://www.onvif.org/Profile/Streaming onvif://www.onvif.org/name/{name} onvif://www.onvif.org/hardware/{HARDWARE_ID} onvif://www.onvif.org/location/Unknown"
        )
    }

    /// Device service address for the given reachable IP and port.
    pub fn device_service_url(ip: Ipv4Addr, port: u16) -> String {
        format!("http://{ip}:{port}/onvif/device_service")
    }

    /// Media service address for the given reachable IP and port.
    pub fn media_service_url(ip: Ipv4Addr, port: u16) -> String {
        format!("http://{ip}:{port}/onvif/media_service")
    }

    /// Snapshot URL for the given reachable IP and port.
    pub fn snapshot_url(ip: Ipv4Addr, port: u16) -> String {
        format!("http://{ip}:{port}/snapshot.jpg")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_stable_and_derived_from_name() {
        let a = DeviceIdentity::new("Front Door");
        let b = DeviceIdentity::new("Front Door");
        let c = DeviceIdentity::new("Back Door");
        assert_eq!(a, b);
        assert_ne!(a.endpoint_reference, c.endpoint_reference);
        assert!(a.endpoint_reference.starts_with("urn:uuid:"));
        assert!(a.serial_number.starts_with("OMT-"));
        assert_eq!(a.firmware_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn scopes_are_percent_encoded() {
        let id = DeviceIdentity::new("Front Door & Yard");
        let scopes = id.scopes();
        assert!(scopes.contains("onvif://www.onvif.org/name/Front%20Door%20%26%20Yard"));
        assert!(!scopes.contains("Front Door"));
        assert!(scopes.contains("onvif://www.onvif.org/Profile/Streaming"));
    }

    #[test]
    fn service_urls() {
        let ip = Ipv4Addr::new(192, 0, 2, 1);
        assert_eq!(
            DeviceIdentity::device_service_url(ip, 8080),
            "http://192.0.2.1:8080/onvif/device_service"
        );
        assert_eq!(
            DeviceIdentity::snapshot_url(ip, 80),
            "http://192.0.2.1:80/snapshot.jpg"
        );
    }
}
