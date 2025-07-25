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
<tt:UsernameToken>true</tt:UsernameToken>
<tt:HttpDigest>true</tt:HttpDigest>
<tt:WSUsernameToken>true</tt:WSUsernameToken>
<tt:WSSecurityDuration>5</tt:WSSecurityDuration>
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
<trt:Profiles token="HQProfile" fixed="true">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">HQProfile</tt:Name>
<tt:VideoSourceConfiguration token="VideoSourceConfig_HQ">
<tt:Name>VideoSourceConfig_HQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>VideoSource_1</tt:SourceToken>
<tt:Bounds x="0" y="0" width="960" height="540"/>
</tt:VideoSourceConfiguration>
<tt:VideoEncoderConfiguration token="VideoEncoderConfig_HQ">
<tt:Name>VideoEncoderConfig_HQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:Encoding>H264</tt:Encoding>
<tt:Resolution>
<tt:Width>960</tt:Width>
<tt:Height>540</tt:Height>
</tt:Resolution>
<tt:Quality>4</tt:Quality>
<tt:RateControl>
<tt:FrameRateLimit>15</tt:FrameRateLimit>
<tt:EncodingInterval>1</tt:EncodingInterval>
<tt:BitrateLimit>1500</tt:BitrateLimit>
</tt:RateControl>
<tt:H264>
<tt:GovLength>15</tt:GovLength>
<tt:H264Profile>Main</tt:H264Profile>
<tt:Level>4.1</tt:Level>
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
</trt:Profiles>
<trt:Profiles token="LQProfile" fixed="true">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">LQProfile</tt:Name>
<tt:VideoSourceConfiguration token="VideoSourceConfig_LQ">
<tt:Name>VideoSourceConfig_LQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>VideoSource_1</tt:SourceToken>
<tt:Bounds x="0" y="0" width="960" height="540"/>
</tt:VideoSourceConfiguration>
<tt:VideoEncoderConfiguration token="VideoEncoderConfig_LQ">
<tt:Name>VideoEncoderConfig_LQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:Encoding>H264</tt:Encoding>
<tt:Resolution>
<tt:Width>960</tt:Width>
<tt:Height>540</tt:Height>
</tt:Resolution>
<tt:Quality>4</tt:Quality>
<tt:RateControl>
<tt:FrameRateLimit>15</tt:FrameRateLimit>
<tt:EncodingInterval>1</tt:EncodingInterval>
<tt:BitrateLimit>1500</tt:BitrateLimit>
</tt:RateControl>
<tt:H264>
<tt:GovLength>15</tt:GovLength>
<tt:H264Profile>Baseline</tt:H264Profile>
<tt:Level>3.1</tt:Level>
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
</trt:Profiles>
</trt:GetProfilesResponse>
</soap:Body>
</soap:Envelope>"#
        .to_string()
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
<tt:Framerate xmlns:tt="http://www.onvif.org/ver10/schema">15</tt:Framerate>
<tt:Resolution xmlns:tt="http://www.onvif.org/ver10/schema">
<tt:Width>960</tt:Width>
<tt:Height>540</tt:Height>
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
<trt:Configurations token="VideoSourceConfig_HQ">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">VideoSourceConfig_HQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>VideoSource_1</tt:SourceToken>
<tt:Bounds x="0" y="0" width="960" height="540"/>
</trt:Configurations>
<trt:Configurations token="VideoSourceConfig_LQ">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">VideoSourceConfig_LQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>VideoSource_1</tt:SourceToken>
<tt:Bounds x="0" y="0" width="960" height="540"/>
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
<trt:Configurations token="VideoEncoderConfig_HQ">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">VideoEncoderConfig_HQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:Encoding>H264</tt:Encoding>
<tt:Resolution>
<tt:Width>960</tt:Width>
<tt:Height>540</tt:Height>
</tt:Resolution>
<tt:Quality>4</tt:Quality>
<tt:RateControl>
<tt:FrameRateLimit>15</tt:FrameRateLimit>
<tt:EncodingInterval>1</tt:EncodingInterval>
<tt:BitrateLimit>1500</tt:BitrateLimit>
</tt:RateControl>
<tt:H264>
<tt:GovLength>15</tt:GovLength>
<tt:H264Profile>Main</tt:H264Profile>
<tt:Level>4.1</tt:Level>
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
<trt:Configurations token="VideoEncoderConfig_LQ">
<tt:Name xmlns:tt="http://www.onvif.org/ver10/schema">VideoEncoderConfig_LQ</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:Encoding>H264</tt:Encoding>
<tt:Resolution>
<tt:Width>960</tt:Width>
<tt:Height>540</tt:Height>
</tt:Resolution>
<tt:Quality>4</tt:Quality>
<tt:RateControl>
<tt:FrameRateLimit>15</tt:FrameRateLimit>
<tt:EncodingInterval>1</tt:EncodingInterval>
<tt:BitrateLimit>1500</tt:BitrateLimit>
</tt:RateControl>
<tt:H264>
<tt:GovLength>15</tt:GovLength>
<tt:H264Profile>Main</tt:H264Profile>
<tt:Level>4.1</tt:Level>
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
</trt:GetAudioEncoderConfigurationsResponse>
</soap:Body>
</soap:Envelope>"#
        .to_string()
}

pub fn get_auth_required_response() -> String {
    // Generate a fresh nonce for each authentication challenge
    let nonce = uuid::Uuid::new_v4().to_string().replace('-', "");

    format!(
        r#"HTTP/1.1 401 Unauthorized
WWW-Authenticate: Digest realm="ONVIF Camera", nonce="{nonce}", qop="auth", stale=false
Content-Type: application/soap+xml; charset=utf-8
Content-Length: 350

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
<soap:Body>
<soap:Fault>
<soap:Code>
<soap:Value>soap:Sender</soap:Value>
<soap:Subcode>
<soap:Value>ter:NotAuthorized</soap:Value>
</soap:Subcode>
</soap:Code>
<soap:Reason>
<soap:Text xml:lang="en">Authentication required</soap:Text>
</soap:Reason>
</soap:Fault>
</soap:Body>
</soap:Envelope>
"#
    )
}

pub fn get_ws_security_auth_fault() -> String {
    r#"HTTP/1.1 401 Unauthorized
Content-Type: application/soap+xml; charset=utf-8
Content-Length: 546

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:ter="http://www.onvif.org/ver10/error">
<soap:Body>
<soap:Fault>
<soap:Code>
<soap:Value>soap:Sender</soap:Value>
<soap:Subcode>
<soap:Value>ter:NotAuthorized</soap:Value>
</soap:Subcode>
</soap:Code>
<soap:Reason>
<soap:Text xml:lang="en">Sender not Authorized</soap:Text>
</soap:Reason>
<soap:Detail>
<soap:Text>WS-Security authentication required. Please provide UsernameToken with PasswordDigest or PasswordText.</soap:Text>
</soap:Detail>
</soap:Fault>
</soap:Body>
</soap:Envelope>
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
