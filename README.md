# FFmpeg ONVIF Camera Emulator

A Docker-based ONVIF camera emulator that converts any input stream (especially .m3u8 HLS streams) to RTSP and exposes it as an ONVIF-compatible camera for network discovery.

## Overview

This project creates a virtual ONVIF camera that:
- Accepts any video input URL (HLS, MP4, etc.)
- Converts it to standardized RTSP stream using FFmpeg
- Exposes ONVIF web service endpoints for camera discovery and control
- Supports WS-Discovery for automatic network device detection
- Provides standard ONVIF media profiles and streaming capabilities

## Architecture

The emulator consists of three main components:

1. **FFmpeg Stream Converter**: Converts input streams to RTSP H.264/AAC format
2. **ONVIF Web Service**: Rust-based HTTP server implementing ONVIF SOAP endpoints
3. **WSDD Discovery Service**: WS-Discovery daemon for automatic network discovery

## Quick Start

### Build and Run

```bash
# Build the Docker image
docker build -t ffmpeg-onvif-emulator .

# Run with default demo stream
docker run -p 8080:8080 -p 8554:8554 ffmpeg-onvif-emulator

# Run with custom input stream
docker run -p 8080:8080 -p 8554:8554 \
  -e INPUT_URL="your-stream-url.m3u8" \
  -e DEVICE_NAME="My-Camera" \
  ffmpeg-onvif-emulator
```

### Environment Variables

| Variable      | Default                 | Description             |
| ------------- | ----------------------- | ----------------------- |
| `INPUT_URL`   | Demo HLS stream         | Source video stream URL |
| `OUTPUT_PORT` | `8554`                  | RTSP output port        |
| `ONVIF_PORT`  | `8080`                  | ONVIF web service port  |
| `DEVICE_NAME` | `FFmpeg-ONVIF-Emulator` | Camera device name      |
| `RTSP_PATH`   | `stream`                | RTSP stream path        |

## ONVIF Compatibility

The emulator implements standard ONVIF endpoints:

### Device Service
- `GetCapabilities` - Device and service capabilities
- `GetDeviceInformation` - Device model, firmware, serial number
- `GetServiceCapabilities` - Service-specific capabilities

### Media Service  
- `GetProfiles` - Video/audio encoding profiles
- `GetStreamUri` - RTSP stream URI for media playback
- `GetVideoSources` - Available video sources and resolutions

### Discovery
- **WS-Discovery (WSDD)**: Automatic network discovery
- **Device Type**: `NetworkVideoTransmitter`
- **ONVIF Profile**: Streaming compliant

## Stream Configuration

The FFmpeg conversion uses optimized settings:

- **Video**: H.264, 1080p, 30fps, 4Mbps max bitrate
- **Audio**: AAC, 48kHz, 128kbps
- **Transport**: RTSP over TCP
- **Latency**: Optimized for real-time streaming

## Network Discovery

The camera is discoverable through:

1. **WS-Discovery**: Standard ONVIF discovery protocol
2. **SOAP Endpoints**: Direct HTTP access to ONVIF services
3. **RTSP Stream**: Direct access for testing (rtsp://host:8554/stream)

## Testing

### ONVIF Discovery Tools
- ONVIF Device Manager
- VLC Media Player (Network Stream)
- FFprobe: `ffprobe rtsp://localhost:8554/stream`

### Manual SOAP Testing
```bash
# Test GetCapabilities
curl -X POST http://localhost:8080/onvif/device_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>'
```

## Development

### VS Code Debugging

Configured VS Code tasks and debugging:

```bash
# Build container
Ctrl+Shift+P -> "Tasks: Run Task" -> "Docker Build"

# Run with test stream  
Ctrl+Shift+P -> "Tasks: Run Task" -> "Docker Run"

# Debug build (experimental)
F5 -> "Debug Docker Build"
```

### Project Structure

```
├── src/
│   └── main.rs          # ONVIF web service (Rust)
├── Dockerfile           # Multi-stage build
├── entrypoint.sh        # Service orchestration  
├── Cargo.toml          # Rust dependencies
└── .vscode/            # VS Code configuration
    ├── tasks.json      # Build/run tasks
    └── launch.json     # Debug configuration
```

## Based on Reference Implementation

This implementation draws inspiration from [fcabrera23/onvif-camera-mock](https://github.com/fcabrera23/onvif-camera-mock), adapting the proven ONVIF architecture to a streamlined Rust + FFmpeg solution.

## License

MIT License - See LICENSE file for details.
