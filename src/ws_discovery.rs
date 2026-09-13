//! WS-Discovery responder for ONVIF device discovery.
//!
//! Listens on the well-known multicast group, answers `Probe` messages with
//! `ProbeMatches`, announces itself with `Hello` and says goodbye with `Bye`
//! on shutdown. Incoming messages are parsed as XML and classified by the
//! `wsa:Action` header, so the responder never reacts to its own or other
//! devices' announcements.

use crate::onvif::soap::xml_escape;
use roxmltree::Document;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, trace, warn};
use uuid::Uuid;

/// WS-Discovery multicast group.
pub const WS_DISCOVERY_MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
/// WS-Discovery UDP port.
pub const WS_DISCOVERY_PORT: u16 = 3702;
/// WS-Discovery (April 2005) namespace used by ONVIF.
pub const WS_DISCOVERY_NAMESPACE: &str = "http://schemas.xmlsoap.org/ws/2005/04/discovery";
/// WS-Discovery 1.1 (OASIS) namespace, accepted on input.
pub const WS_DISCOVERY_11_NAMESPACE: &str = "http://docs.oasis-open.org/ws-dd/ns/discovery/2009/01";
/// WS-Addressing namespace.
pub const WS_ADDRESSING_NAMESPACE: &str = "http://www.w3.org/2005/08/addressing";
/// ONVIF network device type namespace.
pub const ONVIF_NETWORK_NAMESPACE: &str = "http://www.onvif.org/ver10/network/wsdl";
/// Interval between unsolicited Hello announcements.
pub const HELLO_INTERVAL: Duration = Duration::from_secs(60);

/// Device information advertised through WS-Discovery.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Stable endpoint reference, e.g. `urn:uuid:...`.
    pub endpoint_reference: String,
    /// Space separated list of ONVIF scope URIs.
    pub scopes: String,
    /// Space separated list of service addresses.
    pub xaddrs: String,
}

/// A classified incoming WS-Discovery message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomingMessage {
    /// A Probe we should answer, with the sender's MessageID if present.
    Probe {
        message_id: Option<String>,
        types: Vec<String>,
    },
    /// Any other WS-Discovery or unrelated message.
    Other(String),
}

/// WS-Discovery responder bound to the multicast group.
pub struct WSDiscoveryServer {
    device_info: DeviceInfo,
    socket: UdpSocket,
    instance_id: u64,
    message_number: AtomicU32,
}

