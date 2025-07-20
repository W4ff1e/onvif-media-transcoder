use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use uuid::Uuid;

const WS_DISCOVERY_MULTICAST_ADDR: &str = "239.255.255.250:3702";
const WS_DISCOVERY_NAMESPACE: &str = "http://schemas.xmlsoap.org/ws/2005/04/discovery";
const WS_ADDRESSING_NAMESPACE: &str = "http://www.w3.org/2005/08/addressing";

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub endpoint_reference: String,
    pub types: String,
    pub scopes: String,
    pub xaddrs: String,
    pub manufacturer: String,
    pub model_name: String,
    pub friendly_name: String,
    pub firmware_version: String,
    pub serial_number: String,
}

pub struct WSDiscoveryServer {
    device_info: DeviceInfo,
    socket: UdpSocket,
}

impl WSDiscoveryServer {
    pub fn new(
        device_info: DeviceInfo,
        interface_addr: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Bind to the multicast address
        let socket = UdpSocket::bind(format!("{}:3702", interface_addr))?;

        // Join the multicast group
        let multicast_addr: Ipv4Addr = "239.255.255.250".parse()?;
        let interface_addr: Ipv4Addr = interface_addr.parse()?;
        socket.join_multicast_v4(&multicast_addr, &interface_addr)?;

        println!("WS-Discovery server bound to {}:3702", interface_addr);
        println!("Joined multicast group {}", WS_DISCOVERY_MULTICAST_ADDR);

        Ok(WSDiscoveryServer {
            device_info,
            socket,
        })
    }

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
                        eprintln!("Error handling WS-Discovery message: {}", e);
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

    fn handle_message(
        &self,
        message: &str,
        src: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Received WS-Discovery message from {}: {}",
            src,
            message.lines().next().unwrap_or("")
        );

        if message.contains("Probe") && message.contains(WS_DISCOVERY_NAMESPACE) {
            // Extract message ID for correlation
            let message_id = self.extract_message_id(message);
            self.send_probe_match(src, &message_id)?;
        }

        Ok(())
    }

    fn extract_message_id(&self, message: &str) -> String {
        // Simple XML parsing to extract MessageID
        if let Some(start) = message.find("<a:MessageID>") {
            if let Some(end) = message[start..].find("</a:MessageID>") {
                let id_start = start + "<a:MessageID>".len();
                let id_end = start + end;
                return message[id_start..id_end].to_string();
            }
        }

        // Fallback to generating a new UUID
        self.generate_uuid()
    }

    fn send_hello(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = self.generate_uuid();
        let hello_message = self.create_hello_message(&message_id);

        let multicast_addr: SocketAddr = WS_DISCOVERY_MULTICAST_ADDR.parse()?;
        self.socket
            .send_to(hello_message.as_bytes(), multicast_addr)?;

        println!("Sent Hello message");
        Ok(())
    }

    fn send_bye(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = self.generate_uuid();
        let bye_message = self.create_bye_message(&message_id);

        let multicast_addr: SocketAddr = WS_DISCOVERY_MULTICAST_ADDR.parse()?;
        self.socket
            .send_to(bye_message.as_bytes(), multicast_addr)?;

        println!("Sent Bye message");
        Ok(())
    }

    fn send_probe_match(
        &self,
        dest: SocketAddr,
        relates_to: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message_id = self.generate_uuid();
        let probe_match = self.create_probe_match_message(&message_id, relates_to);

        self.socket.send_to(probe_match.as_bytes(), dest)?;

        println!("Sent ProbeMatch to {}", dest);
        Ok(())
    }

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

    fn generate_uuid(&self) -> String {
        // Generate a proper UUID v4
        Uuid::new_v4().to_string()
    }
}

// Helper function to get the default network interface IP
pub fn get_default_interface_ip() -> Result<String, Box<dyn std::error::Error>> {
    // Try to connect to a remote address to determine our local IP
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}
