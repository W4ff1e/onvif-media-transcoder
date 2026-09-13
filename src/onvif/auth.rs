//! Request authentication: HTTP Basic, HTTP Digest (RFC 7616, MD5) and
//! WS-Security UsernameToken (PasswordText and PasswordDigest).
//!
//! Digest nonces are issued by the server, expire after [`NONCE_LIFETIME`]
//! and track the client nonce count so that a captured `Authorization`
//! header cannot be replayed.

use crate::onvif::soap::{PasswordType, UsernameToken};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha1::Digest as _;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// How long an issued Digest nonce stays valid.
pub const NONCE_LIFETIME: Duration = Duration::from_secs(300);
/// Maximum accepted clock skew for the WS-Security `Created` timestamp.
pub const WS_SECURITY_MAX_SKEW: Duration = Duration::from_secs(300);
/// Realm advertised in HTTP challenges.
pub const REALM: &str = "ONVIF";

/// Outcome of authenticating a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthResult {
    /// Valid credentials were presented.
    Authenticated,
    /// No credentials of any supported kind were presented.
    Missing,
    /// Credentials were presented but were wrong.
    Invalid,
    /// A Digest response used an unknown or expired nonce; the client should retry.
    Stale,
}

struct NonceState {
    issued: Instant,
    last_nc: u32,
}

/// Validates credentials for HTTP and SOAP requests.
pub struct Authenticator {
    username: String,
    password: String,
    nonces: Mutex<HashMap<String, NonceState>>,
}