impl WSDiscoveryServer {
    /// Binds the discovery socket on all interfaces and joins the multicast
    /// group on `interface`.
    pub fn new(
        device_info: DeviceInfo,
        interface: Ipv4Addr,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        // Other discovery responders (Windows, other cameras in host network
        // mode) may already own port 3702; share it instead of failing.
        socket.set_reuse_address(true)?;
        socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, WS_DISCOVERY_PORT).into())?;
        socket.join_multicast_v4(&WS_DISCOVERY_MULTICAST_GROUP, &interface)?;
        socket.set_multicast_if_v4(&interface)?;
        // Never receive our own announcements.
        socket.set_multicast_loop_v4(false)?;
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;
        let socket: UdpSocket = socket.into();

        let instance_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1);

        info!(%interface, port = WS_DISCOVERY_PORT, "WS-Discovery bound and joined multicast group");

        Ok(Self {
            device_info,
            socket,
            instance_id,
            message_number: AtomicU32::new(0),
        })
    }

    /// Runs the responder until `stop` is set, then sends Bye.
    pub fn run(&self, stop: &AtomicBool) {
        if let Err(e) = self.send_hello() {
            warn!(error = %e, "failed to send initial Hello");
        }

        let mut buffer = [0u8; 8192];
        let mut last_hello = Instant::now();

        while !stop.load(Ordering::Relaxed) {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    let message = String::from_utf8_lossy(&buffer[..size]);
                    self.handle_message(&message, src);
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    warn!(error = %e, "error receiving WS-Discovery message");
                    std::thread::sleep(Duration::from_millis(200));
                }
            }

            if last_hello.elapsed() >= HELLO_INTERVAL {
                if let Err(e) = self.send_hello() {
                    warn!(error = %e, "failed to send periodic Hello");
                }
                last_hello = Instant::now();
            }
        }

        if let Err(e) = self.send_bye() {
            warn!(error = %e, "failed to send Bye");
        }
    }

    fn handle_message(&self, message: &str, src: SocketAddr) {
        match classify_message(message) {
            IncomingMessage::Probe { message_id, types } => {
                if !types_match(&types) {
                    trace!(%src, ?types, "ignoring Probe for other device types");
                    return;
                }
                debug!(%src, "answering Probe");
                if let Err(e) = self.send_probe_match(src, message_id.as_deref()) {
                    warn!(%src, error = %e, "failed to send ProbeMatch");
                }
            }
            IncomingMessage::Other(action) => {
                trace!(%src, action, "ignoring non-Probe message");
            }
        }
    }

    fn next_message_number(&self) -> u32 {
        self.message_number.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn multicast_target(&self) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(
            WS_DISCOVERY_MULTICAST_GROUP,
            WS_DISCOVERY_PORT,
        ))
    }

    fn send_hello(&self) -> std::io::Result<()> {
        let message = build_hello(
            &self.device_info,
            self.instance_id,
            self.next_message_number(),
        );
        self.socket
            .send_to(message.as_bytes(), self.multicast_target())?;
        debug!("sent Hello");
        Ok(())
    }

    /// Announces that this device is leaving the network.
    pub fn send_bye(&self) -> std::io::Result<()> {
        let message = build_bye(
            &self.device_info,
            self.instance_id,
            self.next_message_number(),
        );
        self.socket
            .send_to(message.as_bytes(), self.multicast_target())?;
        info!("sent Bye");
        Ok(())
    }

    fn send_probe_match(&self, dest: SocketAddr, relates_to: Option<&str>) -> std::io::Result<()> {
        let message = build_probe_match(
            &self.device_info,
            self.instance_id,
            self.next_message_number(),
            relates_to,
        );
        self.socket.send_to(message.as_bytes(), dest)?;
        Ok(())
    }
}

/// Returns true if the probe asked for no particular type or for an ONVIF
/// network video transmitter / device.
fn types_match(types: &[String]) -> bool {
    types.is_empty()
        || types.iter().any(|t| {
            let local = t.rsplit(':').next().unwrap_or(t);
            local == "NetworkVideoTransmitter" || local == "Device"
        })
}

/// Classifies an incoming message by its WS-Addressing Action header.
pub fn classify_message(message: &str) -> IncomingMessage {
    let Ok(doc) = Document::parse(message) else {
        return IncomingMessage::Other("<malformed>".to_string());
    };

    let action = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "Action")
        .and_then(|n| n.text())
        .map(str::trim)
        .unwrap_or("");

    let is_probe = action.ends_with("/Probe")
        && (action.starts_with(WS_DISCOVERY_NAMESPACE)
            || action.starts_with(WS_DISCOVERY_11_NAMESPACE));
    if !is_probe {
        return IncomingMessage::Other(action.to_string());
    }

    let message_id = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "MessageID")
        .and_then(|n| n.text())
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());

    let types = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "Probe")
        .and_then(|probe| {
            probe
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "Types")
        })
        .and_then(|n| n.text())
        .map(|t| t.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default();

    IncomingMessage::Probe { message_id, types }
}

