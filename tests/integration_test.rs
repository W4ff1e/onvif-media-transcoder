//! End-to-end tests over a real TCP socket against the ONVIF HTTP service.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use clap::Parser;
use onvif_media_transcoder::config::Config;
use onvif_media_transcoder::identity::DeviceIdentity;
use onvif_media_transcoder::onvif::{OnvifService, ServerHandle};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

fn start_server() -> (ServerHandle, SocketAddr) {
    let config = Config::try_parse_from([
        "test",
        "--rtsp-stream-url",
        "rtsp://127.0.0.1:8554/stream",
        "--onvif-port",
        "0",
    ])
    .expect("config parses");
    let identity = DeviceIdentity::new(&config.device_name);
    let service = Arc::new(OnvifService::new(config, identity));
    let handle = service.serve("127.0.0.1:0", 2).expect("server starts");
    let addr = handle.local_addr().expect("bound address");
    (handle, addr)
}

/// Sends raw bytes in `chunks` with a pause between them and returns the
/// full HTTP response.
fn raw_request(addr: SocketAddr, chunks: &[&[u8]], pause: Duration) -> String {
    let mut stream = TcpStream::connect(addr).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    for (i, chunk) in chunks.iter().enumerate() {
        stream.write_all(chunk).unwrap();
        stream.flush().unwrap();
        if i + 1 < chunks.len() {
            std::thread::sleep(pause);
        }
    }
    // Ask the server to close after this exchange so read_to_end returns.
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    String::from_utf8_lossy(&response).into_owned()
}

fn post(addr: SocketAddr, path: &str, headers: &str, body: &str) -> String {
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: test\r\nConnection: close\r\nContent-Type: application/soap+xml\r\nContent-Length: {}\r\n{headers}\r\n{body}",
        body.len()
    );
    raw_request(addr, &[request.as_bytes()], Duration::ZERO)
}

fn envelope(operation: &str) -> String {
    format!(
        r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body><{operation} xmlns="http://www.onvif.org/ver10/media/wsdl"/></s:Body></s:Envelope>"#
    )
}

fn status_line(response: &str) -> &str {
    response.lines().next().unwrap_or("")
}

#[test]
fn get_capabilities_over_tcp() {
    let (server, addr) = start_server();
    let response = post(
        addr,
        "/onvif/device_service",
        "",
        &envelope("GetCapabilities"),
    );
    assert!(status_line(&response).contains("200"), "{response}");
    assert!(response.contains("GetCapabilitiesResponse"));
    assert!(response.contains("Content-Type: application/soap+xml"));
    server.shutdown();
}

#[test]
fn headers_and_body_in_separate_writes_are_handled() {
    let (server, addr) = start_server();
    let body = envelope("GetCapabilities");
    let head = format!(
        "POST /onvif/device_service HTTP/1.1\r\nHost: test\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let response = raw_request(
        addr,
        &[head.as_bytes(), body.as_bytes()],
        Duration::from_millis(300),
    );
    assert!(status_line(&response).contains("200"), "{response}");
    assert!(response.contains("GetCapabilitiesResponse"));
    server.shutdown();
}

#[test]
fn large_bodies_are_read_completely() {
    let (server, addr) = start_server();
    let padding = format!("<!--{}-->", "x".repeat(20_000));
    let body = format!(
        r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">{padding}<s:Body><GetCapabilities/></s:Body></s:Envelope>"#
    );
    let response = post(addr, "/onvif/device_service", "", &body);
    assert!(
        status_line(&response).contains("200"),
        "{}",
        status_line(&response)
    );
    server.shutdown();
}

#[test]
fn oversized_bodies_are_rejected() {
    let (server, addr) = start_server();
    let body = "x".repeat(onvif_media_transcoder::onvif::MAX_BODY_BYTES + 10);
    let response = post(addr, "/onvif/device_service", "", &body);
    assert!(
        status_line(&response).contains("413"),
        "{}",
        status_line(&response)
    );
    server.shutdown();
}

#[test]
fn protected_operation_challenge_then_basic_auth() {
    let (server, addr) = start_server();
    let body = envelope("GetProfiles");

    let response = post(addr, "/onvif/media_service", "", &body);
    assert!(status_line(&response).contains("401"), "{response}");
    assert!(response.contains("WWW-Authenticate: Digest"));
    assert!(response.contains("WWW-Authenticate: Basic"));
    assert!(response.contains("ter:NotAuthorized"));

    let auth = format!(
        "Authorization: Basic {}\r\n",
        BASE64.encode("admin:onvif-rust")
    );
    let response = post(addr, "/onvif/media_service", &auth, &body);
    assert!(status_line(&response).contains("200"), "{response}");
    assert!(response.contains("GetProfilesResponse"));
    server.shutdown();
}

