//! ONVIF SOAP response bodies.
//!
//! Every function returns a complete SOAP envelope. Dynamic values are
//! escaped; namespaces are declared once on the envelope.

use crate::identity::DeviceIdentity;
use crate::onvif::soap::{xml_escape, SoapResponseBuilder};
use crate::onvif::stream_info::{StreamInfo, VideoEncoding};
use chrono::{Datelike, Timelike};
use std::net::Ipv4Addr;

pub const DEVICE_NS: &str = "http://www.onvif.org/ver10/device/wsdl";
pub const MEDIA_NS: &str = "http://www.onvif.org/ver10/media/wsdl";
pub const SCHEMA_NS: &str = "http://www.onvif.org/ver10/schema";

/// Token of the single media profile this device exposes.
pub const PROFILE_TOKEN: &str = "Profile_1";
pub const PROFILE_NAME: &str = "MainStream";
pub const VIDEO_SOURCE_TOKEN: &str = "VideoSource_1";
pub const VIDEO_SOURCE_CONFIG_TOKEN: &str = "VideoSourceConfig_1";
pub const VIDEO_ENCODER_CONFIG_TOKEN: &str = "VideoEncoderConfig_1";

/// ONVIF version advertised in capabilities.
const ONVIF_MAJOR: u32 = 2;
const ONVIF_MINOR: u32 = 60;

fn device_envelope(body: &str) -> String {
    SoapResponseBuilder::new()
        .add_namespace("tds", DEVICE_NS)
        .add_namespace("tt", SCHEMA_NS)
        .set_body(body)
        .build()
}

fn media_envelope(body: &str) -> String {
    SoapResponseBuilder::new()
        .add_namespace("trt", MEDIA_NS)
        .add_namespace("tt", SCHEMA_NS)
        .set_body(body)
        .build()
}

/// `GetCapabilitiesResponse` (legacy capability discovery).
pub fn capabilities(ip: Ipv4Addr, port: u16) -> String {
    let device_url = DeviceIdentity::device_service_url(ip, port);
    let media_url = DeviceIdentity::media_service_url(ip, port);
    device_envelope(&format!(
        "<tds:GetCapabilitiesResponse>
<tds:Capabilities>
<tt:Device>
<tt:XAddr>{device_url}</tt:XAddr>
<tt:Network>
<tt:IPFilter>false</tt:IPFilter>
<tt:ZeroConfiguration>false</tt:ZeroConfiguration>
<tt:IPVersion6>false</tt:IPVersion6>
<tt:DynDNS>false</tt:DynDNS>
</tt:Network>
<tt:System>
<tt:DiscoveryResolve>false</tt:DiscoveryResolve>
<tt:DiscoveryBye>true</tt:DiscoveryBye>
<tt:RemoteDiscovery>false</tt:RemoteDiscovery>
<tt:SystemBackup>false</tt:SystemBackup>
<tt:SystemLogging>false</tt:SystemLogging>
<tt:FirmwareUpgrade>false</tt:FirmwareUpgrade>
<tt:SupportedVersions>
<tt:Major>{ONVIF_MAJOR}</tt:Major>
<tt:Minor>{ONVIF_MINOR}</tt:Minor>
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
<tt:Media>
<tt:XAddr>{media_url}</tt:XAddr>
<tt:StreamingCapabilities>
<tt:RTPMulticast>false</tt:RTPMulticast>
<tt:RTP_TCP>true</tt:RTP_TCP>
<tt:RTP_RTSP_TCP>true</tt:RTP_RTSP_TCP>
</tt:StreamingCapabilities>
</tt:Media>
</tds:Capabilities>
</tds:GetCapabilitiesResponse>"
    ))
}