fn envelope(
    action: &str,
    relates_to: Option<&str>,
    to: &str,
    instance_id: u64,
    message_number: u32,
    body: &str,
) -> String {
    let relates_to = relates_to
        .map(|r| format!("\n<wsa:RelatesTo>{}</wsa:RelatesTo>", xml_escape(r)))
        .unwrap_or_default();
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<soap:Envelope xmlns:soap=\"http://www.w3.org/2003/05/soap-envelope\" xmlns:wsa=\"{WS_ADDRESSING_NAMESPACE}\" xmlns:wsd=\"{WS_DISCOVERY_NAMESPACE}\" xmlns:dn=\"{ONVIF_NETWORK_NAMESPACE}\">\n<soap:Header>\n<wsa:Action>{WS_DISCOVERY_NAMESPACE}/{action}</wsa:Action>\n<wsa:MessageID>urn:uuid:{}</wsa:MessageID>{relates_to}\n<wsa:To>{to}</wsa:To>\n<wsd:AppSequence InstanceId=\"{instance_id}\" MessageNumber=\"{message_number}\"/>\n</soap:Header>\n<soap:Body>\n{body}\n</soap:Body>\n</soap:Envelope>",
        Uuid::new_v4()
    )
}

fn endpoint_block(device_info: &DeviceInfo) -> String {
    format!(
        "<wsa:EndpointReference>\n<wsa:Address>{}</wsa:Address>\n</wsa:EndpointReference>\n<wsd:Types>dn:NetworkVideoTransmitter</wsd:Types>\n<wsd:Scopes>{}</wsd:Scopes>\n<wsd:XAddrs>{}</wsd:XAddrs>\n<wsd:MetadataVersion>1</wsd:MetadataVersion>",
        xml_escape(&device_info.endpoint_reference),
        xml_escape(&device_info.scopes),
        xml_escape(&device_info.xaddrs)
    )
}

const DISCOVERY_TO: &str = "urn:schemas-xmlsoap-org:ws:2005:04:discovery";
const ANONYMOUS_TO: &str = "http://www.w3.org/2005/08/addressing/anonymous";

/// Builds a multicast Hello announcement.
pub fn build_hello(device_info: &DeviceInfo, instance_id: u64, message_number: u32) -> String {
    let body = format!("<wsd:Hello>\n{}\n</wsd:Hello>", endpoint_block(device_info));
    envelope(
        "Hello",
        None,
        DISCOVERY_TO,
        instance_id,
        message_number,
        &body,
    )
}

/// Builds a multicast Bye announcement.
pub fn build_bye(device_info: &DeviceInfo, instance_id: u64, message_number: u32) -> String {
    let body = format!(
        "<wsd:Bye>\n<wsa:EndpointReference>\n<wsa:Address>{}</wsa:Address>\n</wsa:EndpointReference>\n</wsd:Bye>",
        xml_escape(&device_info.endpoint_reference)
    );
    envelope(
        "Bye",
        None,
        DISCOVERY_TO,
        instance_id,
        message_number,
        &body,
    )
}

