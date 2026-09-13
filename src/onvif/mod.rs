//! ONVIF SOAP service: HTTP transport, authentication and operation dispatch.

pub mod auth;
pub mod process;
pub mod responses;
pub mod snapshot;
pub mod soap;
pub mod stream_info;

use crate::config::Config;
use crate::identity::DeviceIdentity;
use auth::{AuthResult, Authenticator};
use soap::{FaultCode, SoapRequest, soap_fault};
use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use stream_info::StreamProbe;
use tracing::{debug, info, warn};

/// Largest accepted request body. ONVIF requests are a few kilobytes at most.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

const SOAP_CONTENT_TYPE: &str = "application/soap+xml; charset=utf-8";
const TEXT_CONTENT_TYPE: &str = "text/plain; charset=utf-8";

/// Operations that ONVIF classifies as PRE_AUTH and that may be called
/// without credentials.
const PUBLIC_OPERATIONS: &[&str] = &[
    "GetCapabilities",
    "GetServices",
    "GetServiceCapabilities",
    "GetSystemDateAndTime",
    "GetWsdlUrl",
    "GetHostname",
];

/// A fully built HTTP response, independent of the transport library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnvifResponse {
    pub status: u16,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl OnvifResponse {
    fn soap(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: SOAP_CONTENT_TYPE.to_string(),
            headers: Vec::new(),
            body: body.into_bytes(),
        }
    }

    fn text(status: u16, body: &str) -> Self {
        Self {
            status,
            content_type: TEXT_CONTENT_TYPE.to_string(),
            headers: Vec::new(),
            body: body.as_bytes().to_vec(),
        }
    }

    fn with_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = headers;
        self
    }

    /// Body as UTF-8 text, for logging and tests.
    pub fn body_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }
}

/// The ONVIF device and media service.
pub struct OnvifService {
    config: Config,
    identity: DeviceIdentity,
    auth: Authenticator,
    stream: Arc<StreamProbe>,
}

