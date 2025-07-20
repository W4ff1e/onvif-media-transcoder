# Scripts

This directory contains utility scripts for building, testing, and publishing the ONVIF Media Transcoder.

## Available Scripts

- **`build.sh`** - Comprehensive Docker build script with multi-architecture support
- **`publish.sh`** - Docker image publishing script for Docker Hub
- **`quick-start.sh`** - Quick setup and testing script

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

For detailed usage information, run each script with the `--help` flag.