/// Builds a unicast ProbeMatches reply. `relates_to` must be the probe's
/// MessageID exactly as received.
pub fn build_probe_match(
    device_info: &DeviceInfo,
    instance_id: u64,
    message_number: u32,
    relates_to: Option<&str>,
) -> String {
    let body = format!(
        "<wsd:ProbeMatches>\n<wsd:ProbeMatch>\n{}\n</wsd:ProbeMatch>\n</wsd:ProbeMatches>",
        endpoint_block(device_info)
    );
    envelope(
        "ProbeMatches",
        relates_to,
        ANONYMOUS_TO,
        instance_id,
        message_number,
        &body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> DeviceInfo {
        DeviceInfo {
            endpoint_reference: "urn:uuid:11111111-2222-3333-4444-555555555555".to_string(),
            scopes: "onvif://www.onvif.org/type/NetworkVideoTransmitter onvif://www.onvif.org/name/Test%20Cam".to_string(),
            xaddrs: "http://192.0.2.10:8080/onvif/device_service".to_string(),
        }
    }

    const PROBE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<e:Envelope xmlns:e="http://www.w3.org/2003/05/soap-envelope" xmlns:w="http://schemas.xmlsoap.org/ws/2005/08/addressing" xmlns:d="http://schemas.xmlsoap.org/ws/2005/04/discovery" xmlns:dn="http://www.onvif.org/ver10/network/wsdl">
<e:Header><w:MessageID>uuid:84ede3de-7dec-11d0-c360-f01234567890</w:MessageID><w:To e:mustUnderstand="true">urn:schemas-xmlsoap-org:ws:2005:04:discovery</w:To><w:Action a:mustUnderstand="true" xmlns:a="http://www.w3.org/2003/05/soap-envelope">http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</w:Action></e:Header>
<e:Body><d:Probe><d:Types>dn:NetworkVideoTransmitter</d:Types></d:Probe></e:Body>
</e:Envelope>"#;

    #[test]
    fn classifies_probe_and_keeps_message_id_verbatim() {
        match classify_message(PROBE) {
            IncomingMessage::Probe { message_id, types } => {
                assert_eq!(
                    message_id.as_deref(),
                    Some("uuid:84ede3de-7dec-11d0-c360-f01234567890")
                );
                assert_eq!(types, vec!["dn:NetworkVideoTransmitter".to_string()]);
                assert!(types_match(&types));
            }
            other => panic!("expected probe, got {other:?}"),
        }
    }

    #[test]
    fn own_hello_and_probe_match_are_not_probes() {
        let d = device();
        for msg in [
            build_hello(&d, 1, 1),
            build_bye(&d, 1, 2),
            build_probe_match(&d, 1, 3, Some("urn:uuid:x")),
        ] {
            assert!(
                matches!(classify_message(&msg), IncomingMessage::Other(_)),
                "{msg}"
            );
        }
        assert!(matches!(
            classify_message("garbage"),
            IncomingMessage::Other(_)
        ));
        // Mentions of ONVIF strings without a Probe action are ignored.
        let hello_like = r#"<e:Envelope xmlns:e="http://www.w3.org/2003/05/soap-envelope"><e:Header><Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/Hello</Action></e:Header><e:Body><Hello><Types>tdn:NetworkVideoTransmitter</Types><Scopes>onvif://www.onvif.org/name/x</Scopes></Hello></e:Body></e:Envelope>"#;
        assert!(matches!(
            classify_message(hello_like),
            IncomingMessage::Other(_)
        ));
    }

    #[test]
    fn probe_without_types_or_with_device_type_matches() {
        let no_types = r#"<e:Envelope xmlns:e="http://www.w3.org/2003/05/soap-envelope"><e:Header><Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</Action></e:Header><e:Body><Probe/></e:Body></e:Envelope>"#;
        match classify_message(no_types) {
            IncomingMessage::Probe { message_id, types } => {
                assert!(message_id.is_none());
                assert!(types.is_empty());
                assert!(types_match(&types));
            }
            other => panic!("{other:?}"),
        }
        assert!(types_match(&["tds:Device".to_string()]));
        assert!(!types_match(&["wsdp:Printer".to_string()]));
    }

    #[test]
    fn probe_match_relates_to_original_message_id() {
        let msg = build_probe_match(&device(), 7, 3, Some("uuid:abc-123"));
        let doc = Document::parse(&msg).expect("well-formed");
        let relates = doc
            .descendants()
            .find(|n| n.tag_name().name() == "RelatesTo")
            .and_then(|n| n.text())
            .unwrap();
        assert_eq!(relates, "uuid:abc-123");
        let app_seq = doc
            .descendants()
            .find(|n| n.tag_name().name() == "AppSequence")
            .unwrap();
        assert_eq!(app_seq.attribute("InstanceId"), Some("7"));
        assert_eq!(app_seq.attribute("MessageNumber"), Some("3"));
        assert!(msg.contains("ProbeMatches"));
        assert!(msg.contains("http://192.0.2.10:8080/onvif/device_service"));
    }

    #[test]
    fn announcements_are_well_formed_and_escaped() {
        let mut d = device();
        d.scopes.push_str(" onvif://www.onvif.org/hardware/A&B");
        for msg in [build_hello(&d, 1, 1), build_bye(&d, 1, 2)] {
            let doc = Document::parse(&msg).unwrap_or_else(|e| panic!("{e}: {msg}"));
            assert!(doc.descendants().any(|n| n.tag_name().name() == "Action"));
        }
        assert!(build_hello(&d, 1, 1).contains("A&amp;B"));
    }
}
