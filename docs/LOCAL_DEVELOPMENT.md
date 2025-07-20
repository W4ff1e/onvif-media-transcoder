# Local Development and Testing

This guide explains how to run and test the ONVIF Media Transcoder locally without Docker.

## Prerequisites

- [Rust](https://rustlang.org) 1.70 or later
- VS Code with Rust extensions (recommended)

## Local Development Setup

### Option 1: Using VS Code Tasks (Recommended)

1. Open the project in VS Code
2. Use `Ctrl+Shift+P` → "Tasks: Run Task" and choose:
   - **"Rust: Run Local (Demo Stream)"** - Runs with demo HLS stream
   - **"Rust: Test"** - Runs all unit tests
   - **"Rust: Check"** - Checks code without running

### Option 2: Manual Environment Setup

Copy the local environment template:

```bash
cp .env.local .env
```

Edit `.env` with your preferred settings, then run:

```bash
# Load environment variables and run
source .env && cargo run

# Or export manually
export RTSP_INPUT="rtsp://127.0.0.1:8554/stream"
export ONVIF_PORT="8080"
export DEVICE_NAME="Local-ONVIF-Transcoder"
export ONVIF_USERNAME="admin"
export ONVIF_PASSWORD="onvif-rust"
cargo run
```

## Running Tests

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_authentication_integration
```

### Integration Testing

For integration testing, you'll need a running MediaMTX server or external stream:

```bash
# Start MediaMTX (if testing locally)
# Then run the ONVIF service
RTSP_INPUT="rtsp://127.0.0.1:8554/your-stream" cargo run
```

## Test Coverage

The test suite covers:

- ✅ **Authentication**: Basic Auth, Digest Auth, WS-Security
- ✅ **Configuration**: Environment variable parsing and validation
- ✅ **ONVIF Endpoints**: Public vs protected endpoint detection
- ✅ **Request Parsing**: Authorization header extraction
- ✅ **Error Handling**: Unsupported endpoint detection

## VS Code Integration

The project includes pre-configured VS Code tasks:

- **Build Tasks**: Standard Rust build and release builds
- **Test Tasks**: Unit tests and code checking
- **Local Run Tasks**: Pre-configured environment for local testing
- **Docker Tasks**: Container building and testing

Access tasks via `Ctrl+Shift+P` → "Tasks: Run Task"

## Environment Variables

| Variable         | Default                  | Description             |
| ---------------- | ------------------------ | ----------------------- |
| `RTSP_INPUT`     | Required                 | Source stream URL       |
| `ONVIF_PORT`     | `8080`                   | ONVIF service port      |
| `DEVICE_NAME`    | `Local-ONVIF-Transcoder` | Device identifier       |
| `ONVIF_USERNAME` | `admin`                  | Authentication username |
| `ONVIF_PASSWORD` | `onvif-rust`             | Authentication password |

## Testing ONVIF Endpoints

Once running locally, test with:

```bash
# Test device capabilities (no auth)
curl -X POST http://localhost:8080/onvif/device_service \
  -H "Content-Type: application/soap+xml" \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>'

# Test media profiles (with auth)
curl -X POST http://localhost:8080/onvif/media_service \
  -H "Content-Type: application/soap+xml" \
  -u admin:onvif-rust \
  -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetProfiles/></soap:Body></soap:Envelope>'
```

## Debugging

Use VS Code's integrated debugger:

1. Set breakpoints in the code
2. Press `F5` to start debugging
3. VS Code will prompt for environment variables if needed

The debugger is pre-configured with the necessary environment variables for local testing.
