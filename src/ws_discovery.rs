use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use uuid::Uuid;

/// WS-Discovery multicast address and port
const WS_DISCOVERY_MULTICAST_ADDR: &str = "239.255.255.250:3702";
/// WS-Discovery namespace URI
const WS_DISCOVERY_NAMESPACE: &str = "http://schemas.xmlsoap.org/ws/2005/04/discovery";
/// WS-Addressing namespace URI
const WS_ADDRESSING_NAMESPACE: &str = "http://www.w3.org/2005/08/addressing";

/// Device information for WS-Discovery announcements and responses
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Unique endpoint reference for the device
    pub endpoint_reference: String,
    /// Device types (e.g., "tdn:NetworkVideoTransmitter")
    pub types: String,
    /// Device scopes for discovery filtering
    pub scopes: String,
    /// ONVIF service addresses (XAddrs)
    pub xaddrs: String,
    /// Device manufacturer name
    #[allow(dead_code)]
    pub manufacturer: String,
    /// Device model name
    #[allow(dead_code)]
    pub model_name: String,
    /// Human-readable device name
    #[allow(dead_code)]
    pub friendly_name: String,
    /// Firmware version
    #[allow(dead_code)]
    pub firmware_version: String,
    /// Device serial number
    #[allow(dead_code)]
    pub serial_number: String,
}

/// WS-Discovery server for ONVIF device discovery
///
/// This server handles multicast UDP communication for device discovery
/// according to the WS-Discovery specification. It responds to probe requests
/// and sends hello/bye announcements.
pub struct WSDiscoveryServer {
    device_info: DeviceInfo,
    socket: UdpSocket,
    debug: bool,
}

