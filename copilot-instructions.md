# Copilot Instructions for ONVIF Media Transcoder

## Project Overview

ONVIF-compatible media transcoder (Rust) converting input streams to RTSP with full ONVIF device emulation.
Integrates FFmpeg, MediaMTX, and custom ONVIF service with WS-Discovery.

## Critical Review Points

### Rust Code Quality

- **MUST pass**: `cargo clippy --all-targets --all-features -- -D warnings` (zero warnings)
- **Memory safety**: Review `unsafe` blocks, prefer `Result<T,E>`, avoid `.unwrap()`
- **ONVIF specific**: Validate SOAP XML structure, authentication (Basic/Digest/WS-Security), WS-Discovery compliance

```rust
// ❌ Avoid
let response = format!("<?xml version=\"1.0\"?><soap:Envelope>{}</soap:Envelope>", body);
let config = std::env::var("ONVIF_PORT").unwrap();

// ✅ Prefer  
let response = format!("<?xml version=\"1.0\"?><soap:Envelope>{body}</soap:Envelope>");
let config = std::env::var("ONVIF_PORT")?;
```

### Security & Infrastructure

- **Docker**: Non-root user, no hardcoded secrets, multi-stage builds
- **Ports**: 8080 (ONVIF), 8554 (RTSP), 3702 (WS-Discovery)
- **Authentication**: No plaintext logging, validate inputs, prevent XML injection
- **FFmpeg**: Prevent command injection, validate stream URLs

### ONVIF Compliance

- SOAP responses match ONVIF specs
- GetCapabilities/GetProfiles/GetStreamUri compliance
- WS-Security timestamp validation
- Device discovery metadata requirements

## Pre-Merge Checklist

- [ ] Clippy clean (`-D warnings`)
- [ ] `cargo fmt --check` passes
- [ ] Security scan clean
- [ ] Docker multi-arch builds
- [ ] ONVIF endpoint tests pass
- [ ] No performance regression

## Patterns

```rust
// Error types
#[derive(Debug, thiserror::Error)]
enum OnvifError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

// Logging
tracing::info!(port = %onvif_port, "ONVIF service starting");

// Constants
const DEFAULT_RTSP_PORT: u16 = 8554;
```

## AI Code Notes

- Extra security review for auth/network code
- Verify ONVIF protocol compliance manually
- Test all network-facing functionality
- Reference: [ONVIF Core Spec](https://www.onvif.org/specs/core/ONVIF-Core-Specification.pdf)