impl OnvifService {
    pub fn new(config: Config, identity: DeviceIdentity) -> Self {
        let auth = Authenticator::new(&config.onvif_username, &config.onvif_password);
        Self {
            config,
            identity,
            auth,
            stream: StreamProbe::new(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The stream description used in media responses.
    pub fn stream(&self) -> &Arc<StreamProbe> {
        &self.stream
    }

    /// Starts probing the RTSP stream in the background so that profiles
    /// report the real codec, resolution and frame rate.
    pub fn start_stream_probe(&self, stop: Arc<AtomicBool>) {
        self.stream
            .start_background_probe(self.config.internal_rtsp_url(), stop);
    }

    /// Handles one HTTP request and produces a response. Pure with respect
    /// to the network so it can be tested without sockets.
    pub fn handle(
        &self,
        method: &str,
        path: &str,
        authorization: Option<&str>,
        body: &[u8],
    ) -> OnvifResponse {
        let path = path.split('?').next().unwrap_or(path);
        match method {
            "GET" | "HEAD" if path == "/snapshot.jpg" => {
                self.handle_snapshot(method, path, authorization)
            }
            "GET" | "HEAD" => OnvifResponse::text(200, "ONVIF Media Transcoder\n"),
            "POST" => self.handle_soap(method, path, authorization, body),
            _ => OnvifResponse::text(405, "Method Not Allowed\n"),
        }
    }

    fn unauthorized(&self, result: AuthResult) -> OnvifResponse {
        let headers = self
            .auth
            .challenge_headers(result == AuthResult::Stale)
            .into_iter()
            .map(|v| ("WWW-Authenticate".to_string(), v))
            .collect();
        OnvifResponse::soap(
            401,
            soap_fault(
                FaultCode::Sender,
                "NotAuthorized",
                "The action requested requires authorization and the sender is not authorized",
            ),
        )
        .with_headers(headers)
    }

    /// Malformed or empty POST bodies. Digest clients (curl, many NVRs) probe
    /// with an empty body first and expect a 401 challenge, so unauthenticated
    /// callers get the challenge and authenticated ones get a 400 fault.
    fn malformed(
        &self,
        method: &str,
        path: &str,
        authorization: Option<&str>,
        reason: &str,
    ) -> OnvifResponse {
        let result = self.auth.authenticate(method, path, authorization, None);
        if result != AuthResult::Authenticated {
            return self.unauthorized(result);
        }
        OnvifResponse::soap(400, soap_fault(FaultCode::Sender, "WellFormed", reason))
    }

    fn handle_snapshot(
        &self,
        method: &str,
        path: &str,
        authorization: Option<&str>,
    ) -> OnvifResponse {
        let result = self.auth.authenticate(method, path, authorization, None);
        if result != AuthResult::Authenticated {
            debug!(?result, "snapshot request rejected");
            return self.unauthorized(result);
        }

        match snapshot::capture_jpeg(&self.config.internal_rtsp_url()) {
            Ok(image) => OnvifResponse {
                status: 200,
                content_type: "image/jpeg".to_string(),
                headers: vec![("Cache-Control".to_string(), "no-store".to_string())],
                body: if method == "HEAD" { Vec::new() } else { image },
            },
            Err(snapshot::SnapshotError::Unavailable(e)) => {
                warn!(error = %e, "snapshot unavailable");
                OnvifResponse::text(503, "Snapshot generation unavailable\n")
            }
            Err(e) => {
                warn!(error = %e, "snapshot failed");
                OnvifResponse::text(502, "Failed to generate snapshot\n")
            }
        }
    }

    fn handle_soap(
        &self,
        method: &str,
        path: &str,
        authorization: Option<&str>,
        body: &[u8],
    ) -> OnvifResponse {
        let text = String::from_utf8_lossy(body);
        let request = match SoapRequest::parse(&text) {
            Ok(request) => request,
            Err(e) => {
                debug!(error = %e, "rejecting malformed SOAP request");
                return self.malformed(method, path, authorization, "Malformed SOAP request");
            }
        };
        let Some(action) = request.action().map(str::to_string) else {
            return self.malformed(method, path, authorization, "SOAP Body has no operation");
        };
        let action = action.as_str();

        if !PUBLIC_OPERATIONS.contains(&action) {
            let token = request.username_token();
            let result = self
                .auth
                .authenticate(method, path, authorization, token.as_ref());
            if result != AuthResult::Authenticated {
                debug!(action, ?result, "authentication failed");
                return self.unauthorized(result);
            }
        }

        match self.dispatch(action, path, &request) {
            Ok(body) => {
                debug!(action, "handled operation");
                OnvifResponse::soap(200, body)
            }
            Err(fault) => fault,
        }
    }

    /// Produces the SOAP response body for an authenticated (or public)
    /// operation, or a fault response.
    fn dispatch(
        &self,
        action: &str,
        path: &str,
        request: &SoapRequest<'_>,
    ) -> Result<String, OnvifResponse> {
        let ip = self.config.container_ip;
        let port = self.config.onvif_port;
        let stream = self.stream.current();

        let body = match action {
            "GetCapabilities" => responses::capabilities(ip, port),
            "GetServices" => {
                let include = request
                    .action_parameter("IncludeCapability")
                    .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                    .unwrap_or(false);
                responses::services(ip, port, include)
            }
            "GetServiceCapabilities" => {
                if path.contains("media") {
                    responses::media_service_capabilities()
                } else {
                    responses::device_service_capabilities()
                }
            }
            "GetSystemDateAndTime" => responses::system_date_time(),
            "GetDeviceInformation" => responses::device_information(&self.identity),
            "GetHostname" => responses::hostname(&self.identity),
            "GetScopes" => responses::scopes(&self.identity),
            "GetWsdlUrl" => responses::wsdl_url(),
            "GetProfiles" => responses::profiles(&stream),
            "GetProfile" => {
                self.check_profile_token(request)?;
                responses::profile(&stream)
            }
            "GetStreamUri" => {
                self.check_profile_token(request)?;
                responses::stream_uri(&self.config.rtsp_stream_url)
            }
            "GetSnapshotUri" => {
                self.check_profile_token(request)?;
                responses::snapshot_uri(ip, port)
            }
            "GetVideoSources" => responses::video_sources(&stream),
            "GetVideoSourceConfigurations" => responses::video_source_configurations(&stream),
            "GetVideoSourceConfiguration" => {
                self.check_configuration_token(request, responses::VIDEO_SOURCE_CONFIG_TOKEN)?;
                responses::video_source_configuration(&stream)
            }
            "GetVideoEncoderConfigurations" => responses::video_encoder_configurations(&stream),
            "GetVideoEncoderConfiguration" => {
                self.check_configuration_token(request, responses::VIDEO_ENCODER_CONFIG_TOKEN)?;
                responses::video_encoder_configuration(&stream)
            }
            "GetVideoEncoderConfigurationOptions" => {
                responses::video_encoder_configuration_options(&stream)
            }
            "GetAudioSourceConfigurations" => responses::audio_source_configurations(),
            "GetAudioEncoderConfigurations" => responses::audio_encoder_configurations(),
            other => {
                info!(action = other, "unsupported ONVIF operation");
                return Err(OnvifResponse::soap(
                    400,
                    soap_fault(
                        FaultCode::Sender,
                        "ActionNotSupported",
                        &format!("The operation '{other}' is not supported by this device"),
                    ),
                ));
            }
        };
        Ok(body)
    }

    /// Accepts a missing ProfileToken (lenient clients) or our single token.
    fn check_profile_token(&self, request: &SoapRequest<'_>) -> Result<(), OnvifResponse> {
        match request.action_parameter("ProfileToken") {
            Some(token) if token != responses::PROFILE_TOKEN => {
                debug!(token, "unknown profile token");
                Err(OnvifResponse::soap(
                    400,
                    soap_fault(
                        FaultCode::Sender,
                        "InvalidArgVal",
                        &format!("The requested profile token '{token}' does not exist"),
                    ),
                ))
            }
            _ => Ok(()),
        }
    }

    fn check_configuration_token(
        &self,
        request: &SoapRequest<'_>,
        expected: &str,
    ) -> Result<(), OnvifResponse> {
        match request.action_parameter("ConfigurationToken") {
            Some(token) if token != expected => {
                debug!(token, "unknown configuration token");
                Err(OnvifResponse::soap(
                    400,
                    soap_fault(
                        FaultCode::Sender,
                        "InvalidArgVal",
                        &format!("The requested configuration token '{token}' does not exist"),
                    ),
                ))
            }
            _ => Ok(()),
        }
    }

    /// Binds `addr` and serves requests on `workers` threads until the
    /// returned handle is shut down.
    pub fn serve(
        self: Arc<Self>,
        addr: &str,
        workers: usize,
    ) -> Result<ServerHandle, Box<dyn std::error::Error + Send + Sync>> {
        let server = Arc::new(tiny_http::Server::http(addr)?);
        let stop = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::with_capacity(workers);
        for i in 0..workers.max(1) {
            let server = Arc::clone(&server);
            let service = Arc::clone(&self);
            let stop = Arc::clone(&stop);
            let thread = std::thread::Builder::new()
                .name(format!("onvif-http-{i}"))
                .spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        match server.recv_timeout(Duration::from_millis(250)) {
                            Ok(Some(request)) => service.respond(request),
                            Ok(None) => continue,
                            Err(e) => {
                                debug!(error = %e, "http worker stopping");
                                break;
                            }
                        }
                    }
                })?;
            threads.push(thread);
        }
        Ok(ServerHandle {
            server,
            stop,
            threads,
        })
    }