impl Authenticator {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            nonces: Mutex::new(HashMap::new()),
        }
    }

    /// Authenticates using, in order: HTTP Authorization header, then a
    /// WS-Security UsernameToken.
    pub fn authenticate(
        &self,
        method: &str,
        request_uri: &str,
        authorization: Option<&str>,
        token: Option<&UsernameToken>,
    ) -> AuthResult {
        if let Some(header) = authorization {
            let header = header.trim();
            if let Some(encoded) = strip_scheme(header, "Basic") {
                return self.check_basic(encoded);
            }
            if let Some(params) = strip_scheme(header, "Digest") {
                return self.check_digest(method, request_uri, params);
            }
            debug!("unsupported Authorization scheme");
        }

        if let Some(token) = token {
            return self.check_ws_security(token);
        }

        AuthResult::Missing
    }

    /// Issues a fresh nonce and returns `WWW-Authenticate` header values.
    pub fn challenge_headers(&self, stale: bool) -> Vec<String> {
        let nonce = self.issue_nonce();
        vec![
            format!(
                "Digest realm=\"{REALM}\", qop=\"auth\", algorithm=MD5, nonce=\"{nonce}\", stale={}",
                if stale { "true" } else { "false" }
            ),
            format!("Basic realm=\"{REALM}\", charset=\"UTF-8\""),
        ]
    }

    fn issue_nonce(&self) -> String {
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let mut nonces = self.nonces.lock().unwrap_or_else(|e| e.into_inner());
        nonces.retain(|_, s| s.issued.elapsed() < NONCE_LIFETIME);
        nonces.insert(
            nonce.clone(),
            NonceState {
                issued: Instant::now(),
                last_nc: 0,
            },
        );
        nonce
    }

    fn check_basic(&self, encoded: &str) -> AuthResult {
        let Ok(decoded) = BASE64.decode(encoded.trim()) else {
            return AuthResult::Invalid;
        };
        let Ok(decoded) = String::from_utf8(decoded) else {
            return AuthResult::Invalid;
        };
        let Some((user, pass)) = decoded.split_once(':') else {
            return AuthResult::Invalid;
        };
        if constant_time_eq(user.as_bytes(), self.username.as_bytes())
            & constant_time_eq(pass.as_bytes(), self.password.as_bytes())
        {
            AuthResult::Authenticated
        } else {
            debug!("basic auth rejected");
            AuthResult::Invalid
        }
    }

    fn check_digest(&self, method: &str, request_uri: &str, params: &str) -> AuthResult {
        let params = parse_auth_params(params);
        let get = |k: &str| params.get(k).map(String::as_str).unwrap_or("");

        let username = get("username");
        let realm = get("realm");
        let nonce = get("nonce");
        let uri = get("uri");
        let response = get("response");
        let qop = get("qop");
        let nc = get("nc");
        let cnonce = get("cnonce");
        let algorithm = params
            .get("algorithm")
            .map(|a| a.to_ascii_uppercase())
            .unwrap_or_else(|| "MD5".to_string());

        if username.is_empty() || nonce.is_empty() || response.is_empty() {
            return AuthResult::Invalid;
        }
        if algorithm != "MD5" && algorithm != "MD5-SESS" {
            debug!(algorithm, "unsupported digest algorithm");
            return AuthResult::Invalid;
        }
        if !uri_matches(uri, request_uri) {
            debug!(uri, request_uri, "digest uri does not match request");
            return AuthResult::Invalid;
        }

        // Verify the nonce was issued by us and is still fresh; enforce a
        // strictly increasing nonce count to defeat replay.
        {
            let mut nonces = self.nonces.lock().unwrap_or_else(|e| e.into_inner());
            let Some(state) = nonces.get_mut(nonce) else {
                debug!("digest nonce unknown");
                return AuthResult::Stale;
            };
            if state.issued.elapsed() >= NONCE_LIFETIME {
                nonces.remove(nonce);
                debug!("digest nonce expired");
                return AuthResult::Stale;
            }
            if !qop.is_empty() {
                let Ok(nc_value) = u32::from_str_radix(nc, 16) else {
                    return AuthResult::Invalid;
                };
                if nc_value <= state.last_nc {
                    warn!("digest nonce count replayed");
                    return AuthResult::Invalid;
                }
                state.last_nc = nc_value;
            }
        }

        if !constant_time_eq(username.as_bytes(), self.username.as_bytes()) {
            debug!("digest username mismatch");
            return AuthResult::Invalid;
        }

        let mut ha1 = md5_hex(&format!("{}:{realm}:{}", self.username, self.password));
        if algorithm == "MD5-SESS" {
            ha1 = md5_hex(&format!("{ha1}:{nonce}:{cnonce}"));
        }
        let ha2 = md5_hex(&format!("{method}:{uri}"));
        let expected = if qop.is_empty() {
            md5_hex(&format!("{ha1}:{nonce}:{ha2}"))
        } else if qop.eq_ignore_ascii_case("auth") {
            md5_hex(&format!("{ha1}:{nonce}:{nc}:{cnonce}:{qop}:{ha2}"))
        } else {
            debug!(qop, "unsupported digest qop");
            return AuthResult::Invalid;
        };

        if constant_time_eq(
            expected.as_bytes(),
            response.to_ascii_lowercase().as_bytes(),
        ) {
            AuthResult::Authenticated
        } else {
            debug!("digest response mismatch");
            AuthResult::Invalid
        }
    }

    fn check_ws_security(&self, token: &UsernameToken) -> AuthResult {
        if !constant_time_eq(token.username.as_bytes(), self.username.as_bytes()) {
            debug!("ws-security username mismatch");
            return AuthResult::Invalid;
        }

        match token.password_type {
            PasswordType::Text => {
                if constant_time_eq(token.password.as_bytes(), self.password.as_bytes()) {
                    AuthResult::Authenticated
                } else {
                    debug!("ws-security text password mismatch");
                    AuthResult::Invalid
                }
            }
            PasswordType::Digest => {
                let (Some(nonce), Some(created)) = (&token.nonce, &token.created) else {
                    debug!("ws-security digest without nonce or created");
                    return AuthResult::Invalid;
                };
                if !created_is_fresh(created) {
                    debug!(created, "ws-security created timestamp outside window");
                    return AuthResult::Invalid;
                }
                let Ok(nonce_bytes) = BASE64.decode(nonce) else {
                    debug!("ws-security nonce is not base64");
                    return AuthResult::Invalid;
                };
                let mut hasher = sha1::Sha1::new();
                hasher.update(&nonce_bytes);
                hasher.update(created.as_bytes());
                hasher.update(self.password.as_bytes());
                let expected = BASE64.encode(hasher.finalize());
                if constant_time_eq(expected.as_bytes(), token.password.as_bytes()) {
                    AuthResult::Authenticated
                } else {
                    debug!("ws-security digest mismatch");
                    AuthResult::Invalid
                }
            }
        }
    }
}

/// Returns the parameters after `scheme` if the header uses that scheme (case-insensitive).
fn strip_scheme<'a>(header: &'a str, scheme: &str) -> Option<&'a str> {
    let (found, rest) = header.split_once(char::is_whitespace)?;
    if found.eq_ignore_ascii_case(scheme) {
        Some(rest.trim_start())
    } else {
        None
    }
}

