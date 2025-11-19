# ONVIF Media Transcoder

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/W4ff1e/onvif-media-transcoder/docker-publish.yml?branch=main&label=build)](https://github.com/W4ff1e/onvif-media-transcoder/actions/workflows/docker-publish.yml)
[![Docker Hub](https://img.shields.io/docker/pulls/w4ff1e/onvif-media-transcoder?logo=docker)](https://hub.docker.com/r/w4ff1e/onvif-media-transcoder)
[![Docker Image Version](https://img.shields.io/docker/v/w4ff1e/onvif-media-transcoder?logo=docker&sort=semver)](https://hub.docker.com/r/w4ff1e/onvif-media-transcoder/tags)
[![Docker Image Size](https://img.shields.io/docker/image-size/w4ff1e/onvif-media-transcoder/latest?logo=docker)](https://hub.docker.com/r/w4ff1e/onvif-media-transcoder)
[![ONVIF](https://img.shields.io/badge/ONVIF-compatible-green.svg)](https://www.onvif.org/)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Security](https://img.shields.io/badge/security-policy-red.svg)](SECURITY.md)

> ⚠️ **AI-Generated Code Warning**: This project contains code generated with the assistance of AI tools
> (GitHub Copilot). While thoroughly tested, this software is provided as-is and may not be suitable for
> production environments without proper review, testing, and validation by qualified developers.
> Use at your own risk.

## Overview

A Rust-based ONVIF Media Transcoder that converts media streams (RTSP, HTTP, etc.) into ONVIF-compatible RTSP streams.
It includes a built-in WS-Discovery server for device discovery.

## Table of Contents

- [ONVIF Media Transcoder](#onvif-media-transcoder)
  - [Overview](#overview)
  - [Features](#features)
  - [Quick Start](#quick-start)
  - [Architecture](#architecture)
  - [ONVIF Compatibility](#onvif-compatibility)
  - [Testing](#testing)
  - [Troubleshooting](#troubleshooting)
  - [Development](#development)
  - [CI/CD and Releases](#cicd-and-releases)
  - [Contributing](#contributing)
  - [Security](#security)
  - [License](#license)
  - [Authors](#authors)

## Features

- [x] **Input Stream Support**: Re-mux MediaMTX-compatible input (HLS, MP4, RTSP, HTTP streams)
- [x] **ONVIF Compliance**: ONVIF Profile S compatibility with standard endpoints
- [x] **Network Discovery**: WS-Discovery implementation for device detection
- [x] **Authentication**: HTTP Basic, HTTP Digest, and WS-Security support
- [x] **Stream Re-muxing**: Direct stream re-muxing without re-encoding, minimal latency

## Quick Start

### Quick Start Script

```bash
git clone https://github.com/W4ff1e/onvif-media-transcoder.git
cd onvif-media-transcoder
./scripts/quick-start.sh run
```

### Docker Run

```bash
# Run with default demo stream
docker run --rm --network host w4ff1e/onvif-media-transcoder:latest

# Run with custom stream and credentials
docker run --rm --network host \
  -e INPUT_URL="https://your-stream.m3u8" \
  -e DEVICE_NAME="My-Custom-Camera" \
  -e ONVIF_USERNAME="myuser" \
  -e ONVIF_PASSWORD="mypassword" \
  w4ff1e/onvif-media-transcoder:latest
```

### Available Docker Tags

- **`latest`**: Latest stable release
- **`unstable`**: Latest commit to main branch
- **`v0.x.x`**: Specific version releases

### Environment Variables

| Variable | Default | Description |
| :--- | :--- | :--- |
| `INPUT_URL` | Demo HLS stream | Source video stream URL |
| `RTSP_OUTPUT_PORT` | `8554` | RTSP server port |
| `RTSP_PATH` | `/stream` | RTSP stream path |
| `ONVIF_PORT` | `8080` | ONVIF web service port |
| `DEVICE_NAME` | `ONVIF-Media-Transcoder` | Camera device name |
| `ONVIF_USERNAME` | `admin` | ONVIF authentication username |
| `ONVIF_PASSWORD` | `onvif-rust` | ONVIF authentication password |
| `WS_DISCOVERY_ENABLED` | `true` | Enable WS-Discovery service |
| `DEBUGLOGGING` | `false` | Enable debug logging |

**Note**: `--network host` is recommended for WS-Discovery to work across network boundaries.

## Architecture

The service consists of three components:

1. **ONVIF Service (Rust)**: SOAP web service for device management.
2. **WS-Discovery Service (Rust)**: Multicast discovery service.
3. **MediaMTX**: RTSP server for stream re-muxing.

```text
┌─────────────────┐    ┌─────────────────┐
│   Input Stream  │──▶│    MediaMTX     │
│  (HLS/MP4/etc)  │    │ Re-mux & RTSP   │
└─────────────────┘    └────────┬────────┘
                                │
                                ▼
┌─────────────────┐    ┌─────────────────┐
│  ONVIF Clients  │◀──│  ONVIF Service  │
│                 │    │   (Port 8080)   │
└────────┬────────┘    └────────┬────────┘
         │                      │
         ▼                      ▲
┌─────────────────┐    ┌─────────────────┐
│  WS-Discovery   │──▶│     Device      │
│   (Port 3702)   │    │   Discovery     │
└─────────────────┘    └─────────────────┘
```

## ONVIF Compatibility

### Supported Endpoints

**Device Service** (`/onvif/device_service`):
- `GetCapabilities`, `GetDeviceInformation`

**Media Service** (`/onvif/media_service`):
- `GetProfiles`, `GetStreamUri`, `GetVideoSources`, `GetServiceCapabilities`

### Authentication

> ⚠️ **Warning**: The authentication implementation is custom-built.
> **It is strongly recommended to restrict access at the network level.**

- **Methods**: HTTP Basic, HTTP Digest, WS-Security (PasswordDigest/PasswordText)
- **Default**: `admin` / `onvif-rust`

### Discovery

- **Protocol**: WS-Discovery (UDP 3702)
- **Multicast**: `239.255.255.250:3702`

## Testing

### Manual Testing

```bash
# Test device discovery
curl -X POST http://localhost:8080/onvif/device_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>'

# Test RTSP stream
ffprobe rtsp://localhost:8554/stream
```

## Troubleshooting

- **WS-Discovery**: Use `--network host`. Ensure UDP 3702 is open.
- **Connection**: Check ports 8080/8554. Verify credentials.
- **Logs**: Check container logs for details.

## Development

### Prerequisites

- Docker
- Rust 1.70+ (for local dev)

### Building

```bash
# Build with Docker
docker build -t onvif-media-transcoder .

# Build locally
cargo build --release
```

Project structure:

```text
├── src/                     # Rust source code
│   ├── lib.rs               # Library root
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── ws_discovery.rs      # WS-Discovery implementation
│   └── onvif/               # ONVIF logic
│       ├── mod.rs           # Request handling
│       ├── endpoints.rs     # Constants
│       └── responses.rs     # SOAP templates
├── examples/                # Example configurations
├── scripts/                 # Utility scripts
├── docs/                    # Documentation
├── Dockerfile               # Multi-stage build
├── entrypoint.sh            # Service orchestration
└── mediamtx.yml             # MediaMTX configuration
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an
issue first to discuss what you would like to change.

**Code Guidelines:**

- This project uses AI-assisted development with GitHub Copilot
- All contributions should be reviewed and tested before merging
- Ensure proper error handling and documentation
- Follow existing code patterns and style
- Add tests and update documentation as needed

Please make sure to update tests as appropriate and follow the existing code style.

## Security

Security is an important consideration for ONVIF Media Transcoder, especially given its network-exposed
services and authentication mechanisms.

### Important Security Notice

⚠️ This project contains AI-generated code and should undergo security review before production deployment.

### Key Security Considerations

- **Default Credentials**: Change the default `admin`/`onvif-rust` credentials in production
- **Network Exposure**: Multiple services (ONVIF, RTSP, WS-Discovery) are exposed by default
- **Authentication**: Supports multiple methods including WS-Security
- **Container Security**: Regular vulnerability scanning and security updates

### Reporting Security Issues

Please report security vulnerabilities responsibly:

- Use [GitHub Security Advisories](https://github.com/W4ff1e/onvif-media-transcoder/security/advisories)

For detailed security information, deployment best practices, and vulnerability reporting procedures,
see the [**Security Policy**](SECURITY.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Disclaimer**: This software is provided "AS IS" without warranty of any kind. The use of AI-generated
code components requires additional validation and testing before deployment in production environments.

## Authors

- [@W4ff1e](https://github.com/W4ff1e) - Initial work and maintenance
- [GitHub Copilot](https://github.com/features/copilot) - Pair programming and code assistance

## Stats

![Alt](https://repobeats.axiom.co/api/embed/f19d8fae5d95fd971fe46aa847f9f23b9e278420.svg "Repobeats analytics image")

---

<!--markdownlint-disable-next-line -->
**Made with :yellow_heart: by [Waffle](https://github.com/W4ff1e) in collaboration with GitHub Copilot**
