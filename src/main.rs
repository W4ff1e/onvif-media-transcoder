use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    println!("Starting ONVIF Camera Emulator...");

    // Strict configuration - no defaults, expect environment variables to be set
    let rtsp_input = std::env::var("RTSP_INPUT")
        .expect("RTSP_INPUT environment variable must be set");
    let onvif_port = std::env::var("ONVIF_PORT")
        .expect("ONVIF_PORT environment variable must be set");
    let device_name = std::env::var("DEVICE_NAME")
        .expect("DEVICE_NAME environment variable must be set");

    // Validate port number
    let _: u16 = onvif_port.parse()
        .expect("ONVIF_PORT must be a valid port number");

    println!("Configuration:");
    println!("  RTSP Input Stream: {}", rtsp_input);
    println!("  ONVIF Port: {}", onvif_port);
    println!("  Device Name: {}", device_name);

    // Start ONVIF web service
    start_onvif_service(&onvif_port, &rtsp_input, &device_name);
}

fn start_onvif_service(port: &str, rtsp_stream: &str, device_name: &str) {
    println!("Starting ONVIF web service on port {}", port);
    println!("Exposing RTSP stream: {}", rtsp_stream);
    println!("Device Name: {}", device_name);
    println!("WSDD device discovery is running in background");

    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind to ONVIF port");

    println!("ONVIF Camera service running on port {}", port);
    println!("Device discovery available via WSDD");
    println!("Stream URI: {}", rtsp_stream);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let rtsp_clone = rtsp_stream.to_string();
                let device_clone = device_name.to_string();
                thread::spawn(move || {
                    handle_onvif_request(stream, &rtsp_clone, &device_clone);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

fn handle_onvif_request(mut stream: TcpStream, rtsp_stream: &str, device_name: &str) {
    let mut buffer = [0; 2048];
    if let Ok(size) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..size]);

        println!(
            "Received ONVIF request: {}",
            request.lines().next().unwrap_or("Unknown")
        );

        // Enhanced ONVIF endpoint routing
        if request.contains("GetCapabilities") {
            send_capabilities_response(&mut stream, rtsp_stream);
        } else if request.contains("GetProfiles") {
            send_profiles_response(&mut stream, rtsp_stream);
        } else if request.contains("GetStreamUri") {
            send_stream_uri_response(&mut stream, rtsp_stream);
        } else if request.contains("GetDeviceInformation") {
            send_device_info_response(&mut stream, device_name);
        } else if request.contains("GetVideoSources") {
            send_video_sources_response(&mut stream);
        } else if request.contains("GetServiceCapabilities") {
            send_service_capabilities_response(&mut stream);
        } else {
            send_default_response(&mut stream);
        }
    }
}

