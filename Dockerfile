# syntax=docker/dockerfile:1
# check=skip=SecretsUsedInArgOrEnv

# Build stage - for compiling Rust dependencies and application
FROM rust:alpine AS builder

# Install build tools
RUN apk add --no-cache build-base musl-dev

# Set working directory
WORKDIR /app

# Copy dependency files first (for Docker layer caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies with cache mount (this will cache Rust dependencies)
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src

# Build the actual application with cache mount
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release && \
    cp target/release/onvif-media-transcoder /tmp/onvif-media-transcoder

# Runtime stage - minimal image for running the application
FROM alpine:latest

# Install runtime dependencies including image processing libraries
RUN apk add --no-cache \
    curl \
    ffmpeg \
    musl-dev

# Download and install MediaMTX
RUN curl -L https://github.com/bluenviron/mediamtx/releases/download/v1.13.0/mediamtx_v1.13.0_linux_amd64.tar.gz \
    | tar -xz -C /usr/local/bin/ mediamtx

# Copy configuration files
COPY entrypoint.sh /entrypoint.sh
COPY mediamtx.yml /etc/mediamtx.yml
RUN chmod +x /entrypoint.sh

# Copy the built binary from the builder stage
COPY --from=builder /tmp/onvif-media-transcoder /usr/local/bin/

# Set environment variables with default values
ENV INPUT_URL="https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8"
ENV RTSP_OUTPUT_PORT="8554"
ENV RTSP_PATH="/stream"
ENV ONVIF_PORT="8080"
ENV DEVICE_NAME="ONVIF-Media-Transcoder"
ENV ONVIF_USERNAME="admin"
ENV ONVIF_PASSWORD="onvif-rust"
ENV WS_DISCOVERY_ENABLED="true"

# Expose the ports (TCP for ONVIF, UDP for WS-Discovery)
EXPOSE 8080 8554 3702/udp

# Set the default command to run both commands
CMD ["/entrypoint.sh"]
