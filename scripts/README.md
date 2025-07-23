# Scripts

This directory contains utility scripts for building, testing, and publishing the ONVIF Media Transcoder.

## Available Scripts

- **`build.sh`** - Comprehensive Docker build script with multi-architecture support
- **`publish.sh`** - Docker image publishing script for Docker Hub
- **`quick-start.sh`** - Quick setup and testing script
- **`env-setup.sh`** - Environment variable management for local development
- **`run-local.sh`** - Direct run script with environment variables pre-configured

## Usage

Make scripts executable before running:

```bash
chmod +x scripts/*.sh
```

### Build Script

```bash
./scripts/build.sh [options]
```

### Publish Script

```bash
./scripts/publish.sh [options]
```

### Quick Start

```bash
./scripts/quick-start.sh
```

### Environment Setup

For local development, use the environment setup script:

```bash
# Set environment variables in current shell
source ./scripts/env-setup.sh set

# Check status
source ./scripts/env-setup.sh status

# Run cargo
cargo run

# Cleanup when done
source ./scripts/env-setup.sh cleanup
```

### Direct Local Run

For the simplest local testing experience:

```bash
./scripts/run-local.sh
```

This script automatically sets the required environment variables and runs `cargo run`.

For detailed usage information, run each script with the `--help` flag.
