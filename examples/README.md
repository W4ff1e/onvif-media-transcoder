# Examples

This directory contains example configurations and Docker Compose files for running the ONVIF Media Transcoder.

## Available Examples

- **`docker-compose.yml`** - Production-ready setup using published Docker Hub images
- **`docker-compose.local.yml`** - Local development setup with source code building
- **`.env.example`** - Example environment variables configuration

## Quick Start

### Using Published Images (Recommended)

1. Copy the environment example:

   ```bash
   cp examples/.env.example .env
   ```

2. Edit `.env` with your configuration

3. Run with Docker Compose:

   ```bash
   # Using published images from Docker Hub
   docker-compose -f examples/docker-compose.yml up
   ```

### Local Development

For development with local source code:

1. Build the image locally first:

   ```bash
   ./scripts/build.sh
   ```

2. Run the local development setup:

   ```bash
   # For local development with source building
   docker-compose -f examples/docker-compose.local.yml up
   ```

## Configuration Options

### Environment Variables

The `.env.example` file contains all available configuration options:

#### Stream Configuration

- `INPUT_URL` - Source video stream URL (HLS, MP4, RTSP, etc.)
- `RTSP_OUTPUT_PORT` - RTSP server output port (default: 8554)
- `RTSP_PATH` - RTSP stream path (default: /stream)

#### Service Configuration

- `ONVIF_PORT` - ONVIF web service port (default: 8080)
- `DEVICE_NAME` - Camera device name for identification
- `WS_DISCOVERY_ENABLED` - Enable automatic device discovery (default: true)

#### Authentication

- `ONVIF_USERNAME` - ONVIF authentication username (default: admin)
- `ONVIF_PASSWORD` - ONVIF authentication password (default: onvif-rust)

#### Debug Options

- `DEBUGLOGGING` - Enable verbose debug logging (default: false)

### Network Configuration

#### Host Network Mode (Recommended)

Both examples use `network_mode: host` which is recommended for:

- WS-Discovery multicast functionality
- Simplified port management
- Better network performance

#### Port Mapping Alternative

If host networking isn't available, you can modify the compose files to use port mapping:

```yaml
ports:
  - "8080:8080"   # ONVIF service
  - "8554:8554"   # RTSP stream
  - "3702:3702/udp"  # WS-Discovery
```

**Note:** Port mapping may limit WS-Discovery functionality across network boundaries.

## Usage Examples

### Basic Usage

```bash
# Start with default demo stream
docker-compose -f examples/docker-compose.yml up

# Start in background
docker-compose -f examples/docker-compose.yml up -d

# View logs
docker-compose -f examples/docker-compose.yml logs -f

# Stop services
docker-compose -f examples/docker-compose.yml down
```

### Custom Configuration

```bash
# Create custom environment
cat > .env << EOF
INPUT_URL=https://your-stream-url.m3u8
DEVICE_NAME=My-Custom-Camera
ONVIF_USERNAME=myuser
ONVIF_PASSWORD=mypassword
EOF

# Run with custom configuration
docker-compose -f examples/docker-compose.yml --env-file .env up
```

### Development Workflow

```bash
# Build and test locally
./scripts/build.sh
docker-compose -f examples/docker-compose.local.yml up

# Run tests
docker-compose -f examples/docker-compose.local.yml exec onvif-media-transcoder /app/scripts/test.sh

# Debug with logs
docker-compose -f examples/docker-compose.local.yml logs -f onvif-media-transcoder
```

## Health Checks

Both compose files include health checks to monitor service status:

- **ONVIF Service Health**: Checks device service endpoint availability
- **Automatic Restart**: Services restart automatically on failure
- **Startup Grace Period**: 10-second grace period for service initialization

## Resource Management

The examples include resource limits suitable for most deployments:

- **Memory Limit**: 1GB maximum, 512MB reserved
- **CPU Limit**: 2.0 cores maximum, 1.0 core reserved
- **Restart Policy**: `unless-stopped` for automatic recovery

Adjust these limits based on your specific requirements and available hardware.
