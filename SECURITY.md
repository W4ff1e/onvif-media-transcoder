# Security Policy

## Supported versions

| Version | Supported | Notes |
| :--- | :--- | :--- |
| `latest` (0.31 and newer) | Yes | Current release line |
| `unstable` | Best effort | Latest commit on `main` that passed CI |
| Older tags | No | Please upgrade |

## What the software exposes

| Service | Port | Authentication |
| :--- | :--- | :--- |
| ONVIF SOAP over HTTP | 8080/tcp | HTTP Basic, HTTP Digest (RFC 7616 with replay protection) or WS-Security UsernameToken; a few discovery operations are public as ONVIF requires |
| JPEG snapshot | 8080/tcp `/snapshot.jpg` | Same credentials |
| RTSP stream | 8554/tcp (+ UDP RTP/RTCP) | Same credentials via MediaMTX, unless `RTSP_AUTH_ENABLED=false` |
| WS-Discovery | 3702/udp multicast | None (discovery protocol) |

Nothing is encrypted in transit: ONVIF runs over plain HTTP and RTSP is
unencrypted. Treat the network these ports are on as trusted.

## Hardening built in

- The container runs as an unprivileged user (uid 10001).
- Base images are pinned; MediaMTX is downloaded with SHA-256 verification.
- The Rust binary is static (musl); unused MediaMTX servers (RTMP, HLS,
  WebRTC, SRT, MoQ, API) are disabled.
- Credentials are passed through the environment, never on the command line,
  and are not logged unless `DEBUG_LOGGING` is enabled.
- Dependencies are checked by `cargo deny` (advisories, licenses) in CI and
  kept current by Dependabot; images are scanned with Trivy on publish.

## Recommendations for operators

1. Change the default credentials (`admin` / `onvif-rust`) and use a long password.
2. Keep the ONVIF and RTSP ports on the camera/NVR network only; do not expose them to the internet.
3. Leave `RTSP_AUTH_ENABLED=true` unless a client cannot authenticate to RTSP.
4. Do not run with `DEBUG_LOGGING=true` in production; it logs full requests.
5. Update the image regularly; `latest` follows releases and `unstable` follows `main`.

## Reporting a vulnerability

Please use
[GitHub Security Advisories](https://github.com/W4ff1e/onvif-media-transcoder/security/advisories/new)
rather than a public issue. Include steps to reproduce and your assessment of
the impact.

- Acknowledgement within 48 hours
- Assessment and a plan within one week
- Public disclosure after a fix is released

## Known limitations

- The authentication code is custom. It has tests for the documented cases
  and replay protection, but it has not had an external audit.
- ONVIF over HTTPS and RTSPS are not implemented.
- WS-Discovery is unauthenticated by design; anyone on the local network can
  see the device.

Last updated: September 2026.
