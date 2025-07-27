# Scripts

This directory contains utility scripts for building, testing, and publishing the ONVIF Media Transcoder.

## Available Scripts

- **`build.sh`** - Comprehensive Docker build script with multi-architecture support
- **`publish.sh`** - Docker image publishing script for Docker Hub
- **`quick-start.sh`** - Quick setup and testing script with multiple commands

## Usage

Make scripts executable before running:

```bash
chmod +x scripts/*.sh
```

### Build Script (`build.sh`)

Builds Docker images with support for multiple architectures:

```bash
./scripts/build.sh [options]
```

**Features:**

- Multi-architecture support (amd64, arm64)
- Caching optimization
- Security scanning integration
- Build argument customization

### Publish Script (`publish.sh`)

Publishes Docker images to Docker Hub:

```bash
./scripts/publish.sh [options]
```

**Features:**

- Automated tagging (latest, version-specific, unstable)
- Multi-platform image publishing
- Registry authentication handling
- Release workflow integration

### Quick Start Script (`quick-start.sh`)

Provides quick commands for common development and testing operations:

```bash
./scripts/quick-start.sh [command]
```

**Available Commands:**

- `setup` - Create .env file from template
- `build` - Build the Docker image locally
- `run` - Run with default configuration
- `compose` - Start using Docker Compose
- `test` - Run integration tests
- `stop` - Stop running containers
- `clean` - Clean up containers and images
- `logs` - Show container logs
- `help` - Show detailed help

**Examples:**

```bash
# Quick setup and run
./scripts/quick-start.sh setup
./scripts/quick-start.sh run

# Development workflow
./scripts/quick-start.sh build
./scripts/quick-start.sh compose

# Testing and debugging
./scripts/quick-start.sh test
./scripts/quick-start.sh logs
```

## Prerequisites

- Docker and Docker Compose
- Bash shell environment
- Network access for pulling dependencies
- For publishing: Docker Hub credentials configured

## Environment Variables

Scripts respect the following environment variables:

- `DOCKER_REGISTRY` - Docker registry URL (default: docker.io)
- `IMAGE_NAME` - Image name (default: w4ff1e/onvif-media-transcoder)
- `BUILD_ARGS` - Additional Docker build arguments
- `COMPOSE_FILE` - Docker Compose file to use

## CI/CD Integration

These scripts are designed to work in both local development and CI/CD environments:

- **GitHub Actions** - Automated builds and publishing
- **Local Development** - Manual testing and debugging
- **Production Deployment** - Streamlined container management