/// `GetServicesResponse` listing the device and media services.
pub fn services(ip: Ipv4Addr, port: u16, include_capability: bool) -> String {
    let device_url = DeviceIdentity::device_service_url(ip, port);
    let media_url = DeviceIdentity::media_service_url(ip, port);
    let device_caps = if include_capability {
        format!(
            "<tds:Capabilities>\n{}\n</tds:Capabilities>\n",
            device_service_capabilities_element()
        )
    } else {
        String::new()
    };
    let media_caps = if include_capability {
        format!(
            "<tds:Capabilities>\n{}\n</tds:Capabilities>\n",
            media_service_capabilities_element()
        )
    } else {
        String::new()
    };
    device_envelope(&format!(
        "<tds:GetServicesResponse>
<tds:Service>
<tds:Namespace>{DEVICE_NS}</tds:Namespace>
<tds:XAddr>{device_url}</tds:XAddr>
{device_caps}<tds:Version>
<tt:Major>{ONVIF_MAJOR}</tt:Major>
<tt:Minor>{ONVIF_MINOR}</tt:Minor>
</tds:Version>
</tds:Service>
<tds:Service>
<tds:Namespace>{MEDIA_NS}</tds:Namespace>
<tds:XAddr>{media_url}</tds:XAddr>
{media_caps}<tds:Version>
<tt:Major>{ONVIF_MAJOR}</tt:Major>
<tt:Minor>{ONVIF_MINOR}</tt:Minor>
</tds:Version>
</tds:Service>
</tds:GetServicesResponse>"
    ))
}

fn device_service_capabilities_element() -> String {
    format!(
        "<tds:Capabilities xmlns:tds=\"{DEVICE_NS}\">
<tds:Network IPFilter=\"false\" ZeroConfiguration=\"false\" IPVersion6=\"false\" DynDNS=\"false\" Dot11Configuration=\"false\" HostnameFromDHCP=\"false\" NTP=\"0\" DHCPv6=\"false\"/>
<tds:Security TLS1.0=\"false\" TLS1.1=\"false\" TLS1.2=\"false\" OnboardKeyGeneration=\"false\" AccessPolicyConfig=\"false\" DefaultAccessPolicy=\"false\" Dot1X=\"false\" RemoteUserHandling=\"false\" X.509Token=\"false\" SAMLToken=\"false\" KerberosToken=\"false\" UsernameToken=\"true\" HttpDigest=\"true\" RELToken=\"false\"/>
<tds:System DiscoveryResolve=\"false\" DiscoveryBye=\"true\" RemoteDiscovery=\"false\" SystemBackup=\"false\" SystemLogging=\"false\" FirmwareUpgrade=\"false\" HttpFirmwareUpgrade=\"false\" HttpSystemBackup=\"false\" HttpSystemLogging=\"false\" HttpSupportInformation=\"false\" StorageConfiguration=\"false\"/>
</tds:Capabilities>"
    )
}

fn media_service_capabilities_element() -> String {
    format!(
        "<trt:Capabilities xmlns:trt=\"{MEDIA_NS}\" SnapshotUri=\"true\" Rotation=\"false\" VideoSourceMode=\"false\" OSD=\"false\" TemporaryOSDText=\"false\" EXICompression=\"false\">
<trt:ProfileCapabilities MaximumNumberOfProfiles=\"1\"/>
<trt:StreamingCapabilities RTPMulticast=\"false\" RTP_TCP=\"true\" RTP_RTSP_TCP=\"true\" NonAggregateControl=\"false\" NoRTSPStreaming=\"false\"/>
</trt:Capabilities>"
    )
}

/// Device service `GetServiceCapabilitiesResponse`.
pub fn device_service_capabilities() -> String {
    device_envelope(&format!(
        "<tds:GetServiceCapabilitiesResponse>\n{}\n</tds:GetServiceCapabilitiesResponse>",
        device_service_capabilities_element()
    ))
}

/// Media service `GetServiceCapabilitiesResponse`.
pub fn media_service_capabilities() -> String {
    media_envelope(&format!(
        "<trt:GetServiceCapabilitiesResponse>\n{}\n</trt:GetServiceCapabilitiesResponse>",
        media_service_capabilities_element()
    ))
}

/// `GetSystemDateAndTimeResponse` with the current UTC time.
pub fn system_date_time() -> String {
    let now = chrono::Utc::now();
    device_envelope(&format!(
        "<tds:GetSystemDateAndTimeResponse>
<tds:SystemDateAndTime>
<tt:DateTimeType>NTP</tt:DateTimeType>
<tt:DaylightSavings>false</tt:DaylightSavings>
<tt:TimeZone>
<tt:TZ>UTC0</tt:TZ>
</tt:TimeZone>
<tt:UTCDateTime>
<tt:Time>
<tt:Hour>{h}</tt:Hour>
<tt:Minute>{mi}</tt:Minute>
<tt:Second>{s}</tt:Second>
</tt:Time>
<tt:Date>
<tt:Year>{y}</tt:Year>
<tt:Month>{mo}</tt:Month>
<tt:Day>{d}</tt:Day>
</tt:Date>
</tt:UTCDateTime>
<tt:LocalDateTime>
<tt:Time>
<tt:Hour>{h}</tt:Hour>
<tt:Minute>{mi}</tt:Minute>
<tt:Second>{s}</tt:Second>
</tt:Time>
<tt:Date>
<tt:Year>{y}</tt:Year>
<tt:Month>{mo}</tt:Month>
<tt:Day>{d}</tt:Day>
</tt:Date>
</tt:LocalDateTime>
</tds:SystemDateAndTime>
</tds:GetSystemDateAndTimeResponse>",
        h = now.hour(),
        mi = now.minute(),
        s = now.second(),
        y = now.year(),
        mo = now.month(),
        d = now.day()
    ))
}