    fn respond(&self, mut request: tiny_http::Request) {
        let method = request.method().as_str().to_string();
        let url = request.url().to_string();
        let remote = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();
        let authorization = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Authorization"))
            .map(|h| h.value.as_str().to_string());

        if request.body_length().unwrap_or(0) > MAX_BODY_BYTES {
            let _ = request.respond(
                tiny_http::Response::from_string("Payload Too Large\n").with_status_code(413),
            );
            return;
        }

        let mut body = Vec::new();
        if let Err(e) = request
            .as_reader()
            .take(MAX_BODY_BYTES as u64 + 1)
            .read_to_end(&mut body)
        {
            debug!(error = %e, remote, "failed to read request body");
            return;
        }
        if body.len() > MAX_BODY_BYTES {
            let _ = request.respond(
                tiny_http::Response::from_string("Payload Too Large\n").with_status_code(413),
            );
            return;
        }

        let response = self.handle(&method, &url, authorization.as_deref(), &body);
        info!(remote, method, url, status = response.status, "request");

        let mut http = tiny_http::Response::from_data(response.body)
            .with_status_code(response.status)
            .with_header(header("Content-Type", &response.content_type));
        for (name, value) in &response.headers {
            http.add_header(header(name, value));
        }
        if let Err(e) = request.respond(http) {
            debug!(error = %e, remote, "failed to send response");
        }
    }
}

fn header(name: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes())
        .expect("header name and value are ASCII")
}

