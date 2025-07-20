// ONVIF Response Templates
// This module contains all the hardcoded ONVIF SOAP responses

pub fn get_capabilities_response(container_ip: &str, onvif_port: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<tds:GetCapabilitiesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
<tds:Capabilities>
<tt:Device xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:XAddr>http://{}:{}/onvif/device_service</tt:XAddr>
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
<tt:XAddr>http://{}:{}/onvif/media_service</tt:XAddr>
<tt:StreamingCapabilities>
<tt:RTPMulticast>false</tt:RTPMulticast>
<tt:RTP_TCP>true</tt:RTP_TCP>
<tt:RTP_RTSP_TCP>true</tt:RTP_RTSP_TCP>
</tt:StreamingCapabilities>
</tt:Media>
</tds:Capabilities>
</tds:GetCapabilitiesResponse>
</soap:Body>
</soap:Envelope>"#,
        container_ip, onvif_port, container_ip, onvif_port
    )
}

pub fn get_profiles_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
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
</soap:Envelope>"#.to_string()
}

pub fn get_stream_uri_response(rtsp_stream: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
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
    )
}

pub fn get_device_info_response(device_name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<tds:GetDeviceInformationResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
<tds:Manufacturer>ONVIF Media Solutions</tds:Manufacturer>
<tds:Model>{}</tds:Model>
<tds:FirmwareVersion>1.0.0</tds:FirmwareVersion>
<tds:SerialNumber>EMU-{}</tds:SerialNumber>
<tds:HardwareId>onvif-media-transcoder</tds:HardwareId>
</tds:GetDeviceInformationResponse>
</soap:Body>
</soap:Envelope>"#,
        device_name,
        device_name.chars().take(6).collect::<String>()
    )
}

pub fn get_video_sources_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
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
</soap:Envelope>"#
        .to_string()
}

pub fn get_service_capabilities_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
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
</soap:Envelope>"#
        .to_string()
}

pub fn get_auth_required_response() -> String {
    r#"HTTP/1.1 401 Unauthorized
WWW-Authenticate: Basic realm="ONVIF Camera"
WWW-Authenticate: Digest realm="ONVIF Camera", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093", qop="auth"
Content-Type: application/soap+xml
Content-Length: 0

"#.to_string()
}

pub fn get_default_response() -> String {
    "ONVIF Camera\n".to_string()
}