/// `GetDeviceInformationResponse`.
pub fn device_information(identity: &DeviceIdentity) -> String {
    device_envelope(&format!(
        "<tds:GetDeviceInformationResponse>
<tds:Manufacturer>{}</tds:Manufacturer>
<tds:Model>{}</tds:Model>
<tds:FirmwareVersion>{}</tds:FirmwareVersion>
<tds:SerialNumber>{}</tds:SerialNumber>
<tds:HardwareId>{}</tds:HardwareId>
</tds:GetDeviceInformationResponse>",
        xml_escape(identity.manufacturer()),
        xml_escape(&identity.name),
        xml_escape(&identity.firmware_version),
        xml_escape(&identity.serial_number),
        xml_escape(identity.hardware_id())
    ))
}

/// `GetHostnameResponse`.
pub fn hostname(identity: &DeviceIdentity) -> String {
    let name: String = identity
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    device_envelope(&format!(
        "<tds:GetHostnameResponse>
<tds:HostnameInformation>
<tt:FromDHCP>false</tt:FromDHCP>
<tt:Name>{}</tt:Name>
</tds:HostnameInformation>
</tds:GetHostnameResponse>",
        xml_escape(name.trim_matches('-'))
    ))
}

/// `GetScopesResponse` mirroring the discovery scopes.
pub fn scopes(identity: &DeviceIdentity) -> String {
    let items: String = identity
        .scopes()
        .split_whitespace()
        .map(|scope| {
            format!(
                "<tds:Scopes>\n<tt:ScopeDef>Fixed</tt:ScopeDef>\n<tt:ScopeItem>{}</tt:ScopeItem>\n</tds:Scopes>\n",
                xml_escape(scope)
            )
        })
        .collect();
    device_envelope(&format!(
        "<tds:GetScopesResponse>\n{items}</tds:GetScopesResponse>"
    ))
}

/// `GetWsdlUrlResponse`.
pub fn wsdl_url() -> String {
    device_envelope(
        "<tds:GetWsdlUrlResponse>\n<tds:WsdlUrl>http://www.onvif.org/</tds:WsdlUrl>\n</tds:GetWsdlUrlResponse>",
    )
}

fn video_source_configuration_element(stream: &StreamInfo) -> String {
    format!(
        "<tt:Name>{VIDEO_SOURCE_CONFIG_TOKEN}</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:SourceToken>{VIDEO_SOURCE_TOKEN}</tt:SourceToken>
<tt:Bounds x=\"0\" y=\"0\" width=\"{}\" height=\"{}\"/>",
        stream.width, stream.height
    )
}

fn video_encoder_configuration_element(stream: &StreamInfo) -> String {
    let codec_block = match (stream.encoding, &stream.h264_profile) {
        (VideoEncoding::H264, Some(profile)) => format!(
            "<tt:H264>\n<tt:GovLength>{}</tt:GovLength>\n<tt:H264Profile>{}</tt:H264Profile>\n</tt:H264>\n",
            stream.framerate.max(1),
            xml_escape(profile)
        ),
        (VideoEncoding::H264, None) => format!(
            "<tt:H264>\n<tt:GovLength>{}</tt:GovLength>\n<tt:H264Profile>Main</tt:H264Profile>\n</tt:H264>\n",
            stream.framerate.max(1)
        ),
        _ => String::new(),
    };
    format!(
        "<tt:Name>{VIDEO_ENCODER_CONFIG_TOKEN}</tt:Name>
<tt:UseCount>1</tt:UseCount>
<tt:Encoding>{}</tt:Encoding>
<tt:Resolution>
<tt:Width>{}</tt:Width>
<tt:Height>{}</tt:Height>
</tt:Resolution>
<tt:Quality>5</tt:Quality>
<tt:RateControl>
<tt:FrameRateLimit>{}</tt:FrameRateLimit>
<tt:EncodingInterval>1</tt:EncodingInterval>
<tt:BitrateLimit>{}</tt:BitrateLimit>
</tt:RateControl>
{codec_block}<tt:Multicast>
<tt:Address>
<tt:Type>IPv4</tt:Type>
<tt:IPv4Address>0.0.0.0</tt:IPv4Address>
</tt:Address>
<tt:Port>0</tt:Port>
<tt:TTL>1</tt:TTL>
<tt:AutoStart>false</tt:AutoStart>
</tt:Multicast>
<tt:SessionTimeout>PT60S</tt:SessionTimeout>",
        stream.encoding.as_onvif(),
        stream.width,
        stream.height,
        stream.framerate,
        stream.bitrate_kbps
    )
}