#[test]
fn digest_auth_round_trip() {
    let (server, addr) = start_server();
    let body = envelope("GetProfiles");
    let path = "/onvif/media_service";

    let challenge = post(addr, path, "", &body);
    let nonce = challenge
        .lines()
        .find(|l| l.starts_with("WWW-Authenticate: Digest"))
        .and_then(|l| l.split("nonce=\"").nth(1))
        .and_then(|l| l.split('"').next())
        .expect("nonce in challenge")
        .to_string();

    let md5 = |s: &str| format!("{:x}", md5::compute(s.as_bytes()));
    let ha1 = md5("admin:ONVIF:onvif-rust");
    let ha2 = md5(&format!("POST:{path}"));
    let response_hash = md5(&format!("{ha1}:{nonce}:00000001:cafe:auth:{ha2}"));
    let auth = format!(
        "Authorization: Digest username=\"admin\", realm=\"ONVIF\", nonce=\"{nonce}\", uri=\"{path}\", qop=auth, nc=00000001, cnonce=\"cafe\", response=\"{response_hash}\", algorithm=MD5\r\n"
    );
    let response = post(addr, path, &auth, &body);
    assert!(status_line(&response).contains("200"), "{response}");

    // Same Authorization header again is a replay and must be refused.
    let response = post(addr, path, &auth, &body);
    assert!(status_line(&response).contains("401"), "{response}");
    server.shutdown();
}

#[test]
fn health_endpoint_and_snapshot_auth() {
    let (server, addr) = start_server();
    let response = raw_request(
        addr,
        &[b"GET / HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n"],
        Duration::ZERO,
    );
    assert!(status_line(&response).contains("200"), "{response}");

    let response = raw_request(
        addr,
        &[b"GET /snapshot.jpg HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n"],
        Duration::ZERO,
    );
    assert!(status_line(&response).contains("401"), "{response}");
    server.shutdown();
}

#[test]
fn ws_security_password_digest_over_tcp() {
    use sha1::Digest as _;
    let (server, addr) = start_server();

    let created = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let nonce_bytes = b"integration-nonce";
    let mut hasher = sha1::Sha1::new();
    hasher.update(nonce_bytes);
    hasher.update(created.as_bytes());
    hasher.update(b"onvif-rust");
    let digest = BASE64.encode(hasher.finalize());
    let nonce = BASE64.encode(nonce_bytes);

    let body = format!(
        r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd" xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"><s:Header><wsse:Security><wsse:UsernameToken><wsse:Username>admin</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest">{digest}</wsse:Password><wsse:Nonce>{nonce}</wsse:Nonce><wsu:Created>{created}</wsu:Created></wsse:UsernameToken></wsse:Security></s:Header><s:Body><GetVideoEncoderConfigurations xmlns="http://www.onvif.org/ver10/media/wsdl"/></s:Body></s:Envelope>"#
    );
    let response = post(addr, "/onvif/media_service", "", &body);
    assert!(status_line(&response).contains("200"), "{response}");
    assert!(response.contains("GetVideoEncoderConfigurationsResponse"));

    // Tampering with the digest is refused.
    let tampered = body.replace(&digest, "AAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    let response = post(addr, "/onvif/media_service", "", &tampered);
    assert!(status_line(&response).contains("401"), "{response}");
    server.shutdown();
}

#[test]
fn public_operation_names_in_headers_do_not_unlock_protected_operations() {
    let (server, addr) = start_server();
    let body = envelope("GetStreamUri");
    let response = post(
        addr,
        "/onvif/media_service",
        "X-Note: GetDeviceInformation GetCapabilities\r\n",
        &body,
    );
    assert!(status_line(&response).contains("401"), "{response}");
    assert!(!response.contains("<tt:Uri>"));
    server.shutdown();
}

#[test]
fn keep_alive_serves_several_requests_on_one_connection() {
    let (server, addr) = start_server();
    let body = envelope("GetCapabilities");
    let one = format!(
        "POST /onvif/device_service HTTP/1.1\r\nHost: test\r\nContent-Type: application/soap+xml\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    for _ in 0..3 {
        stream.write_all(one.as_bytes()).unwrap();
        let mut buf = vec![0u8; 65536];
        let mut got = String::new();
        while !got.contains("</soap:Envelope>") {
            let n = stream.read(&mut buf).expect("response chunk");
            assert!(n > 0, "connection closed early");
            got.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
        assert!(got.starts_with("HTTP/1.1 200"), "{got}");
        assert!(got.contains("GetCapabilitiesResponse"));
    }
    server.shutdown();
}

#[test]
fn snapshot_with_credentials_fails_cleanly_without_a_stream() {
    let (server, addr) = start_server();
    let request = format!(
        "GET /snapshot.jpg HTTP/1.1\r\nHost: test\r\nConnection: close\r\nAuthorization: Basic {}\r\n\r\n",
        BASE64.encode("admin:onvif-rust")
    );
    let response = raw_request(addr, &[request.as_bytes()], Duration::ZERO);
    let status = status_line(&response);
    // 502 when ffmpeg exists but there is no stream, 503 when ffmpeg is missing.
    assert!(status.contains("502") || status.contains("503"), "{status}");
    server.shutdown();
}

#[test]
fn unsupported_and_public_operations_over_tcp() {
    let (server, addr) = start_server();

    let response = post(
        addr,
        "/onvif/device_service",
        "",
        &envelope("GetSystemDateAndTime"),
    );
    assert!(status_line(&response).contains("200"), "{response}");
    let year = chrono::Utc::now().format("%Y").to_string();
    assert!(response.contains(&format!("<tt:Year>{year}</tt:Year>")));

    let auth = format!(
        "Authorization: Basic {}\r\n",
        BASE64.encode("admin:onvif-rust")
    );
    let response = post(
        addr,
        "/onvif/device_service",
        &auth,
        &envelope("SetHostname"),
    );
    assert!(status_line(&response).contains("400"), "{response}");
    assert!(response.contains("ter:ActionNotSupported"));
    server.shutdown();
}