/// Parses `key="value", key2=value2` auth parameters, honouring quoted commas.
fn parse_auth_params(input: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let mut rest = input.trim();
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq].trim().to_ascii_lowercase();
        rest = rest[eq + 1..].trim_start();
        let value;
        if let Some(stripped) = rest.strip_prefix('"') {
            let mut end = None;
            let mut escaped = false;
            for (i, c) in stripped.char_indices() {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    end = Some(i);
                    break;
                }
            }
            let Some(end) = end else { break };
            value = stripped[..end].replace("\\\"", "\"");
            rest = stripped[end + 1..].trim_start();
        } else {
            let end = rest.find(',').unwrap_or(rest.len());
            value = rest[..end].trim().to_string();
            rest = &rest[end..];
        }
        params.insert(key, value);
        rest = rest.trim_start_matches(',').trim_start();
    }
    params
}

/// Accepts an exact path match or an absolute URI whose path matches.
fn uri_matches(digest_uri: &str, request_uri: &str) -> bool {
    if digest_uri.is_empty() {
        return false;
    }
    if digest_uri == request_uri {
        return true;
    }
    if let Some(idx) = digest_uri.find("://") {
        let after_scheme = &digest_uri[idx + 3..];
        let path = after_scheme
            .find('/')
            .map(|p| &after_scheme[p..])
            .unwrap_or("/");
        return path == request_uri;
    }
    false
}

/// Checks that an xsd:dateTime `Created` value is within the accepted skew.
fn created_is_fresh(created: &str) -> bool {
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(created) else {
        return false;
    };
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(parsed.with_timezone(&chrono::Utc));
    diff.abs()
        .to_std()
        .map(|d| d <= WS_SECURITY_MAX_SKEW)
        .unwrap_or(false)
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input.as_bytes()))
}