/// `GetProfilesResponse` with the single fixed profile.
pub fn profiles(stream: &StreamInfo) -> String {
    media_envelope(&format!(
        "<trt:GetProfilesResponse>\n{}\n</trt:GetProfilesResponse>",
        profile_element("trt:Profiles", stream)
    ))
}

/// `GetProfileResponse` for the single fixed profile.
pub fn profile(stream: &StreamInfo) -> String {
    media_envelope(&format!(
        "<trt:GetProfileResponse>\n{}\n</trt:GetProfileResponse>",
        profile_element("trt:Profile", stream)
    ))
}

fn profile_element(tag: &str, stream: &StreamInfo) -> String {
    format!(
        "<{tag} token=\"{PROFILE_TOKEN}\" fixed=\"true\">
<tt:Name>{PROFILE_NAME}</tt:Name>
<tt:VideoSourceConfiguration token=\"{VIDEO_SOURCE_CONFIG_TOKEN}\">
{}
</tt:VideoSourceConfiguration>
<tt:VideoEncoderConfiguration token=\"{VIDEO_ENCODER_CONFIG_TOKEN}\">
{}
</tt:VideoEncoderConfiguration>
</{tag}>",
        video_source_configuration_element(stream),
        video_encoder_configuration_element(stream)
    )
}

fn media_uri(uri: &str) -> String {
    format!(
        "<trt:MediaUri>
<tt:Uri>{}</tt:Uri>
<tt:InvalidAfterConnect>false</tt:InvalidAfterConnect>
<tt:InvalidAfterReboot>false</tt:InvalidAfterReboot>
<tt:Timeout>PT0S</tt:Timeout>
</trt:MediaUri>",
        xml_escape(uri)
    )
}

/// `GetStreamUriResponse`.
pub fn stream_uri(rtsp_url: &str) -> String {
    media_envelope(&format!(
        "<trt:GetStreamUriResponse>\n{}\n</trt:GetStreamUriResponse>",
        media_uri(rtsp_url)
    ))
}

/// `GetSnapshotUriResponse`.
pub fn snapshot_uri(ip: Ipv4Addr, port: u16) -> String {
    media_envelope(&format!(
        "<trt:GetSnapshotUriResponse>\n{}\n</trt:GetSnapshotUriResponse>",
        media_uri(&DeviceIdentity::snapshot_url(ip, port))
    ))
}

/// `GetVideoSourcesResponse`.
pub fn video_sources(stream: &StreamInfo) -> String {
    media_envelope(&format!(
        "<trt:GetVideoSourcesResponse>
<trt:VideoSources token=\"{VIDEO_SOURCE_TOKEN}\">
<tt:Framerate>{}</tt:Framerate>
<tt:Resolution>
<tt:Width>{}</tt:Width>
<tt:Height>{}</tt:Height>
</tt:Resolution>
</trt:VideoSources>
</trt:GetVideoSourcesResponse>",
        stream.framerate, stream.width, stream.height
    ))
}

/// `GetVideoSourceConfigurationsResponse`.
pub fn video_source_configurations(stream: &StreamInfo) -> String {
    media_envelope(&format!(
        "<trt:GetVideoSourceConfigurationsResponse>
<trt:Configurations token=\"{VIDEO_SOURCE_CONFIG_TOKEN}\">
{}
</trt:Configurations>
</trt:GetVideoSourceConfigurationsResponse>",
        video_source_configuration_element(stream)
    ))
}

