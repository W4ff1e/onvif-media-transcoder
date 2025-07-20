// ONVIF Response Templates
// This module contains all the hardcoded ONVIF SOAP responses

use chrono::{Datelike, Timelike};

pub fn get_capabilities_response(container_ip: &str, onvif_port: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<tds:GetCapabilitiesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
<tds:Capabilities>
<tt:Device xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:XAddr>http://{container_ip}:{onvif_port}/onvif/device_service</tt:XAddr>
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
<tt:XAddr>http://{container_ip}:{onvif_port}/onvif/device_service</tt:XAddr>
<tt:StreamingCapabilities>
<tt:RTPMulticast>false</tt:RTPMulticast>
<tt:RTP_TCP>true</tt:RTP_TCP>
<tt:RTP_RTSP_TCP>true</tt:RTP_RTSP_TCP>
</tt:StreamingCapabilities>
</tt:Media>
<tt:PTZ xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:XAddr>http://{container_ip}:{onvif_port}/onvif/device_service</tt:XAddr>
</tt:PTZ>
</tds:Capabilities>
</tds:GetCapabilitiesResponse>
</soap:Body>
</soap:Envelope>"#
    )
}

pub fn get_services_response(container_ip: &str, onvif_port: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<tds:GetServicesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
<tds:Service>
<tds:Namespace>http://www.onvif.org/ver10/device/wsdl</tds:Namespace>
<tds:XAddr>http://{container_ip}:{onvif_port}/onvif/device_service</tds:XAddr>
<tds:Capabilities>
<tds:Network>
<tds:IPFilter>false</tds:IPFilter>
<tds:ZeroConfiguration>false</tds:ZeroConfiguration>
<tds:IPVersion6>false</tds:IPVersion6>
<tds:DynDNS>false</tds:DynDNS>
</tds:Network>
<tds:System>
<tds:DiscoveryResolve>false</tds:DiscoveryResolve>
<tds:DiscoveryBye>false</tds:DiscoveryBye>
<tds:RemoteDiscovery>false</tds:RemoteDiscovery>
<tds:SystemBackup>false</tds:SystemBackup>
<tds:SystemLogging>false</tds:SystemLogging>
<tds:FirmwareUpgrade>false</tds:FirmwareUpgrade>
<tds:SupportedVersions>
<tds:Major>2</tds:Major>
<tds:Minor>60</tds:Minor>
</tds:SupportedVersions>
</tds:System>
<tds:IO>
<tds:InputConnectors>0</tds:InputConnectors>
<tds:RelayOutputs>0</tds:RelayOutputs>
</tds:IO>
<tds:Security>
<tds:TLS1.1>false</tds:TLS1.1>
<tds:TLS1.2>false</tds:TLS1.2>
<tds:OnboardKeyGeneration>false</tds:OnboardKeyGeneration>
<tds:AccessPolicyConfig>false</tds:AccessPolicyConfig>
<tds:X.509Token>false</tds:X.509Token>
<tds:SAMLToken>false</tds:SAMLToken>
<tds:KerberosToken>false</tds:KerberosToken>
<tds:RELToken>false</tds:RELToken>
</tds:Security>
</tds:Capabilities>
<tds:Version>
<tds:Major>2</tds:Major>
<tds:Minor>60</tds:Minor>
</tds:Version>
</tds:Service>
<tds:Service>
<tds:Namespace>http://www.onvif.org/ver10/media/wsdl</tds:Namespace>
<tds:XAddr>http://{container_ip}:{onvif_port}/onvif/device_service</tds:XAddr>
<tds:Capabilities>
<tds:StreamingCapabilities>
<tds:RTPMulticast>false</tds:RTPMulticast>
<tds:RTP_TCP>true</tds:RTP_TCP>
<tds:RTP_RTSP_TCP>true</tds:RTP_RTSP_TCP>
</tds:StreamingCapabilities>
</tds:Capabilities>
<tds:Version>
<tds:Major>2</tds:Major>
<tds:Minor>60</tds:Minor>
</tds:Version>
</tds:Service>
</tds:GetServicesResponse>
</soap:Body>
</soap:Envelope>"#
    )
}

