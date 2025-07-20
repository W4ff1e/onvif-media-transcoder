# Examples

This directory contains example configurations and Docker Compose files for running the ONVIF Media Transcoder.

## Available Examples

- **`docker-compose.yml`** - Basic Docker Compose setup for local development
- **`docker-compose.hub.yml`** - Docker Compose using published images from Docker Hub
- **`.env.example`** - Example environment variables configuration

## Quick Start

1. Copy the environment example:

   ```bash
   cp examples/.env.example .env
   ```

2. Edit `.env` with your configuration

3. Run with Docker Compose:

   ```bash
   # For local development
   docker-compose -f examples/docker-compose.yml up
   
   # Using published images
   docker-compose -f examples/docker-compose.hub.yml up
   ```

## Environment Variables

See `.env.example` for all available configuration options including:

- Input stream URLs
- RTSP output configuration
- ONVIF service settings
- Authentication credentials
