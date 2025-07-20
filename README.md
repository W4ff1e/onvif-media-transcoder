# FFmpeg ONVIF Camera Emulator

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](Dockerfile)
[![ONVIF](https://img.shields.io/badge/ONVIF-compatible-green.svg)](https://www.onvif.org/)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## Overview

**FFmpeg ONVIF Camera Emulator** is a Docker-based solution that converts any input stream (HLS, MP4, RTSP, etc.) into a fully ONVIF-compatible camera device. It provides automatic network discovery, standardized media profiles, and authentication-protected endpoints for seamless integration with ONVIF clients.

## Table of Contents

- [FFmpeg ONVIF Camera Emulator](#ffmpeg-onvif-camera-emulator)
  - [Overview](#overview)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Quick Start](#quick-start)
    - [Docker Run](#docker-run)
    - [Environment Variables](#environment-variables)
  - [Architecture](#architecture)
  - [ONVIF Compatibility](#onvif-compatibility)
    - [Supported Endpoints](#supported-endpoints)
    - [Authentication](#authentication)
    - [Discovery](#discovery)
  - [Testing](#testing)
    - [ONVIF Clients](#onvif-clients)
    - [Manual Testing](#manual-testing)
  - [Development](#development)
    - [Prerequisites](#prerequisites)
    - [Building](#building)
    - [VS Code Integration](#vs-code-integration)
  - [Contributing](#contributing)
  - [License](#license)
  - [Authors](#authors)

## Features

- [x] **Universal Input Support**: Convert any FFmpeg-compatible input (HLS, MP4, RTSP, HTTP streams)
- [x] **ONVIF Compliance**: Full ONVIF Profile compatibility with standardized endpoints
- [x] **Network Discovery**: Native WS-Discovery implementation for automatic device detection
- [x] **Professional Streaming**:
  - H.264/AAC encoding with optimized settings
  - RTSP over TCP for reliable delivery
  - Real-time transcoding with low latency
- [x] **Authentication & Security**:
  - HTTP Basic and Digest authentication
  - Configurable credentials
  - Secure endpoint protection

## Quick Start

### Docker Run

```bash
# Build the image
docker build -t ffmpeg-onvif-emulator .

# Run with default demo stream
docker run --rm -p 8080:8080 -p 8554:8554 -p 3702:3702/udp ffmpeg-onvif-emulator

# Run with custom stream and credentials
docker run --rm -p 8080:8080 -p 8554:8554 -p 3702:3702/udp \
  -e INPUT_URL="https://your-stream.m3u8" \
  -e DEVICE_NAME="My-Custom-Camera" \
  -e ONVIF_USERNAME="myuser" \
  -e ONVIF_PASSWORD="mypassword" \
  ffmpeg-onvif-emulator
```

### Environment Variables

| Variable           | Default                 | Description                   |
| ------------------ | ----------------------- | ----------------------------- |
| `INPUT_URL`        | Demo HLS stream         | Source video stream URL       |
| `RTSP_OUTPUT_PORT` | `8554`                  | RTSP server port              |
| `ONVIF_PORT`       | `8080`                  | ONVIF web service port        |
| `DEVICE_NAME`      | `FFmpeg-ONVIF-Emulator` | Camera device name            |
| `RTSP_PATH`        | `/stream`               | RTSP stream path              |
| `ONVIF_USERNAME`   | `admin`                 | ONVIF authentication username |
| `ONVIF_PASSWORD`   | `onvif-rust`            | ONVIF authentication password |

**Note**: The emulator automatically detects the container IP and configures all services accordingly.

## Architecture

The emulator consists of four integrated components:

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

- **Supported Methods**: HTTP Basic Authentication, HTTP Digest Authentication
- **Default Credentials**: `admin` / `onvif-rust`
- **Security**: Device discovery endpoints allow unauthenticated access
- **Customization**: Configure via `ONVIF_USERNAME` and `ONVIF_PASSWORD`

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
# Test device discovery (no authentication)
curl -X POST http://localhost:8080/onvif/device_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>'

# Test authenticated endpoint
curl -X POST http://localhost:8080/onvif/media_service \
  -H "Content-Type: application/soap+xml" \
  -u admin:onvif-rust \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetProfiles/></soap:Body></soap:Envelope>'

# Test RTSP stream
ffprobe -v quiet -select_streams v:0 -show_entries stream=codec_name -of csv=p=0 rtsp://localhost:8554/stream
```

## Development

### Prerequisites

- [Docker](https://www.docker.com/get-started) for containerization
- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (for local development)
- [VS Code](https://code.visualstudio.com/) with Rust extensions (recommended)

### Building

```bash
# Clone the repository
git clone https://github.com/your-username/ffmpeg-onvif-emulator.git
cd ffmpeg-onvif-emulator

# Build with Docker
docker build -t ffmpeg-onvif-emulator .

# Or build Rust components locally
cargo build --release
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
├── entrypoint.sh           # Service orchestration
├── mediamtx.yml            # MediaMTX configuration
└── .vscode/                # VS Code configuration
    ├── tasks.json          # Build and run tasks
    └── launch.json         # Debug configuration
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate and follow the existing code style.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Authors

- [@W4ff1e](https://github.com/W4ff1e) - Initial work and maintenance
- [GitHub Copilot](https://github.com/features/copilot) - Pair programming and code assistance


---

<!--markdownlint-disable-next-line -->
**Made with :yellow_heart: by [Waffle](https://github.com/W4ff1e) in collaboration with GitHub Copilot**