pub fn get_profiles_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetProfilesResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:Profiles token="MainProfile" fixed="true">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">MainProfile</tt:Name>
<tt:VideoSourceConfiguration token="VideoSourceConfig">
<tt:Name>VideoSourceConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>VideoSource_1</tt:SourceToken>
<tt:Bounds x="0" y="0" width="1920" height="1080"/>
</tt:VideoSourceConfiguration>
<tt:VideoEncoderConfiguration token="VideoEncoderConfig">
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
<tt:AudioSourceConfiguration token="AudioSourceConfig">
<tt:Name>AudioSourceConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>AudioSource_1</tt:SourceToken>
</tt:AudioSourceConfiguration>
<tt:AudioEncoderConfiguration token="AudioEncoderConfig">
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
<tt:PTZConfiguration token="PTZConfig">
<tt:Name>PTZConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:NodeToken>PTZNode_1</tt:NodeToken>
<tt:DefaultAbsolutePantTiltPositionSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:DefaultAbsolutePantTiltPositionSpace>
<tt:DefaultAbsoluteZoomPositionSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/PositionGenericSpace</tt:DefaultAbsoluteZoomPositionSpace>
<tt:DefaultRelativePanTiltTranslationSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/TranslationGenericSpace</tt:DefaultRelativePanTiltTranslationSpace>
<tt:DefaultRelativeZoomTranslationSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/TranslationGenericSpace</tt:DefaultRelativeZoomTranslationSpace>
<tt:DefaultContinuousPanTiltVelocitySpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/VelocityGenericSpace</tt:DefaultContinuousPanTiltVelocitySpace>
<tt:DefaultContinuousZoomVelocitySpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/VelocityGenericSpace</tt:DefaultContinuousZoomVelocitySpace>
<tt:DefaultPTZSpeed>
<tt:PanTilt x="0.1" y="0.1" space="http://www.onvif.org/ver10/tptz/PanTiltSpaces/GenericSpeedSpace"/>
<tt:Zoom x="0.1" space="http://www.onvif.org/ver10/tptz/ZoomSpaces/ZoomGenericSpeedSpace"/>
</tt:DefaultPTZSpeed>
<tt:DefaultPTZTimeout>PT5S</tt:DefaultPTZTimeout>
</tt:PTZConfiguration>
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
<tt:Uri xmlns:tt="http://www.onvif.org/ver10/schema">{rtsp_stream}</tt:Uri>
</trt:MediaUri>
</trt:GetStreamUriResponse>
</soap:Body>
</soap:Envelope>"#
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

pub fn get_video_source_configurations_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetVideoSourceConfigurationsResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:Configurations token="VideoSourceConfig">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">VideoSourceConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>VideoSource_1</tt:SourceToken>
<tt:Bounds x="0" y="0" width="1920" height="1080"/>
</trt:Configurations>
</trt:GetVideoSourceConfigurationsResponse>
</soap:Body>
</soap:Envelope>"#
        .to_string()
}

pub fn get_video_encoder_configurations_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetVideoEncoderConfigurationsResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:Configurations token="VideoEncoderConfig">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">VideoEncoderConfig</tt:Name>
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
</trt:Configurations>
</trt:GetVideoEncoderConfigurationsResponse>
</soap:Body>
</soap:Envelope>"#
        .to_string()
}

pub fn get_audio_source_configurations_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetAudioSourceConfigurationsResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:Configurations token="AudioSourceConfig">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">AudioSourceConfig</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>AudioSource_1</tt:SourceToken>
</trt:Configurations>
</trt:GetAudioSourceConfigurationsResponse>
</soap:Body>
</soap:Envelope>"#
        .to_string()
}

pub fn get_audio_encoder_configurations_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetAudioEncoderConfigurationsResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:Configurations token="AudioEncoderConfig">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">AudioEncoderConfig</tt:Name>
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
</trt:Configurations>
</trt:GetAudioEncoderConfigurationsResponse>
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

pub fn get_snapshot_uri_response(container_ip: &str, onvif_port: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<trt:GetSnapshotUriResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
<trt:MediaUri>
<tt:Uri xmlns:tt="http://www.onvif.org/ver10/schema">http://{container_ip}:{onvif_port}/snapshot.jpg</tt:Uri>
</trt:MediaUri>
</trt:GetSnapshotUriResponse>
</soap:Body>
</soap:Envelope>"#
    )
}

pub fn get_system_date_time_response() -> String {
    // Get current UTC time
    let now = chrono::Utc::now();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:tds="http://www.onvif.org/ver10/device/wsdl" xmlns:tt="http://www.onvif.org/ver10/schema">
<soap:Body>
<tds:GetSystemDateAndTimeResponse>
<tds:SystemDateAndTime>
<tt:DateTimeType>NTP</tt:DateTimeType>
<tt:DaylightSavings>false</tt:DaylightSavings>
<tt:TimeZone>
<tt:TZ>UTC</tt:TZ>
</tt:TimeZone>
<tt:UTCDateTime>
<tt:Time>
<tt:Hour>{}</tt:Hour>
<tt:Minute>{}</tt:Minute>
<tt:Second>{}</tt:Second>
</tt:Time>
<tt:Date>
<tt:Year>{}</tt:Year>
<tt:Month>{}</tt:Month>
<tt:Day>{}</tt:Day>
</tt:Date>
</tt:UTCDateTime>
</tds:SystemDateAndTime>
</tds:GetSystemDateAndTimeResponse>
</soap:Body>
</soap:Envelope>"#,
        now.hour(),
        now.minute(),
        now.second(),
        now.year(),
        now.month(),
        now.day()
    )
}

pub fn get_unsupported_endpoint_response(endpoint: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<soap:Fault>
<soap:Code>
<soap:Value>soap:Receiver</soap:Value>
</soap:Code>
<soap:Reason>
<soap:Text xml:lang="en">The requested operation '{endpoint}' is not supported by this ONVIF Media Transcoder implementation.</soap:Text>
</soap:Reason>
<soap:Detail>
<ter:Action xmlns:ter="http://www.onvif.org/ver10/error">
<ter:Operation>{endpoint}</ter:Operation>
<ter:Category>Receiver</ter:Category>
<ter:Reason>OperationNotSupported</ter:Reason>
<ter:Detail>This ONVIF Media Transcoder supports basic streaming functionality. The requested operation is not implemented.</ter:Detail>
</ter:Action>
</soap:Detail>
</soap:Fault>
</soap:Body>
</soap:Envelope>"#
    )
}
