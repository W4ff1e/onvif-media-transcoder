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

**ONVIF Media Transcoder** is a Docker-based solution that re-muxes input streams (HLS, MP4, RTSP, etc.)
into ONVIF-compatible camera devices. It provides network discovery, media profiles, and authentication
endpoints for integration with ONVIF clients.

## Table of Contents

- [ONVIF Media Transcoder](#onvif-media-transcoder)
  - [Overview](#overview)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Quick Start](#quick-start)
    - [Quick Start Script](#quick-start-script)
    - [Docker Run](#docker-run)
    - [Available Docker Tags](#available-docker-tags)
    - [Docker Compose](#docker-compose)
    - [Environment Variables](#environment-variables)
  - [Architecture](#architecture)
  - [ONVIF Compatibility](#onvif-compatibility)
    - [Supported Endpoints](#supported-endpoints)
    - [Authentication](#authentication)
    - [Discovery](#discovery)
  - [Testing](#testing)
    - [ONVIF Clients](#onvif-clients)
    - [Manual Testing](#manual-testing)
  - [Troubleshooting](#troubleshooting)
    - [WS-Discovery Not Working](#ws-discovery-not-working)
    - [Connection Issues](#connection-issues)
    - [Authentication Issues](#authentication-issues)
    - [Performance Issues](#performance-issues)
  - [Development](#development)
    - [Prerequisites](#prerequisites)
    - [Building](#building)
    - [Configuration](#configuration)
    - [VS Code Integration](#vs-code-integration)
  - [CI/CD and Releases](#cicd-and-releases)
    - [Automated Docker Builds](#automated-docker-builds)
    - [Development Workflow](#development-workflow)
  - [Contributing](#contributing)
  - [Security](#security)
    - [Important Security Notice](#important-security-notice)
    - [Key Security Considerations](#key-security-considerations)
    - [Reporting Security Issues](#reporting-security-issues)
  - [License](#license)
  - [Authors](#authors)

## Features

- [x] **Input Stream Support**: Re-mux MediaMTX-compatible input (HLS, MP4, RTSP, HTTP streams)
- [x] **ONVIF Compliance**: ONVIF Profile S compatibility with standard endpoints
- [x] **Network Discovery**: WS-Discovery implementation for device detection
- [x] **Authentication**: HTTP Basic, HTTP Digest, and WS-Security support
- [x] **Stream Re-muxing**:
  - Direct stream re-muxing without re-encoding
  - RTSP delivery over TCP
  - Minimal latency processing

## Quick Start

### Quick Start Script

Use the included quick-start script for setup:

```bash
# Clone and setup
git clone https://github.com/W4ff1e/onvif-media-transcoder.git
cd onvif-media-transcoder

# Setup configuration (optional)
./scripts/quick-start.sh setup

# Build and run with one command
./scripts/quick-start.sh run

# Or use Docker Compose
./scripts/quick-start.sh compose
```

### Docker Run

```bash
# Pull the latest release from Docker Hub
docker pull w4ff1e/onvif-media-transcoder:latest

# Run with default demo stream (using host network for WS-Discovery)
docker run --rm --network host w4ff1e/onvif-media-transcoder:latest

# Run with custom stream and credentials
docker run --rm --network host \
  -e INPUT_URL="https://your-stream.m3u8" \
  -e DEVICE_NAME="My-Custom-Camera" \
  -e ONVIF_USERNAME="myuser" \
  -e ONVIF_PASSWORD="mypassword" \
  w4ff1e/onvif-media-transcoder:latest

# Alternative: Run with port mapping (WS-Discovery may not work across networks)
docker run --rm -p 8080:8080 -p 8554:8554 -p 3702:3702/udp \
  -e INPUT_URL="https://your-stream.m3u8" \
  w4ff1e/onvif-media-transcoder:latest

# Run specific version
docker run --rm --network host w4ff1e/onvif-media-transcoder:v0.1.0

# Run unstable/development version (latest commit to main)
docker run --rm --network host w4ff1e/onvif-media-transcoder:unstable
```

### Available Docker Tags

The project publishes Docker images to Docker Hub with the following tags:

- **`latest`** - Latest stable release
- **`unstable`** - Latest commit to main branch
- **`v0.2.0`** - Specific version releases

### Docker Compose

Use the provided Docker Compose examples for deployment:

```bash
# Start with default configuration
docker-compose -f examples/docker-compose.yml up

# Start with custom environment file
cp examples/.env.example .env
# Edit .env with your settings
docker-compose -f examples/docker-compose.yml --env-file .env up

# Start in background
docker-compose -f examples/docker-compose.yml up -d

# Stop services
docker-compose -f examples/docker-compose.yml down
```

### Environment Variables

| Variable               | Default                  | Description                   |
| ---------------------- | ------------------------ | ----------------------------- |
| `INPUT_URL`            | Demo HLS stream          | Source video stream URL       |
| `RTSP_OUTPUT_PORT`     | `8554`                   | RTSP server port              |
| `RTSP_PATH`            | `/stream`                | RTSP stream path              |
| `ONVIF_PORT`           | `8080`                   | ONVIF web service port        |
| `DEVICE_NAME`          | `ONVIF-Media-Transcoder` | Camera device name            |
| `ONVIF_USERNAME`       | `admin`                  | ONVIF authentication username |
| `ONVIF_PASSWORD`       | `onvif-rust`             | ONVIF authentication password |
| `WS_DISCOVERY_ENABLED` | `true`                   | Enable WS-Discovery service   |
| `DEBUGLOGGING`         | `false`                  | Enable debug logging          |

**Note**: The service automatically detects the container IP and configures all services accordingly.

**Network Requirements**: For WS-Discovery functionality, use `--network host` when running with Docker.
This allows the multicast discovery protocol to work across network boundaries. Port mapping (`-p`)
can be used as an alternative but may limit discovery functionality.

## Architecture

The service consists of three components working together to provide ONVIF-compatible streaming:

1. **ONVIF Service (Rust)**: ONVIF SOAP web service providing device management and media profiles
2. **WS-Discovery Service (Rust)**: Network discovery service for device detection via multicast
3. **MediaMTX**: Stream re-muxing and RTSP server for stream delivery to ONVIF clients

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

**Data Flow:**

- **Stream Processing**: Input streams are re-muxed by MediaMTX and served via RTSP (port configurable via `RTSP_OUTPUT_PORT`)
- **Device Discovery**: WS-Discovery broadcasts device availability on the network
- **ONVIF Integration**: ONVIF service provides standard endpoints and references the RTSP stream
- **Client Access**: ONVIF clients discover the device and access media streams through standard protocols

## ONVIF Compatibility

### Supported Endpoints

**Device Service** (`/onvif/device_service`):

- `GetCapabilities` - Device and service capabilities
- `GetDeviceInformation` - Model, firmware, serial number

**Media Service** (`/onvif/media_service`):

- `GetProfiles` - Video/audio encoding profiles
- `GetStreamUri` - RTSP stream URI for playback
- `GetVideoSources` - Available video sources and resolutions
- `GetServiceCapabilities` - Media service capabilities

### Authentication

> ⚠️ **Warning**: The authentication implementation in this project is custom-built
> and has not undergone extensive security review or battle testing.
> Use with caution—**it is strongly recommended to restrict access at the network level**
> and not expose the service directly to untrusted networks or the public internet.

- **Supported Methods**: HTTP Basic Authentication, HTTP Digest Authentication, WS-Security (PasswordDigest/PasswordText)
- **Default Credentials**: `admin` / `onvif-rust`
- **Security**: Device discovery endpoints allow unauthenticated access for ONVIF compliance
- **Customization**: Configure via `ONVIF_USERNAME` and `ONVIF_PASSWORD` environment variables

### Discovery

- **WS-Discovery Protocol**: Standard device discovery protocol
- **Device Type**: `NetworkVideoTransmitter`
- **ONVIF Profile**: Streaming profile compliant
- **Multicast**: `239.255.255.250:3702` (UDP port 3702)

## Testing

### ONVIF Clients

Test with ONVIF-compatible applications:

- **ONVIF Device Manager** - Device discovery and configuration
- **ffplay** - Network stream playback: `ffplay rtsp://host:8554/stream`
- **FFprobe** - Stream analysis: `ffprobe rtsp://host:8554/stream`

### Manual Testing

```bash
# Test device discovery (no authentication required)
curl -X POST http://localhost:8080/onvif/device_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>'

# Test authenticated endpoint with Basic Auth
curl -X POST http://localhost:8080/onvif/media_service \
  -H "Content-Type: application/soap+xml" \
  -u admin:onvif-rust \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetProfiles/></soap:Body></soap:Envelope>'

# Test RTSP stream
ffprobe -v quiet -select_streams v:0 -show_entries stream=codec_name -of csv=p=0 rtsp://localhost:8554/stream

# Test WS-Security authentication (example with ONVIF Device Manager format)
curl -X POST http://localhost:8080/onvif/media_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><soap:Header><wsse:Security><wsse:UsernameToken><wsse:Username>admin</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordText">onvif-rust</wsse:Password></wsse:UsernameToken></wsse:Security></soap:Header><soap:Body><GetProfiles/></soap:Body></soap:Envelope>'
```

## Troubleshooting

### WS-Discovery Not Working

If devices are not automatically discovered by ONVIF clients:

1. **Use Host Network Mode**: Run with `--network host` or use the provided Docker Compose file
2. **Check Firewall**: Ensure UDP port 3702 is open for multicast traffic
3. **Network Configuration**: WS-Discovery requires multicast support on your network
4. **Manual Discovery**: ONVIF clients can still connect directly using the IP address

### Connection Issues

If ONVIF clients cannot connect:

1. **Port Accessibility**: Ensure ports 8080 (ONVIF) and 8554 (RTSP) are accessible
2. **Authentication**: Verify username/password in client configuration
3. **Stream URL**: Use format `rtsp://host:8554/stream` for direct RTSP access
4. **WS-Security**: Some clients require WS-Security authentication instead of Basic/Digest

### Authentication Issues

If authentication is failing:

1. **Credential Verification**: Double-check `ONVIF_USERNAME` and `ONVIF_PASSWORD` environment variables
2. **Authentication Method**: Try different methods (Basic Auth, Digest Auth, WS-Security)
3. **Client Compatibility**: Some ONVIF clients prefer specific authentication methods
4. **Debug Logs**: Check container logs for detailed authentication information

### Performance Issues

If experiencing stream lag or quality issues:

1. **Input Stream**: Verify the input stream is stable and accessible
2. **Network Bandwidth**: Ensure sufficient bandwidth for re-muxing
3. **Hardware Resources**: Monitor CPU usage during stream processing
4. **Container Resources**: Increase Docker memory/CPU limits if needed

## Development

### Prerequisites

- [Docker](https://www.docker.com/get-started) for containerization
- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (for local development)
- [VS Code](https://code.visualstudio.com/) with Rust extensions
- [Git](https://git-scm.com/) for version control

### Building

```bash
# Clone the repository
git clone https://github.com/W4ff1e/onvif-media-transcoder.git
cd onvif-media-transcoder

# Build with Docker
docker build -t onvif-media-transcoder .

# Build with Docker Compose
docker-compose build

# Or build Rust components locally
cargo build --release
```

### Configuration

The project includes sample configuration files in the `examples/` directory:

- `examples/docker-compose.yml` - Local development Docker Compose setup
- `examples/docker-compose.yml` - Production-ready setup using published Docker Hub images  
- `examples/.env.example` - Sample environment variables

```bash
# Copy and customize environment file
cp examples/.env.example .env
# Edit .env with your preferred settings
nano .env
```

### VS Code Integration

The project includes VS Code tasks and debugging configuration:

- **Build**: `Ctrl+Shift+P` → "Tasks: Run Task" → "Docker: Build Image"
- **Run**: `Ctrl+Shift+P` → "Tasks: Run Task" → "Docker: Run Container (Test)"
- **Local Development**: `Ctrl+Shift+P` → "Tasks: Run Task" → "Rust: Run Local"
- **Debug**: F5 to start debugging with breakpoints

For local development without Docker, see [**Local Development Guide**](docs/LOCAL_DEVELOPMENT.md).

## CI/CD and Releases

### Automated Docker Builds

The project uses GitHub Actions to build and publish Docker images:

- **Every commit to `main`**: Publishes `unstable` tag
- **Tagged releases**: Publishes versioned tags (e.g., `v0.20.0`) and updates `latest`
- **Multi-architecture**: Builds for both `linux/amd64` and `linux/arm64`
- **Security scanning**: Vulnerability scanning with CodeQL and Trivy

> Note: The Multi-architecture builds are currently untested and may not work on anything except x86_64 Linux systems.

### Development Workflow

```bash
# For contributors working on features
git checkout -b feature/my-feature
# ... make changes ...
git push origin feature/my-feature
# Create pull request - triggers CI tests
```

Project structure:

```text
├── src/                     # Rust source code
│   ├── main.rs              # ONVIF service implementation
│   ├── onvif_responses.rs   # SOAP response templates
│   └── ws_discovery.rs      # WS-Discovery implementation
├── examples/                # Example configurations
│   ├── docker-compose.yml   # Production setup with published images
│   ├── docker-compose.local.yml # Local development setup
│   └── .env.example         # Sample environment variables
├── scripts/                 # Utility scripts
│   ├── build.sh             # Docker build script
│   ├── publish.sh           # Publishing script
│   └── quick-start.sh       # Quick start script
├── docs/                    # Documentation
├── Dockerfile               # Multi-stage build
├── entrypoint.sh           # Service orchestration
├── mediamtx.yml            # MediaMTX configuration
└── .vscode/                # VS Code configuration
    ├── tasks.json          # Build and run tasks
    └── launch.json         # Debug configuration
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

---

<!--markdownlint-disable-next-line -->
**Made with :yellow_heart: by [Waffle](https://github.com/W4ff1e) in collaboration with GitHub Copilot**