/// Compares two byte slices in time independent of where they differ.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auth() -> Authenticator {
        Authenticator::new("admin", "onvif-rust")
    }

    fn basic_header(user: &str, pass: &str) -> String {
        format!("Basic {}", BASE64.encode(format!("{user}:{pass}")))
    }

    #[test]
    fn basic_auth() {
        let a = auth();
        assert_eq!(
            a.authenticate(
                "POST",
                "/",
                Some(&basic_header("admin", "onvif-rust")),
                None
            ),
            AuthResult::Authenticated
        );
        assert_eq!(
            a.authenticate("POST", "/", Some(&basic_header("admin", "nope")), None),
            AuthResult::Invalid
        );
        assert_eq!(
            a.authenticate("POST", "/", Some(&basic_header("root", "onvif-rust")), None),
            AuthResult::Invalid
        );
        assert_eq!(
            a.authenticate("POST", "/", Some("Basic !!!"), None),
            AuthResult::Invalid
        );
        assert_eq!(a.authenticate("POST", "/", None, None), AuthResult::Missing);
    }

    fn nonce_from_challenge(a: &Authenticator) -> String {
        let header = a.challenge_headers(false).remove(0);
        let params = parse_auth_params(strip_scheme(&header, "Digest").unwrap());
        params["nonce"].clone()
    }

    fn digest_header(
        nonce: &str,
        nc: &str,
        cnonce: &str,
        method: &str,
        uri: &str,
        pass: &str,
    ) -> String {
        let ha1 = md5_hex(&format!("admin:{REALM}:{pass}"));
        let ha2 = md5_hex(&format!("{method}:{uri}"));
        let response = md5_hex(&format!("{ha1}:{nonce}:{nc}:{cnonce}:auth:{ha2}"));
        format!(
            "Digest username=\"admin\", realm=\"{REALM}\", nonce=\"{nonce}\", uri=\"{uri}\", qop=auth, nc={nc}, cnonce=\"{cnonce}\", response=\"{response}\", algorithm=MD5"
        )
    }

    #[test]
    fn digest_auth_with_qop_auth() {
        let a = auth();
        let nonce = nonce_from_challenge(&a);
        let uri = "/onvif/media_service";
        let header = digest_header(&nonce, "00000001", "abc", "POST", uri, "onvif-rust");
        assert_eq!(
            a.authenticate("POST", uri, Some(&header), None),
            AuthResult::Authenticated
        );

        // Replaying the same header (same nc) must fail.
        assert_eq!(
            a.authenticate("POST", uri, Some(&header), None),
            AuthResult::Invalid
        );

        // A higher nc with the same nonce is fine.
        let header = digest_header(&nonce, "00000002", "abc", "POST", uri, "onvif-rust");
        assert_eq!(
            a.authenticate("POST", uri, Some(&header), None),
            AuthResult::Authenticated
        );

        // Wrong password.
        let header = digest_header(&nonce, "00000003", "abc", "POST", uri, "wrong");
        assert_eq!(
            a.authenticate("POST", uri, Some(&header), None),
            AuthResult::Invalid
        );
    }

    #[test]
    fn digest_auth_rejects_unknown_nonce_as_stale() {
        let a = auth();
        let header = digest_header("deadbeef", "00000001", "abc", "POST", "/", "onvif-rust");
        assert_eq!(
            a.authenticate("POST", "/", Some(&header), None),
            AuthResult::Stale
        );
    }

    #[test]
    fn digest_auth_rejects_uri_mismatch() {
        let a = auth();
        let nonce = nonce_from_challenge(&a);
        let header = digest_header(&nonce, "00000001", "abc", "POST", "/other", "onvif-rust");
        assert_eq!(
            a.authenticate("POST", "/onvif/device_service", Some(&header), None),
            AuthResult::Invalid
        );
    }

    #[test]
    fn digest_auth_accepts_absolute_uri_and_legacy_rfc2069() {
        let a = auth();
        let nonce = nonce_from_challenge(&a);
        let uri = "http://cam:8080/onvif/device_service";
        let header = digest_header(&nonce, "00000001", "abc", "POST", uri, "onvif-rust");
        assert_eq!(
            a.authenticate("POST", "/onvif/device_service", Some(&header), None),
            AuthResult::Authenticated
        );

        let nonce = nonce_from_challenge(&a);
        let ha1 = md5_hex(&format!("admin:{REALM}:onvif-rust"));
        let ha2 = md5_hex("GET:/snapshot.jpg");
        let response = md5_hex(&format!("{ha1}:{nonce}:{ha2}"));
        let header = format!(
            "Digest username=\"admin\", realm=\"{REALM}\", nonce=\"{nonce}\", uri=\"/snapshot.jpg\", response=\"{response}\""
        );
        assert_eq!(
            a.authenticate("GET", "/snapshot.jpg", Some(&header), None),
            AuthResult::Authenticated
        );
    }

    #[test]
    fn ws_security_text_and_digest() {
        let a = auth();
        let text = UsernameToken {
            username: "admin".into(),
            password: "onvif-rust".into(),
            password_type: PasswordType::Text,
            nonce: None,
            created: None,
        };
        assert_eq!(
            a.authenticate("POST", "/", None, Some(&text)),
            AuthResult::Authenticated
        );

        let created = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let nonce_bytes = b"0123456789abcdef";
        let mut hasher = sha1::Sha1::new();
        hasher.update(nonce_bytes);
        hasher.update(created.as_bytes());
        hasher.update(b"onvif-rust");
        let digest = BASE64.encode(hasher.finalize());
        let mut token = UsernameToken {
            username: "admin".into(),
            password: digest,
            password_type: PasswordType::Digest,
            nonce: Some(BASE64.encode(nonce_bytes)),
            created: Some(created),
        };
        assert_eq!(
            a.authenticate("POST", "/", None, Some(&token)),
            AuthResult::Authenticated
        );

        token.username = "other".into();
        assert_eq!(
            a.authenticate("POST", "/", None, Some(&token)),
            AuthResult::Invalid
        );

        token.username = "admin".into();
        token.created = Some("2001-01-01T00:00:00Z".into());
        assert_eq!(
            a.authenticate("POST", "/", None, Some(&token)),
            AuthResult::Invalid
        );
    }

    #[test]
    fn parses_quoted_params_with_commas() {
        let p = parse_auth_params("username=\"a,b\", realm=\"r\", nc=00000001, uri=\"/x?y=1,2\"");
        assert_eq!(p["username"], "a,b");
        assert_eq!(p["realm"], "r");
        assert_eq!(p["nc"], "00000001");
        assert_eq!(p["uri"], "/x?y=1,2");
    }

    #[test]
    fn constant_time_eq_basics() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