fn send_capabilities_response(stream: &mut TcpStream, _rtsp_stream: &str) {
    let response = r#"HTTP/1.1 200 OK
Content-Type: application/soap+xml
Content-Length: 1200

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<tds:GetCapabilitiesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
<tds:Capabilities>
<tt:Device xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:XAddr>http://localhost:8080/onvif/device_service</tt:XAddr>
<tt:Network>
<tt:IPFilter>false</tt:IPFilter>
<tt:ZeroConfiguration>false</tt:ZeroConfiguration>
<tt:IPVersion6>false</tt:IPVersion6>
<tt:DynDNS>false</tt:DynDNS>
</tt:Network>
<tt:System>
<tt:DiscoveryResolve>false</tt:DiscoveryResolve>
<tt:DiscoveryBye>false</tt:DiscoveryBye>
<tt:RemoteDiscovery>false</tt:RemoteDiscovery>
<tt:SystemBackup>false</tt:SystemBackup>
<tt:SystemLogging>false</tt:SystemLogging>
<tt:FirmwareUpgrade>false</tt:FirmwareUpgrade>
<tt:SupportedVersions>
<tt:Major>2</tt:Major>
<tt:Minor>60</tt:Minor>
</tt:SupportedVersions>
</tt:System>
<tt:IO>
<tt:InputConnectors>0</tt:InputConnectors>
<tt:RelayOutputs>0</tt:RelayOutputs>
</tt:IO>
<tt:Security>
<tt:TLS1.1>false</tt:TLS1.1>
<tt:TLS1.2>false</tt:TLS1.2>
<tt:OnboardKeyGeneration>false</tt:OnboardKeyGeneration>
<tt:AccessPolicyConfig>false</tt:AccessPolicyConfig>
<tt:X.509Token>false</tt:X.509Token>
<tt:SAMLToken>false</tt:SAMLToken>
<tt:KerberosToken>false</tt:KerberosToken>
<tt:RELToken>false</tt:RELToken>
</tt:Security>
</tt:Device>
<tt:Media xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:XAddr>http://localhost:8080/onvif/media_service</tt:XAddr>
<tt:StreamingCapabilities>
<tt:RTPMulticast>false</tt:RTPMulticast>
<tt:RTP_TCP>true</tt:RTP_TCP>
<tt:RTP_RTSP_TCP>true</tt:RTP_RTSP_TCP>
</tt:StreamingCapabilities>
</tt:Media>
</tds:Capabilities>
</tds:GetCapabilitiesResponse>
</soap:Body>
</soap:Envelope>"#;

    let _ = stream.write_all(response.as_bytes());
}

fn send_profiles_response(stream: &mut TcpStream, _rtsp_stream: &str) {
    let response = r#"HTTP/1.1 200 OK
Content-Type: application/soap+xml
Content-Length: 1500

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetProfilesResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:Profiles token="MainProfile" fixed="true">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">MainProfile</tt:Name>
<tt:VideoSourceConfiguration xmlns:tt="http://www.onvif.org/ver10/schema" token="VideoSourceConfig">
<tt:Name>VideoSourceConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>VideoSource_1</tt:SourceToken>
<tt:Bounds x="0" y="0" width="1920" height="1080"/>
</tt:VideoSourceConfiguration>
<tt:VideoEncoderConfiguration xmlns:tt="http://www.onvif.org/ver10/schema" token="VideoEncoderConfig">
<tt:Name>VideoEncoderConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:Encoding>H264</tt:Encoding>
<tt:Resolution>
<tt:Width>1920</tt:Width>
<tt:Height>1080</tt:Height>
</tt:Resolution>
<tt:Quality>5</tt:Quality>
<tt:RateControl>
<tt:FrameRateLimit>30</tt:FrameRateLimit>
<tt:EncodingInterval>1</tt:EncodingInterval>
<tt:BitrateLimit>8000</tt:BitrateLimit>
</tt:RateControl>
<tt:H264>
<tt:GovLength>30</tt:GovLength>
<tt:H264Profile>Main</tt:H264Profile>
</tt:H264>
<tt:Multicast>
<tt:Address>
<tt:Type>IPv4</tt:Type>
<tt:IPv4Address>0.0.0.0</tt:IPv4Address>
</tt:Address>
<tt:Port>0</tt:Port>
<tt:TTL>1</tt:TTL>
<tt:AutoStart>false</tt:AutoStart>
</tt:Multicast>
<tt:SessionTimeout>PT60S</tt:SessionTimeout>
</tt:VideoEncoderConfiguration>
<tt:AudioSourceConfiguration xmlns:tt="http://www.onvif.org/ver10/schema" token="AudioSourceConfig">
<tt:Name>AudioSourceConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>AudioSource_1</tt:SourceToken>
</tt:AudioSourceConfiguration>
<tt:AudioEncoderConfiguration xmlns:tt="http://www.onvif.org/ver10/schema" token="AudioEncoderConfig">
<tt:Name>AudioEncoderConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:Encoding>AAC</tt:Encoding>
<tt:Bitrate>64000</tt:Bitrate>
<tt:SampleRate>48000</tt:SampleRate>
<tt:Multicast>
<tt:Address>
<tt:Type>IPv4</tt:Type>
<tt:IPv4Address>0.0.0.0</tt:IPv4Address>
</tt:Address>
<tt:Port>0</tt:Port>
<tt:TTL>1</tt:TTL>
<tt:AutoStart>false</tt:AutoStart>
</tt:Multicast>
<tt:SessionTimeout>PT60S</tt:SessionTimeout>
</tt:AudioEncoderConfiguration>
</trt:Profiles>
</trt:GetProfilesResponse>
</soap:Body>
</soap:Envelope>"#;

    let _ = stream.write_all(response.as_bytes());
}

