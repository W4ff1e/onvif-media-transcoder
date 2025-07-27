# Local Development and Testing

This guide explains how to run and test the ONVIF Media Transcoder locally without Docker.

## Prerequisites

- [Rust](https://rustlang.org) 1.70 or later
- VS Code with Rust extensions (recommended)
- [MediaMTX](https://github.com/bluenviron/mediamtx) (for stream re-muxing, if testing with local streams)

## Local Development Setup

### Option 1: Using VS Code Tasks (Recommended)

1. Open the project in VS Code
2. Use `Ctrl+Shift+P` → "Tasks: Run Task" and choose:
   - **"Rust: Run Local (Demo Stream)"** - Runs with demo HLS stream
   - **"Rust: Test"** - Runs all unit tests
   - **"Rust: Check"** - Checks code without running
   - **"Rust: Build"** - Builds the project
   - **"Rust: Build Release"** - Builds optimized release version

### Option 2: Command Line with Environment Variables

The application supports both environment variables and command-line arguments:

```bash
# Using command-line arguments (recommended for local development)
cargo run -- \
  --rtsp-stream-url "rtsp://127.0.0.1:8554/stream" \
  --onvif-port "8080" \
  --device-name "Local-ONVIF-Transcoder" \
  --onvif-username "admin" \
  --onvif-password "onvif-rust" \
  --container-ip "127.0.0.1" \
  --ws-discovery-enabled \
  --debug

# Show help for all available options
cargo run -- --help
```

### Option 3: Local Development without MediaMTX

For testing ONVIF functionality without setting up MediaMTX:

```bash
# Point to any accessible RTSP stream (even if it doesn't exist)
# The ONVIF service will start and provide endpoints
cargo run -- \
  --rtsp-stream-url "rtsp://localhost:8554/nonexistent" \
  --onvif-port "8080" \
  --device-name "Test-Device"
```

## Running Tests

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test authentication

# Run tests in parallel (default) or serial
cargo test -- --test-threads=1
```

### Integration Testing

For integration testing, you'll need a running stream source. You can use MediaMTX or any RTSP-compatible stream:

```bash
```bash
# Example with MediaMTX or any RTSP-compatible stream
# 1. Start your stream source (MediaMTX, camera, etc.)
# 2. Run the ONVIF service pointing to the stream
cargo run -- --rtsp-stream-url "rtsp://127.0.0.1:8554/your-stream"

# For testing ONVIF endpoints without a real stream:
# The ONVIF service will provide device information and media profiles
# even if the RTSP stream is not accessible (streams are referenced, not validated)
cargo run -- --rtsp-stream-url "rtsp://localhost:8554/test-stream"
```

## Test Coverage

The test suite covers:

- ✅ **Authentication**: Basic Auth, Digest Auth, WS-Security
- ✅ **Configuration**: Environment variable parsing and validation
- ✅ **ONVIF Endpoints**: Public vs protected endpoint detection
- ✅ **Request Parsing**: Authorization header extraction and validation
- ✅ **Error Handling**: Unsupported endpoint detection and error responses
- ✅ **Command Line Interface**: Argument parsing and help generation

## VS Code Integration

The project includes pre-configured VS Code tasks and debugging:

### Available Tasks

- **Build Tasks**: Standard Rust build and release builds
- **Test Tasks**: Unit tests and code checking
- **Local Run Tasks**: Pre-configured environment for local testing
- **Docker Tasks**: Container building and testing

Access tasks via `Ctrl+Shift+P` → "Tasks: Run Task"

### Debugging

1. Set breakpoints in the code
2. Press `F5` to start debugging
3. VS Code will use the pre-configured debug settings with environment variables

The debugger is configured with default environment variables for local testing.

## Configuration Options

### Environment Variables (Docker vs Local Development)

**For Docker deployments**, use these environment variables (handled by entrypoint.sh):

| Variable               | Default                  | Description                       |
| ---------------------- | ------------------------ | --------------------------------- |
| `INPUT_URL`            | Demo HLS stream          | Source stream URL for MediaMTX    |
| `RTSP_OUTPUT_PORT`     | `8554`                   | RTSP server output port           |
| `RTSP_PATH`            | `/stream`                | RTSP stream path                  |
| `ONVIF_PORT`           | `8080`                   | ONVIF service port                |
| `DEVICE_NAME`          | `ONVIF-Media-Transcoder` | Device identifier                 |
| `ONVIF_USERNAME`       | `admin`                  | Authentication username           |
| `ONVIF_PASSWORD`       | `onvif-rust`             | Authentication password           |
| `WS_DISCOVERY_ENABLED` | `true`                   | Enable WS-Discovery service       |
| `DEBUGLOGGING`         | `false`                  | Enable debug logging (sensitive!) |

**For local development**, the Rust application uses command-line arguments:

| CLI Argument             | Default                        | Description                       |
| ------------------------ | ------------------------------ | --------------------------------- |
| `--rtsp-stream-url`      | `rtsp://127.0.0.1:8554/stream` | Source RTSP stream URL            |
| `--onvif-port`           | `8080`                         | ONVIF service port                |
| `--device-name`          | `ONVIF-Media-Transcoder`       | Device identifier                 |
| `--onvif-username`       | `admin`                        | Authentication username           |
| `--onvif-password`       | `onvif-rust`                   | Authentication password           |
| `--container-ip`         | `127.0.0.1`                    | IP address for service binding    |
| `--ws-discovery-enabled` | (flag)                         | Enable WS-Discovery service       |
| `--debug`                | (flag)                         | Enable debug logging (sensitive!) |

### Command Line Arguments

All environment variables have corresponding command-line arguments. Use `cargo run -- --help` to see all available options.

## Testing ONVIF Endpoints

Once running locally, test with:

```bash
# Test device capabilities (no auth required)
curl -X POST http://localhost:8080/onvif/device_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>'

# Test media profiles (with Basic Auth)
curl -X POST http://localhost:8080/onvif/media_service \
  -H "Content-Type: application/soap+xml" \
  -u admin:onvif-rust \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetProfiles/></soap:Body></soap:Envelope>'

# Test with WS-Security authentication
curl -X POST http://localhost:8080/onvif/media_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><soap:Header><wsse:Security><wsse:UsernameToken><wsse:Username>admin</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordText">onvif-rust</wsse:Password></wsse:UsernameToken></wsse:Security></soap:Header><soap:Body><GetProfiles/></soap:Body></soap:Envelope>'
```

## Development Workflow

### Typical Development Cycle

1. **Code Changes**: Edit Rust source files
2. **Check Syntax**: `cargo check` (fast syntax checking)
3. **Run Tests**: `cargo test` (verify functionality)
4. **Local Testing**: `cargo run` (test with real requests)
5. **Docker Testing**: `./scripts/build.sh && docker run ...` (container testing)

### Performance Testing

```bash
# Build optimized version
cargo build --release

# Run with release build
./target/release/onvif-media-transcoder --help

# Profile with tools
cargo install flamegraph
sudo cargo flamegraph --root -- --onvif-port 8080
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check dependencies
cargo audit

# Generate documentation
cargo doc --open
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**: Change `ONVIF_PORT` or stop conflicting services
2. **Stream Not Found**: Verify the RTSP stream URL is accessible and the source is running
3. **Authentication Failing**: Check command-line arguments for username/password
4. **WS-Discovery Not Working**: Ensure proper network interfaces and permissions (may require sudo on some systems)

### Debug Logging

Enable debug logging for detailed request/response information:

```bash
# Enable debug logging (WARNING: logs sensitive information)
cargo run -- --debug

# Or use RUST_LOG for more granular control
RUST_LOG=debug cargo run
```

**Warning**: Debug logging may expose sensitive authentication information. Only use in development environments.
