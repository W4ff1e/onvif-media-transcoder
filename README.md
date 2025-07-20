# ONVIF Media Transcoder

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](Dockerfile)
[![ONVIF](https://img.shields.io/badge/ONVIF-compatible-green.svg)](https://www.onvif.org/)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

> ⚠️ **AI-Generated Code Warning**: This project contains code generated with the assistance of AI tools (GitHub Copilot). While thoroughly tested, this software is provided as-is and may not be suitable for production environments without proper review, testing, and validation by qualified developers. Use at your own risk.

## Overview

**ONVIF Media Transcoder** is a Docker-based solution that converts any input stream (HLS, MP4, RTSP, etc.) into a fully ONVIF-compatible camera device. It provides automatic network discovery, standardized media profiles, and authentication-protected endpoints for seamless integration with ONVIF clients.

## Table of Contents

- [ONVIF Media Transcoder](#onvif-media-transcoder)
  - [Overview](#overview)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Quick Start](#quick-start)
    - [Quick Start Script](#quick-start-script)
    - [Docker Run](#docker-run)
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
  - [Contributing](#contributing)
  - [License](#license)
  - [Authors](#authors)

## Features

- [x] **Universal Input Support**: Convert any FFmpeg-compatible input (HLS, MP4, RTSP, HTTP streams)
- [x] **ONVIF Compliance**: Full ONVIF Profile S compatibility with standardized endpoints
- [x] **Network Discovery**: Native WS-Discovery implementation for automatic device detection
- [x] **Multi-Protocol Authentication**: HTTP Basic, HTTP Digest, and WS-Security support
- [x] **Professional Streaming**:
  - H.264/AAC encoding with optimized settings
  - RTSP over TCP for reliable delivery
  - Real-time transcoding with low latency
- [x] **Production Features**:
  - Comprehensive error handling and logging
  - Graceful service recovery and health monitoring
  - Container orchestration with MediaMTX integration
  - Configurable authentication and security settings

## Quick Start

### Quick Start Script

For the fastest setup, use the included quick-start script:

```bash
# Clone and setup
git clone https://github.com/W4ff1e/onvif-media-transcoder.git
cd onvif-media-transcoder

# Setup configuration (optional)
./quick-start.sh setup

# Build and run with one command
./quick-start.sh run

# Or use Docker Compose (recommended)
./quick-start.sh compose
```

### Docker Run

```bash
# Build the image
docker build -t onvif-media-transcoder .

# Run with default demo stream (using host network for WS-Discovery)
docker run --rm --network host onvif-media-transcoder

# Run with custom stream and credentials
docker run --rm --network host \
  -e INPUT_URL="https://your-stream.m3u8" \
  -e DEVICE_NAME="My-Custom-Camera" \
  -e ONVIF_USERNAME="myuser" \
  -e ONVIF_PASSWORD="mypassword" \
  onvif-media-transcoder

# Alternative: Run with port mapping (WS-Discovery may not work across networks)
docker run --rm -p 8080:8080 -p 8554:8554 -p 3702:3702/udp \
  -e INPUT_URL="https://your-stream.m3u8" \
  onvif-media-transcoder
```

### Docker Compose

For easier deployment and configuration, use the provided Docker Compose file:

```bash
# Start with default configuration
docker-compose up

# Start with custom environment file
docker-compose --env-file .env.custom up

# Start in background
docker-compose up -d

# Stop services
docker-compose down
```

### Environment Variables

| Variable           | Default                  | Description                   |
| ------------------ | ------------------------ | ----------------------------- |
| `INPUT_URL`        | Demo HLS stream          | Source video stream URL       |
| `RTSP_OUTPUT_PORT` | `8554`                   | RTSP server port              |
| `ONVIF_PORT`       | `8080`                   | ONVIF web service port        |
| `DEVICE_NAME`      | `ONVIF-Media-Transcoder` | Camera device name            |
| `RTSP_PATH`        | `/stream`                | RTSP stream path              |
| `ONVIF_USERNAME`   | `admin`                  | ONVIF authentication username |
| `ONVIF_PASSWORD`   | `onvif-rust`             | ONVIF authentication password |

**Note**: The transcoder automatically detects the container IP and configures all services accordingly.

**Network Requirements**: For optimal WS-Discovery functionality, use `--network host` when running with Docker. This allows the multicast discovery protocol to work properly across network boundaries. Port mapping (`-p`) can be used as an alternative but may limit discovery functionality in some network configurations.

## Architecture

The transcoder consists of four integrated components:

1. **MediaMTX RTSP Server**: Professional RTSP server for reliable stream delivery
2. **FFmpeg Transcoder**: Real-time stream conversion with optimized encoding
3. **ONVIF Service**: Native Rust implementation of ONVIF SOAP endpoints
4. **WS-Discovery**: Network discovery service for automatic device detection

```text
Input Stream → FFmpeg → MediaMTX → RTSP Client
                           ↓
                    ONVIF Service ← WS-Discovery ← Network Discovery
```

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

- **Supported Methods**: HTTP Basic Authentication, HTTP Digest Authentication, WS-Security (PasswordDigest/PasswordText)
- **Default Credentials**: `admin` / `onvif-rust`
- **Security**: Device discovery endpoints allow unauthenticated access for ONVIF compliance
- **Customization**: Configure via `ONVIF_USERNAME` and `ONVIF_PASSWORD` environment variables
- **Standards Compliance**: Full WS-Security Username Token Profile implementation

### Discovery

- **WS-Discovery Protocol**: Standards-compliant device discovery
- **Device Type**: `NetworkVideoTransmitter`
- **ONVIF Profile**: Streaming profile compliant
- **Multicast**: `239.255.255.250:3702` (UDP port 3702)

## Testing

### ONVIF Clients

Test with popular ONVIF-compatible applications:

- **ONVIF Device Manager** - Full device discovery and configuration
- **VLC Media Player** - Network stream playback: `rtsp://host:8554/stream`
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
2. **Network Bandwidth**: Ensure sufficient bandwidth for transcoding
3. **Hardware Resources**: Monitor CPU usage during transcoding
4. **Container Resources**: Increase Docker memory/CPU limits if needed

## Development

### Prerequisites

- [Docker](https://www.docker.com/get-started) for containerization
- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (for local development)
- [VS Code](https://code.visualstudio.com/) with Rust extensions (recommended)

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

The project includes sample configuration files:

- `docker-compose.yml` - Production-ready Docker Compose configuration
- `.env.example` - Sample environment variables (copy to `.env` for customization)

```bash
# Copy and customize environment file
cp .env.example .env
# Edit .env with your preferred settings
nano .env
```

### VS Code Integration

The project includes VS Code tasks and debugging configuration:

- **Build**: `Ctrl+Shift+P` → "Tasks: Run Task" → "Docker: Build Image"
- **Run**: `Ctrl+Shift+P` → "Tasks: Run Task" → "Docker: Run Container (Test)"
- **Debug**: F5 to start debugging with breakpoints

Project structure:

```text
├── src/
│   ├── main.rs              # ONVIF service implementation
│   ├── onvif_responses.rs   # SOAP response templates
│   └── ws_discovery.rs      # WS-Discovery implementation
├── Dockerfile               # Multi-stage build
├── docker-compose.yml       # Docker Compose configuration
├── .env.example            # Sample environment variables
├── quick-start.sh          # Quick start script for common operations
├── entrypoint.sh           # Service orchestration
├── mediamtx.yml            # MediaMTX configuration
└── .vscode/                # VS Code configuration
    ├── tasks.json          # Build and run tasks
    └── launch.json         # Debug configuration
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

**Code Quality Guidelines:**

- This project uses AI-assisted development with GitHub Copilot
- All contributions should be reviewed and tested thoroughly before merging
- Please ensure proper error handling and documentation
- Follow existing code patterns and style conventions
- Add appropriate tests and update documentation as needed

Please make sure to update tests as appropriate and follow the existing code style.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Disclaimer**: This software is provided "AS IS" without warranty of any kind. The use of AI-generated code components requires additional validation and testing before deployment in production environments.

## Authors

- [@W4ff1e](https://github.com/W4ff1e) - Initial work and maintenance
- [GitHub Copilot](https://github.com/features/copilot) - Pair programming and code assistance

---

<!--markdownlint-disable-next-line -->
**Made with :yellow_heart: by [Waffle](https://github.com/W4ff1e) in collaboration with GitHub Copilot**