fn send_stream_uri_response(stream: &mut TcpStream, rtsp_stream: &str) {
    let response = format!(
        r#"HTTP/1.1 200 OK
Content-Type: application/soap+xml
Content-Length: 600

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetStreamUriResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:MediaUri>
<tt:Uri xmlns:tt="http://www.onvif.org/ver10/schema">{}</tt:Uri>
</trt:MediaUri>
</trt:GetStreamUriResponse>
</soap:Body>
</soap:Envelope>"#,
        rtsp_stream
    );

    let _ = stream.write_all(response.as_bytes());
}

fn send_device_info_response(stream: &mut TcpStream, device_name: &str) {
    let response = format!(
        r#"HTTP/1.1 200 OK
Content-Type: application/soap+xml
Content-Length: 900

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<tds:GetDeviceInformationResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
<tds:Manufacturer>FFmpeg Solutions</tds:Manufacturer>
<tds:Model>{}</tds:Model>
<tds:FirmwareVersion>1.0.0</tds:FirmwareVersion>
<tds:SerialNumber>EMU-{}</tds:SerialNumber>
<tds:HardwareId>ffmpeg-onvif-emulator</tds:HardwareId>
</tds:GetDeviceInformationResponse>
</soap:Body>
</soap:Envelope>"#,
        device_name,
        device_name.chars().take(6).collect::<String>()
    );

    let _ = stream.write_all(response.as_bytes());
}

fn send_default_response(stream: &mut TcpStream) {
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nONVIF Camera\n";
    let _ = stream.write_all(response.as_bytes());
}

fn send_video_sources_response(stream: &mut TcpStream) {
    let response = r#"HTTP/1.1 200 OK
Content-Type: application/soap+xml
Content-Length: 800

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetVideoSourcesResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:VideoSources token="VideoSource_1">
<tt:Framerate xmlns:tt="http://www.onvif.org/ver10/schema">30</tt:Framerate>
<tt:Resolution xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:Width>1920</tt:Width>
<tt:Height>1080</tt:Height>
</tt:Resolution>
</trt:VideoSources>
</trt:GetVideoSourcesResponse>
</soap:Body>
</soap:Envelope>"#;

    let _ = stream.write_all(response.as_bytes());
}

fn send_service_capabilities_response(stream: &mut TcpStream) {
    let response = r#"HTTP/1.1 200 OK
Content-Type: application/soap+xml
Content-Length: 600

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetServiceCapabilitiesResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:Capabilities>
<tt:ProfileCapabilities xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:MaximumNumberOfProfiles>2</tt:MaximumNumberOfProfiles>
</tt:ProfileCapabilities>
<tt:StreamingCapabilities xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:RTPMulticast>false</tt:RTPMulticast>
<tt:RTP_TCP>true</tt:RTP_TCP>
<tt:RTP_RTSP_TCP>true</tt:RTP_RTSP_TCP>
</tt:StreamingCapabilities>
</trt:Capabilities>
</trt:GetServiceCapabilitiesResponse>
</soap:Body>
</soap:Envelope>"#;

    let _ = stream.write_all(response.as_bytes());
}