/// `GetVideoSourceConfigurationResponse`.
pub fn video_source_configuration(stream: &StreamInfo) -> String {
    media_envelope(&format!(
        "<trt:GetVideoSourceConfigurationResponse>
<trt:Configuration token=\"{VIDEO_SOURCE_CONFIG_TOKEN}\">
{}
</trt:Configuration>
</trt:GetVideoSourceConfigurationResponse>",
        video_source_configuration_element(stream)
    ))
}

/// `GetVideoEncoderConfigurationsResponse`.
pub fn video_encoder_configurations(stream: &StreamInfo) -> String {
    media_envelope(&format!(
        "<trt:GetVideoEncoderConfigurationsResponse>
<trt:Configurations token=\"{VIDEO_ENCODER_CONFIG_TOKEN}\">
{}
</trt:Configurations>
</trt:GetVideoEncoderConfigurationsResponse>",
        video_encoder_configuration_element(stream)
    ))
}

/// `GetVideoEncoderConfigurationResponse`.
pub fn video_encoder_configuration(stream: &StreamInfo) -> String {
    media_envelope(&format!(
        "<trt:GetVideoEncoderConfigurationResponse>
<trt:Configuration token=\"{VIDEO_ENCODER_CONFIG_TOKEN}\">
{}
</trt:Configuration>
</trt:GetVideoEncoderConfigurationResponse>",
        video_encoder_configuration_element(stream)
    ))
}

/// `GetVideoEncoderConfigurationOptionsResponse` describing the fixed stream.
pub fn video_encoder_configuration_options(stream: &StreamInfo) -> String {
    let codec_options = match stream.encoding {
        VideoEncoding::H264 => format!(
            "<tt:H264>
<tt:ResolutionsAvailable>
<tt:Width>{w}</tt:Width>
<tt:Height>{h}</tt:Height>
</tt:ResolutionsAvailable>
<tt:GovLengthRange>
<tt:Min>1</tt:Min>
<tt:Max>{fps}</tt:Max>
</tt:GovLengthRange>
<tt:FrameRateRange>
<tt:Min>1</tt:Min>
<tt:Max>{fps}</tt:Max>
</tt:FrameRateRange>
<tt:EncodingIntervalRange>
<tt:Min>1</tt:Min>
<tt:Max>1</tt:Max>
</tt:EncodingIntervalRange>
<tt:H264ProfilesSupported>{profile}</tt:H264ProfilesSupported>
</tt:H264>\n",
            w = stream.width,
            h = stream.height,
            fps = stream.framerate.max(1),
            profile = xml_escape(stream.h264_profile.as_deref().unwrap_or("Main"))
        ),
        _ => String::new(),
    };
    media_envelope(&format!(
        "<trt:GetVideoEncoderConfigurationOptionsResponse>
<trt:Options>
<tt:QualityRange>
<tt:Min>1</tt:Min>
<tt:Max>5</tt:Max>
</tt:QualityRange>
{codec_options}<tt:Extension>
<tt:H264>
<tt:BitrateRange>
<tt:Min>{kbps}</tt:Min>
<tt:Max>{kbps}</tt:Max>
</tt:BitrateRange>
</tt:H264>
</tt:Extension>
</trt:Options>
</trt:GetVideoEncoderConfigurationOptionsResponse>",
        kbps = stream.bitrate_kbps
    ))
}

/// `GetAudioSourceConfigurationsResponse` (no audio is exposed).
pub fn audio_source_configurations() -> String {
    media_envelope("<trt:GetAudioSourceConfigurationsResponse/>")
}