impl WSDiscoveryServer {
    /// Creates a new WS-Discovery server
    ///
    /// # Arguments
    /// * `device_info` - Device information for announcements
    /// * `interface_addr` - Local interface IP address to bind to
    /// * `debug` - Enable verbose logging
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn std::error::Error>>` - Server instance or error
    pub fn new(
        device_info: DeviceInfo,
        interface_addr: &str,
        debug: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Bind to 0.0.0.0:3702 to listen on all interfaces for multicast
        let bind_addr = "0.0.0.0:3702";
        let socket = UdpSocket::bind(bind_addr)
            .map_err(|e| format!("Failed to bind to {bind_addr}: {e}"))?;

        // Set socket options for better multicast handling
        socket
            .set_broadcast(true)
            .map_err(|e| format!("Failed to set broadcast: {e}"))?;

        // Join the multicast group
        let multicast_addr: Ipv4Addr = "239.255.255.250"
            .parse()
            .map_err(|e| format!("Invalid multicast address: {e}"))?;
        let interface_addr: Ipv4Addr = interface_addr
            .parse()
            .map_err(|e| format!("Invalid interface address: {e}"))?;

        socket
            .join_multicast_v4(&multicast_addr, &interface_addr)
            .map_err(|e| format!("Failed to join multicast group: {e}"))?;

        println!("WS-Discovery server bound to {bind_addr}");
        println!(
            "Joined multicast group {WS_DISCOVERY_MULTICAST_ADDR} on interface {interface_addr}"
        );

        Ok(WSDiscoveryServer {
            device_info,
            socket,
            debug,
        })
    }

    /// Starts the WS-Discovery server main loop
    ///
    /// This method sends a hello message and then listens for incoming probe requests.
    /// It will continue running until an unrecoverable error occurs.
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if server stops gracefully, Err on error
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Send Hello message on startup
        self.send_hello()?;

        println!("WS-Discovery server started, listening for probe requests...");

        // Set a reasonable receive timeout to avoid blocking indefinitely
        let timeout = std::time::Duration::from_secs(1);
        self.socket.set_read_timeout(Some(timeout))?;

        let mut buffer = [0; 4096];
        let mut message_count = 0u32;
        let mut last_hello = std::time::Instant::now();
        let hello_interval = std::time::Duration::from_secs(60); // Send Hello every 60 seconds

        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    message_count += 1;
                    let message = String::from_utf8_lossy(&buffer[..size]);
                    if let Err(e) = self.handle_message(&message, src) {
                        eprintln!(
                            "Error handling WS-Discovery message #{message_count} from {src}: {e}"
                        );
                    }
                }
                Err(e) => {
                    // Handle timeout as normal (not an error)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        // Check if we should send a periodic Hello message
                        if last_hello.elapsed() >= hello_interval {
                            if let Err(e) = self.send_hello() {
                                eprintln!("Failed to send periodic Hello message: {e}");
                            }
                            last_hello = std::time::Instant::now();
                        }

                        // Periodic status update every ~10 seconds
                        if message_count % 10 == 0 && message_count > 0 && self.debug {
                            println!(
                                "WS-Discovery: Processed {message_count} messages, still listening..."
                            );
                        }
                        continue;
                    } else {
                        eprintln!("Error receiving WS-Discovery message: {e}");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handles incoming WS-Discovery messages
    ///
    /// # Arguments
    /// * `message` - The received XML message
    /// * `src` - Source address of the message
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if handled successfully, Err on error
    fn handle_message(
        &self,
        message: &str,
        src: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Log first line of message for debugging (avoid logging full XML for brevity)
        if self.debug {
            let first_line = message.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() {
                println!("Received WS-Discovery message from {src}: {first_line}");
            }
        }

        if is_probe_request(message) {
            if self.debug {
                println!("Detected Probe request from {src}, sending ProbeMatch response");
            }
            let message_id = extract_message_id(message);
            self.send_probe_match(src, &message_id)?;
        } else if self.debug {
            println!("Received non-probe message from {src} (ignoring)");
        }

        Ok(())
    }

    /// Sends a Hello announcement message to the multicast group
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if sent successfully, Err on error
    fn send_hello(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = generate_uuid();
        let hello_message = create_hello_message(&self.device_info, &message_id);

        let multicast_addr: SocketAddr = WS_DISCOVERY_MULTICAST_ADDR
            .parse()
            .map_err(|e| format!("Invalid multicast address: {e}"))?;

        println!("Sending Hello message to {multicast_addr}");
        if self.debug {
            println!("Hello message details:");
            println!("  - Device Name: {}", self.device_info.friendly_name);
            println!("  - Types: {}", self.device_info.types);
            println!("  - XAddrs: {}", self.device_info.xaddrs);
            println!("  - Scopes: {}", self.device_info.scopes);
        }

        self.socket
            .send_to(hello_message.as_bytes(), multicast_addr)
            .map_err(|e| format!("Failed to send Hello message: {e}"))?;

        println!("Hello message sent successfully (MessageID: {message_id})");
        Ok(())
    }

    /// Sends a Bye announcement message to the multicast group
    ///
    /// This method is typically called when the device is shutting down.
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if sent successfully, Err on error
    pub fn send_bye(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = generate_uuid();
        let bye_message = create_bye_message(&self.device_info, &message_id);

        let multicast_addr: SocketAddr = WS_DISCOVERY_MULTICAST_ADDR
            .parse()
            .map_err(|e| format!("Invalid multicast address: {e}"))?;

        self.socket
            .send_to(bye_message.as_bytes(), multicast_addr)
            .map_err(|e| format!("Failed to send Bye message: {e}"))?;

        println!("Sent Bye message");
        Ok(())
    }

    /// Sends a ProbeMatch response to a specific client
    ///
    /// # Arguments
    /// * `dest` - Destination address to send the response to
    /// * `relates_to` - MessageID from the original Probe request
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if sent successfully, Err on error
    fn send_probe_match(
        &self,
        dest: SocketAddr,
        relates_to: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = generate_uuid();
        let probe_match = create_probe_match_message(&self.device_info, &message_id, relates_to);

        if self.debug {
            println!("Sending ProbeMatch response to {dest}");
            println!("  - RelatesTo: {relates_to}");
            println!("  - MessageID: {message_id}");
            println!("  - XAddrs: {}", self.device_info.xaddrs);
        }

        self.socket
            .send_to(probe_match.as_bytes(), dest)
            .map_err(|e| format!("Failed to send ProbeMatch to {dest}: {e}"))?;

        if self.debug {
            println!("ProbeMatch sent successfully to {dest}");
        }
        Ok(())
    }
}

/// Implement Drop to send a Bye message when the server is dropped
impl Drop for WSDiscoveryServer {
    fn drop(&mut self) {
        if let Err(e) = self.send_bye() {
            eprintln!("Failed to send Bye message on drop: {e}");
        }
    }
}

// --- Helper functions (pure logic, testable) ---

fn is_probe_request(message: &str) -> bool {
    // Enhanced probe detection - check for various probe patterns
    let is_probe_request = message.contains("Probe")
        && (message.contains(WS_DISCOVERY_NAMESPACE) || message.contains("discovery"))
        && (message.contains("ProbeType")
            || message.contains("Types")
            || !message.contains("ProbeMatch"));

    // Also check for specific ONVIF probe patterns
    let is_onvif_probe = message.contains("NetworkVideoTransmitter")
        || message.contains("tdn:")
        || message.contains("onvif://www.onvif.org");

    is_probe_request || is_onvif_probe
}

fn extract_message_id(message: &str) -> String {
    // List of possible MessageID patterns to try
    let patterns = [
        ("<a:MessageID>", "</a:MessageID>"),
        ("<wsa:MessageID>", "</wsa:MessageID>"),
        ("<MessageID>", "</MessageID>"),
        ("<soap:MessageID>", "</soap:MessageID>"),
        ("<s:MessageID>", "</s:MessageID>"),
    ];

    for (start_tag, end_tag) in patterns.iter() {
        if let Some(start) = message.find(start_tag) {
            if let Some(end) = message[start..].find(end_tag) {
                let id_start = start + start_tag.len();
                let id_end = start + end;
                let message_id = message[id_start..id_end].trim();

                // Clean up the message ID - remove urn:uuid: prefix if present
                if message_id.starts_with("urn:uuid:") {
                    return message_id[9..].to_string();
                } else if !message_id.is_empty() {
                    return message_id.to_string();
                }
            }
        }
    }

    // Fallback to generating a new UUID
    println!("Could not extract MessageID from probe request, generating new one");
    generate_uuid()
}

fn create_hello_message(device_info: &DeviceInfo, message_id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsa="{}" xmlns:wsd="{}">
<soap:Header>
<wsa:Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/Hello</wsa:Action>
<wsa:MessageID>urn:uuid:{}</wsa:MessageID>
<wsa:To>urn:schemas-xmlsoap-org:ws:2005:04:discovery</wsa:To>
</soap:Header>
<soap:Body>
<wsd:Hello>
<wsa:EndpointReference>
<wsa:Address>{}</wsa:Address>
</wsa:EndpointReference>
<wsd:Types>{}</wsd:Types>
<wsd:Scopes>{}</wsd:Scopes>
<wsd:XAddrs>{}</wsd:XAddrs>
<wsd:MetadataVersion>1</wsd:MetadataVersion>
</wsd:Hello>
</soap:Body>
</soap:Envelope>"#,
        WS_ADDRESSING_NAMESPACE,
        WS_DISCOVERY_NAMESPACE,
        message_id,
        device_info.endpoint_reference,
        device_info.types,
        device_info.scopes,
        device_info.xaddrs
    )
}

fn create_bye_message(device_info: &DeviceInfo, message_id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsa="{}" xmlns:wsd="{}">
<soap:Header>
<wsa:Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/Bye</wsa:Action>
<wsa:MessageID>urn:uuid:{}</wsa:MessageID>
<wsa:To>urn:schemas-xmlsoap-org:ws:2005:04:discovery</wsa:To>
</soap:Header>
<soap:Body>
<wsd:Bye>
<wsa:EndpointReference>
<wsa:Address>{}</wsa:Address>
</wsa:EndpointReference>
<wsd:Types>{}</wsd:Types>
<wsd:Scopes>{}</wsd:Scopes>
<wsd:XAddrs>{}</wsd:XAddrs>
<wsd:MetadataVersion>1</wsd:MetadataVersion>
</wsd:Bye>
</soap:Body>
</soap:Envelope>"#,
        WS_ADDRESSING_NAMESPACE,
        WS_DISCOVERY_NAMESPACE,
        message_id,
        device_info.endpoint_reference,
        device_info.types,
        device_info.scopes,
        device_info.xaddrs
    )
}

fn create_probe_match_message(
    device_info: &DeviceInfo,
    message_id: &str,
    relates_to: &str,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsa="{}" xmlns:wsd="{}">
<soap:Header>
<wsa:Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/ProbeMatches</wsa:Action>
<wsa:MessageID>urn:uuid:{}</wsa:MessageID>
<wsa:RelatesTo>{}</wsa:RelatesTo>
<wsa:To>http://www.w3.org/2005/08/addressing/anonymous</wsa:To>
</soap:Header>
<soap:Body>
<wsd:ProbeMatches>
<wsd:ProbeMatch>
<wsa:EndpointReference>
<wsa:Address>{}</wsa:Address>
</wsa:EndpointReference>
<wsd:Types>{}</wsd:Types>
<wsd:Scopes>{}</wsd:Scopes>
<wsd:XAddrs>{}</wsd:XAddrs>
<wsd:MetadataVersion>1</wsd:MetadataVersion>
</wsd:ProbeMatch>
</wsd:ProbeMatches>
</soap:Body>
</soap:Envelope>"#,
        WS_ADDRESSING_NAMESPACE,
        WS_DISCOVERY_NAMESPACE,
        message_id,
        relates_to,
        device_info.endpoint_reference,
        device_info.types,
        device_info.scopes,
        device_info.xaddrs
    )
}

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_probe_request() {
        let probe_msg = r#"<soap:Envelope><soap:Body><d:Probe><d:Types>tdn:NetworkVideoTransmitter</d:Types></d:Probe></soap:Body></soap:Envelope>"#;
        // Note: The simple contains check might fail if namespaces aren't exactly as expected in the constant,
        // but the function checks for "Probe" and "Types" so it should pass.
        // Let's make a more realistic probe message that matches the logic
        let valid_probe = format!(
            r#"<soap:Envelope xmlns:d="{}"><soap:Body><d:Probe><d:Types>tdn:NetworkVideoTransmitter</d:Types></d:Probe></soap:Body></soap:Envelope>"#,
            WS_DISCOVERY_NAMESPACE
        );
        assert!(is_probe_request(&valid_probe));

        let non_probe = "Just some random text";
        assert!(!is_probe_request(non_probe));
    }

    #[test]
    fn test_extract_message_id() {
        let msg_with_id =
            r#"<soap:Header><wsa:MessageID>urn:uuid:12345-67890</wsa:MessageID></soap:Header>"#;
        assert_eq!(extract_message_id(msg_with_id), "12345-67890");

        let msg_without_id = r#"<soap:Header><wsa:To>somewhere</wsa:To></soap:Header>"#;
        // Should generate a new UUID (length 36)
        assert_eq!(extract_message_id(msg_without_id).len(), 36);
    }

    #[test]
    fn test_create_hello_message() {
        let device_info = DeviceInfo {
            endpoint_reference: "urn:uuid:test-endpoint".to_string(),
            types: "tdn:TestDevice".to_string(),
            scopes: "onvif://www.onvif.org/test".to_string(),
            xaddrs: "http://127.0.0.1:8080/onvif".to_string(),
            manufacturer: "Test Mfg".to_string(),
            model_name: "Test Model".to_string(),
            friendly_name: "Test Device".to_string(),
            firmware_version: "1.0".to_string(),
            serial_number: "12345".to_string(),
        };

        let hello = create_hello_message(&device_info, "test-message-id");
        assert!(hello.contains("Hello"));
        assert!(hello.contains("urn:uuid:test-message-id"));
        assert!(hello.contains("urn:uuid:test-endpoint"));
        assert!(hello.contains("tdn:TestDevice"));
    }
}
