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
}

impl WSDiscoveryServer {
    /// Creates a new WS-Discovery server
    ///
    /// # Arguments
    /// * `device_info` - Device information for announcements
    /// * `interface_addr` - Local interface IP address to bind to
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn std::error::Error>>` - Server instance or error
    pub fn new(
        device_info: DeviceInfo,
        interface_addr: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Bind to the multicast address
        let bind_addr = format!("{}:3702", interface_addr);
        let socket = UdpSocket::bind(&bind_addr)
            .map_err(|e| format!("Failed to bind to {}: {}", bind_addr, e))?;

        // Join the multicast group
        let multicast_addr: Ipv4Addr = "239.255.255.250"
            .parse()
            .map_err(|e| format!("Invalid multicast address: {}", e))?;
        let interface_addr: Ipv4Addr = interface_addr
            .parse()
            .map_err(|e| format!("Invalid interface address: {}", e))?;

        socket
            .join_multicast_v4(&multicast_addr, &interface_addr)
            .map_err(|e| format!("Failed to join multicast group: {}", e))?;

        println!("WS-Discovery server bound to {}", bind_addr);
        println!("Joined multicast group {}", WS_DISCOVERY_MULTICAST_ADDR);

        Ok(WSDiscoveryServer {
            device_info,
            socket,
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

        let mut buffer = [0; 4096];

        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    let message = String::from_utf8_lossy(&buffer[..size]);
                    if let Err(e) = self.handle_message(&message, src) {
                        eprintln!("Error handling WS-Discovery message from {}: {}", src, e);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving WS-Discovery message: {}", e);
                    break;
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
        let first_line = message.lines().next().unwrap_or("").trim();
        if !first_line.is_empty() {
            println!("Received WS-Discovery message from {}: {}", src, first_line);
        }

        // Check if this is a Probe request
        if message.contains("Probe") && message.contains(WS_DISCOVERY_NAMESPACE) {
            let message_id = self.extract_message_id(message);
            self.send_probe_match(src, &message_id)?;
        }

        Ok(())
    }

    /// Extracts the MessageID from a WS-Discovery message
    ///
    /// # Arguments
    /// * `message` - The XML message to parse
    ///
    /// # Returns
    /// * `String` - The extracted MessageID or a new UUID if not found
    fn extract_message_id(&self, message: &str) -> String {
        // Simple XML parsing to extract MessageID
        if let Some(start) = message.find("<a:MessageID>") {
            if let Some(end) = message[start..].find("</a:MessageID>") {
                let id_start = start + "<a:MessageID>".len();
                let id_end = start + end;
                return message[id_start..id_end].to_string();
            }
        }

        // Try alternative namespace prefix
        if let Some(start) = message.find("<wsa:MessageID>") {
            if let Some(end) = message[start..].find("</wsa:MessageID>") {
                let id_start = start + "<wsa:MessageID>".len();
                let id_end = start + end;
                return message[id_start..id_end].to_string();
            }
        }

        // Fallback to generating a new UUID
        self.generate_uuid()
    }

    /// Sends a Hello announcement message to the multicast group
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if sent successfully, Err on error
    fn send_hello(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = self.generate_uuid();
        let hello_message = self.create_hello_message(&message_id);

        let multicast_addr: SocketAddr = WS_DISCOVERY_MULTICAST_ADDR
            .parse()
            .map_err(|e| format!("Invalid multicast address: {}", e))?;

        self.socket
            .send_to(hello_message.as_bytes(), multicast_addr)
            .map_err(|e| format!("Failed to send Hello message: {}", e))?;

        println!("Sent Hello message");
        Ok(())
    }

    /// Sends a Bye announcement message to the multicast group
    ///
    /// This method is typically called when the device is shutting down.
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if sent successfully, Err on error
    pub fn send_bye(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = self.generate_uuid();
        let bye_message = self.create_bye_message(&message_id);

        let multicast_addr: SocketAddr = WS_DISCOVERY_MULTICAST_ADDR
            .parse()
            .map_err(|e| format!("Invalid multicast address: {}", e))?;

        self.socket
            .send_to(bye_message.as_bytes(), multicast_addr)
            .map_err(|e| format!("Failed to send Bye message: {}", e))?;

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
        let message_id = self.generate_uuid();
        let probe_match = self.create_probe_match_message(&message_id, relates_to);

        self.socket
            .send_to(probe_match.as_bytes(), dest)
            .map_err(|e| format!("Failed to send ProbeMatch to {}: {}", dest, e))?;

        println!("Sent ProbeMatch to {}", dest);
        Ok(())
    }

    /// Creates a Hello announcement message
    ///
    /// # Arguments
    /// * `message_id` - Unique message identifier
    ///
    /// # Returns
    /// * `String` - XML formatted Hello message
    fn create_hello_message(&self, message_id: &str) -> String {
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
            self.device_info.endpoint_reference,
            self.device_info.types,
            self.device_info.scopes,
            self.device_info.xaddrs
        )
    }

    /// Creates a Bye announcement message
    ///
    /// # Arguments
    /// * `message_id` - Unique message identifier
    ///
    /// # Returns
    /// * `String` - XML formatted Bye message
    fn create_bye_message(&self, message_id: &str) -> String {
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
            self.device_info.endpoint_reference,
            self.device_info.types,
            self.device_info.scopes,
            self.device_info.xaddrs
        )
    }

    /// Creates a ProbeMatch response message
    ///
    /// # Arguments
    /// * `message_id` - Unique message identifier
    /// * `relates_to` - MessageID from the original Probe request
    ///
    /// # Returns
    /// * `String` - XML formatted ProbeMatch message
    fn create_probe_match_message(&self, message_id: &str, relates_to: &str) -> String {
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
            self.device_info.endpoint_reference,
            self.device_info.types,
            self.device_info.scopes,
            self.device_info.xaddrs
        )
    }

    /// Generates a new UUID v4 string
    ///
    /// # Returns
    /// * `String` - A new UUID v4 as a string
    fn generate_uuid(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

/// Implement Drop to send a Bye message when the server is dropped
impl Drop for WSDiscoveryServer {
    fn drop(&mut self) {
        if let Err(e) = self.send_bye() {
            eprintln!("Failed to send Bye message on drop: {}", e);
        }
    }
}

/// Helper function to get the default network interface IP address
///
/// This function attempts to determine the local IP address by connecting
/// to a remote address and checking the local socket address.
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - The local IP address as a string, or an error
pub fn get_default_interface_ip() -> Result<String, Box<dyn std::error::Error>> {
    // Try to connect to a remote address to determine our local IP
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind temporary socket: {}", e))?;

    socket
        .connect("8.8.8.8:80")
        .map_err(|e| format!("Failed to connect to determine local IP: {}", e))?;

    let local_addr = socket
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;

    Ok(local_addr.ip().to_string())
}