/// `GetAudioEncoderConfigurationsResponse` (no audio is exposed).
pub fn audio_encoder_configurations() -> String {
    media_envelope("<trt:GetAudioEncoderConfigurationsResponse/>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;

    fn ip() -> Ipv4Addr {
        Ipv4Addr::new(192, 0, 2, 10)
    }

    fn identity() -> DeviceIdentity {
        DeviceIdentity::new("Test <Cam> & Co")
    }

    fn parse(xml: &str) -> Document<'_> {
        Document::parse(xml).unwrap_or_else(|e| panic!("{e}\n{xml}"))
    }

    fn text_of<'a>(doc: &'a Document<'a>, name: &str) -> Option<&'a str> {
        doc.descendants()
            .find(|n| n.is_element() && n.tag_name().name() == name)
            .and_then(|n| n.text())
    }

    #[test]
    fn every_response_is_well_formed() {
        let stream = StreamInfo::default();
        let id = identity();
        let all = [
            capabilities(ip(), 8080),
            services(ip(), 8080, true),
            services(ip(), 8080, false),
            device_service_capabilities(),
            media_service_capabilities(),
            system_date_time(),
            device_information(&id),
            hostname(&id),
            scopes(&id),
            wsdl_url(),
            profiles(&stream),
            profile(&stream),
            stream_uri("rtsp://192.0.2.10:8554/stream"),
            snapshot_uri(ip(), 8080),
            video_sources(&stream),
            video_source_configurations(&stream),
            video_source_configuration(&stream),
            video_encoder_configurations(&stream),
            video_encoder_configuration(&stream),
            video_encoder_configuration_options(&stream),
            audio_source_configurations(),
            audio_encoder_configurations(),
        ];
        for xml in &all {
            parse(xml);
        }
    }

    #[test]
    fn service_addresses_point_at_the_right_paths() {
        let xml = capabilities(ip(), 8080);
        assert!(xml.contains("<tt:XAddr>http://192.0.2.10:8080/onvif/device_service</tt:XAddr>"));
        assert!(xml.contains("<tt:XAddr>http://192.0.2.10:8080/onvif/media_service</tt:XAddr>"));
        let xml = services(ip(), 8080, true);
        assert!(xml.contains("<tds:XAddr>http://192.0.2.10:8080/onvif/media_service</tds:XAddr>"));
        assert!(xml.contains("SnapshotUri=\"true\""));
        assert!(xml.contains("HttpDigest=\"true\""));
    }

    #[test]
    fn stream_uri_includes_required_media_uri_fields() {
        let doc_xml = stream_uri("rtsp://192.0.2.10:8554/a&b");
        let doc = parse(&doc_xml);
        assert_eq!(text_of(&doc, "Uri"), Some("rtsp://192.0.2.10:8554/a&b"));
        assert_eq!(text_of(&doc, "InvalidAfterConnect"), Some("false"));
        assert_eq!(text_of(&doc, "InvalidAfterReboot"), Some("false"));
        assert_eq!(text_of(&doc, "Timeout"), Some("PT0S"));
        let snap_xml = snapshot_uri(ip(), 8080);
        let snap = parse(&snap_xml);
        assert_eq!(
            text_of(&snap, "Uri"),
            Some("http://192.0.2.10:8080/snapshot.jpg")
        );
    }

    #[test]
    fn profile_reflects_probed_stream() {
        let stream = StreamInfo {
            encoding: VideoEncoding::H264,
            width: 1280,
            height: 720,
            framerate: 30,
            bitrate_kbps: 2500,
            h264_profile: Some("High".to_string()),
            probed: true,
        };
        let xml = profiles(&stream);
        let doc = parse(&xml);
        assert_eq!(text_of(&doc, "Width"), Some("1280"));
        assert_eq!(text_of(&doc, "Height"), Some("720"));
        assert_eq!(text_of(&doc, "FrameRateLimit"), Some("30"));
        assert_eq!(text_of(&doc, "BitrateLimit"), Some("2500"));
        assert_eq!(text_of(&doc, "H264Profile"), Some("High"));
        assert_eq!(text_of(&doc, "Encoding"), Some("H264"));
        assert!(xml.contains(&format!("token=\"{PROFILE_TOKEN}\"")));

        // Encoder configuration and profile agree.
        let enc_xml = video_encoder_configurations(&stream);
        let enc = parse(&enc_xml);
        assert_eq!(text_of(&enc, "H264Profile"), Some("High"));
        assert_eq!(text_of(&enc, "Width"), Some("1280"));
    }

    #[test]
    fn h265_streams_do_not_emit_h264_block() {
        let stream = StreamInfo {
            encoding: VideoEncoding::H265,
            h264_profile: None,
            ..StreamInfo::default()
        };
        let xml = profiles(&stream);
        assert!(xml.contains("<tt:Encoding>H265</tt:Encoding>"));
        assert!(!xml.contains("<tt:H264>"));
    }

    #[test]
    fn device_information_is_escaped() {
        let xml = device_information(&identity());
        let doc = parse(&xml);
        assert_eq!(text_of(&doc, "Model"), Some("Test <Cam> & Co"));
        assert_eq!(
            text_of(&doc, "FirmwareVersion"),
            Some(env!("CARGO_PKG_VERSION"))
        );
        let host_xml = hostname(&identity());
        let host = parse(&host_xml);
        assert_eq!(text_of(&host, "Name"), Some("Test--Cam----Co"));
    }
}
