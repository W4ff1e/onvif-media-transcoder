# Copilot Instructions for ONVIF Media Transcoder

## Project Overview

ONVIF-compatible media transcoder (Rust) that re-muxes input streams via MediaMTX and provides ONVIF device
emulation with WS-Discovery.

**Key Components:** Rust ONVIF service, WS-Discovery, MediaMTX integration, Docker containerization

## Critical Requirements

### Code Quality

- **MUST pass**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Memory safety**: Prefer `Result<T,E>`, avoid `.unwrap()` except in tests
- **ONVIF compliance**: Validate SOAP XML, authentication (Basic/Digest/WS-Security), WS-Discovery

### Security

- **Docker**: Non-root user, no hardcoded secrets
- **Authentication**: No credential logging, validate inputs, prevent XML injection
- **Ports**: 8080 (ONVIF), 8554 (RTSP), 3702 (WS-Discovery)

## Pre-Merge Checklist

- [ ] Clippy clean (`-D warnings`)
- [ ] `cargo fmt --check` passes
- [ ] All tests pass
- [ ] Docker builds successfully
- [ ] ONVIF endpoints tested with real clients

## Code Patterns

```rust
// Error handling
#[derive(Debug, thiserror::Error)]
enum OnvifError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

// Logging
tracing::info!(port = %onvif_port, "ONVIF service starting");

// Configuration (CLI args for local dev, env vars handled by entrypoint.sh)
#[derive(Debug, Clone, Parser)]
struct Config {
    #[arg(short = 'r', long)]
    rtsp_stream_url: String,
}
```

## AI Code Review Notes

- **Extra security review** for authentication and network-facing code
- **Verify ONVIF compliance** against [official specs](https://www.onvif.org/specs/)
- **Check dependencies** for vulnerabilities regularly