/// A running HTTP server.
pub struct ServerHandle {
    server: Arc<tiny_http::Server>,
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl ServerHandle {
    /// The address the server is listening on.
    pub fn local_addr(&self) -> Option<std::net::SocketAddr> {
        self.server.server_addr().to_ip()
    }

    /// Stops accepting requests and waits for worker threads to exit.
    pub fn shutdown(self) {
        self.stop.store(true, Ordering::Relaxed);
        self.server.unblock();
        for thread in self.threads {
            let _ = thread.join();
        }
    }

    /// Blocks until all worker threads exit.
    pub fn join(self) {
        for thread in self.threads {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use clap::Parser;

    fn service() -> OnvifService {
        let config =
            Config::try_parse_from(["test", "-r", "rtsp://127.0.0.1:8554/stream"]).expect("config");
        let identity = DeviceIdentity::new(&config.device_name);
        OnvifService::new(config, identity)
    }

    fn envelope(body: &str) -> Vec<u8> {
        format!(
            r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body>{body}</s:Body></s:Envelope>"#
        )
        .into_bytes()
    }

    fn basic() -> String {
        format!("Basic {}", BASE64.encode("admin:onvif-rust"))
    }

    #[test]
    fn public_operation_without_credentials() {
        let svc = service();
        let r = svc.handle(
            "POST",
            "/onvif/device_service",
            None,
            &envelope(
                "<tds:GetCapabilities xmlns:tds=\"http://www.onvif.org/ver10/device/wsdl\"/>",
            ),
        );
        assert_eq!(r.status, 200);
        assert!(r.body_str().contains("GetCapabilitiesResponse"));
        assert!(roxmltree::Document::parse(&r.body_str()).is_ok());
    }

    #[test]
    fn protected_operation_requires_credentials() {
        let svc = service();
        let body =
            envelope("<trt:GetStreamUri xmlns:trt=\"http://www.onvif.org/ver10/media/wsdl\"/>");
        let r = svc.handle("POST", "/onvif/media_service", None, &body);
        assert_eq!(r.status, 401);
        assert!(r.body_str().contains("ter:NotAuthorized"));
        let challenges: Vec<_> = r
            .headers
            .iter()
            .filter(|(k, _)| k == "WWW-Authenticate")
            .collect();
        assert_eq!(challenges.len(), 2);
        assert!(challenges[0].1.starts_with("Digest "));
        assert!(challenges[0].1.contains("qop=\"auth\""));

        let r = svc.handle("POST", "/onvif/media_service", Some(&basic()), &body);
        assert_eq!(r.status, 200);
        assert!(r.body_str().contains("rtsp://127.0.0.1:8554/stream"));
        assert!(r.body_str().contains("<tt:Timeout>PT0S</tt:Timeout>"));
    }

    #[test]
    fn public_operation_names_elsewhere_do_not_bypass_auth() {
        let svc = service();
        let body = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Header><X>GetDeviceInformation GetCapabilities</X></s:Header><s:Body><!-- GetServices --><trt:GetStreamUri xmlns:trt="http://www.onvif.org/ver10/media/wsdl"/></s:Body></s:Envelope>"#;
        let r = svc.handle("POST", "/onvif/media_service", None, body.as_bytes());
        assert_eq!(r.status, 401);
    }

    #[test]
    fn device_information_now_requires_auth() {
        let svc = service();
        let body = envelope(
            "<tds:GetDeviceInformation xmlns:tds=\"http://www.onvif.org/ver10/device/wsdl\"/>",
        );
        assert_eq!(
            svc.handle("POST", "/onvif/device_service", None, &body)
                .status,
            401
        );
        assert_eq!(
            svc.handle("POST", "/onvif/device_service", Some(&basic()), &body)
                .status,
            200
        );
    }

    #[test]
    fn ws_security_prefixed_text_password_is_accepted() {
        let svc = service();
        let body = r#"<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><soap:Header><wsse:Security><wsse:UsernameToken><wsse:Username>admin</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordText">onvif-rust</wsse:Password></wsse:UsernameToken></wsse:Security></soap:Header><soap:Body><GetProfiles/></soap:Body></soap:Envelope>"#;
        let r = svc.handle("POST", "/onvif/media_service", None, body.as_bytes());
        assert_eq!(r.status, 200, "{}", r.body_str());
        assert!(r.body_str().contains("GetProfilesResponse"));
    }

    #[test]
    fn unsupported_operation_returns_action_not_supported() {
        let svc = service();
        let r = svc.handle(
            "POST",
            "/onvif/device_service",
            Some(&basic()),
            &envelope(
                "<tds:SetSystemDateAndTime xmlns:tds=\"http://www.onvif.org/ver10/device/wsdl\"/>",
            ),
        );
        assert_eq!(r.status, 400);
        assert!(r.body_str().contains("ter:ActionNotSupported"));
    }

    #[test]
    fn profile_tokens_are_validated() {
        let svc = service();
        let ok = envelope(&format!(
            "<trt:GetStreamUri xmlns:trt=\"http://www.onvif.org/ver10/media/wsdl\"><trt:ProfileToken>{}</trt:ProfileToken></trt:GetStreamUri>",
            responses::PROFILE_TOKEN
        ));
        assert_eq!(
            svc.handle("POST", "/onvif/media_service", Some(&basic()), &ok)
                .status,
            200
        );
        let bad = envelope(
            "<trt:GetStreamUri xmlns:trt=\"http://www.onvif.org/ver10/media/wsdl\"><trt:ProfileToken>Nope</trt:ProfileToken></trt:GetStreamUri>",
        );
        let r = svc.handle("POST", "/onvif/media_service", Some(&basic()), &bad);
        assert_eq!(r.status, 400);
        assert!(r.body_str().contains("ter:InvalidArgVal"));
    }

    #[test]
    fn service_capabilities_depend_on_path() {
        let svc = service();
        let body = envelope("<GetServiceCapabilities/>");
        let dev = svc.handle("POST", "/onvif/device_service", None, &body);
        assert!(
            dev.body_str()
                .contains("tds:GetServiceCapabilitiesResponse")
        );
        let media = svc.handle("POST", "/onvif/media_service", None, &body);
        assert!(
            media
                .body_str()
                .contains("trt:GetServiceCapabilitiesResponse")
        );
    }

    #[test]
    fn malformed_requests_are_rejected() {
        let svc = service();
        // Unauthenticated malformed/empty bodies receive a Digest challenge so
        // that clients probing for the auth scheme (curl --digest) can proceed.
        let r = svc.handle("POST", "/onvif/device_service", None, b"");
        assert_eq!(r.status, 401);
        assert!(
            r.headers
                .iter()
                .any(|(k, v)| k == "WWW-Authenticate" && v.starts_with("Digest"))
        );
        let r = svc.handle("POST", "/onvif/device_service", Some(&basic()), b"<not xml");
        assert_eq!(r.status, 400);
        assert!(r.body_str().contains("ter:WellFormed"));
        let r = svc.handle("DELETE", "/", None, b"");
        assert_eq!(r.status, 405);
    }

    #[test]
    fn health_and_snapshot_paths() {
        let svc = service();
        assert_eq!(svc.handle("GET", "/", None, b"").status, 200);
        assert_eq!(
            svc.handle("GET", "/onvif/device_service", None, b"").status,
            200
        );
        // Snapshot needs credentials now.
        assert_eq!(svc.handle("GET", "/snapshot.jpg", None, b"").status, 401);
        assert_eq!(
            svc.handle("GET", "/snapshot.jpg?x=1", None, b"").status,
            401
        );
    }

    #[test]
    fn all_supported_operations_return_well_formed_xml() {
        let svc = service();
        for op in [
            "GetCapabilities",
            "GetServices",
            "GetServiceCapabilities",
            "GetSystemDateAndTime",
            "GetDeviceInformation",
            "GetHostname",
            "GetScopes",
            "GetWsdlUrl",
            "GetProfiles",
            "GetProfile",
            "GetStreamUri",
            "GetSnapshotUri",
            "GetVideoSources",
            "GetVideoSourceConfigurations",
            "GetVideoSourceConfiguration",
            "GetVideoEncoderConfigurations",
            "GetVideoEncoderConfiguration",
            "GetVideoEncoderConfigurationOptions",
            "GetAudioSourceConfigurations",
            "GetAudioEncoderConfigurations",
        ] {
            let r = svc.handle(
                "POST",
                "/onvif/device_service",
                Some(&basic()),
                &envelope(&format!("<{op}/>")),
            );
            assert_eq!(r.status, 200, "{op}");
            roxmltree::Document::parse(&r.body_str()).unwrap_or_else(|e| panic!("{op}: {e}"));
        }
    }
}
