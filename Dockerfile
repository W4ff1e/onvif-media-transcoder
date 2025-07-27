# syntax=docker/dockerfile:1
# check=skip=SecretsUsedInArgOrEnv

# Build stage - for compiling Rust dependencies and application
FROM rust:alpine AS builder

# Declare build arguments for cross-platform support
ARG TARGETARCH

# Install build tools and cross-compilation targets
RUN apk add --no-cache build-base musl-dev

# Add Rust targets for cross-compilation
RUN case ${TARGETARCH} in \
    "amd64") rustup target add x86_64-unknown-linux-musl ;; \
    "arm64") rustup target add aarch64-unknown-linux-musl ;; \
    *) echo "Unsupported architecture: ${TARGETARCH}. Supported: amd64, arm64" && exit 1 ;; \
    esac

# Set working directory
WORKDIR /app

# Copy dependency files first (for Docker layer caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy source structure to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Pre-compile dependencies with cache mount for faster subsequent builds
RUN --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    case ${TARGETARCH} in \
    "amd64") cargo build --release --target x86_64-unknown-linux-musl ;; \
    "arm64") cargo build --release --target aarch64-unknown-linux-musl ;; \
    esac

# Remove dummy source
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Build the actual application with cache mount
RUN --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    case ${TARGETARCH} in \
    "amd64") cargo build --release --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/onvif-media-transcoder /tmp/onvif-media-transcoder ;; \
    "arm64") cargo build --release --target aarch64-unknown-linux-musl && \
    cp target/aarch64-unknown-linux-musl/release/onvif-media-transcoder /tmp/onvif-media-transcoder ;; \
    esac

# Runtime stage - minimal image for running the application
FROM alpine:latest

# Declare build arguments for cross-platform support
ARG TARGETARCH

# Declare version of MediaMTX to install
# This can be overridden at build time with --build-arg MEDIAMTX_VERSION=vX.Y.Z
# Default version is set to v1.13.0
ARG MEDIAMTX_VERSION=v1.13.0

# Install runtime dependencies including image processing libraries
RUN apk add --no-cache curl ffmpeg musl-dev

# Download and install MediaMTX (architecture-aware)
RUN case ${TARGETARCH} in \
    "amd64") MEDIAMTX_ARCH="amd64" ;; \
    "arm64") MEDIAMTX_ARCH="arm64" ;; \
    *) echo "Unsupported architecture: ${TARGETARCH}. Supported: amd64, arm64" && exit 1 ;; \
    esac && \
    echo "Downloading MediaMTX ${MEDIAMTX_VERSION} for architecture: ${MEDIAMTX_ARCH}" && \
    curl -L "https://github.com/bluenviron/mediamtx/releases/download/${MEDIAMTX_VERSION}/mediamtx_${MEDIAMTX_VERSION}_linux_${MEDIAMTX_ARCH}.tar.gz" \
    | tar -xz -C /usr/local/bin/ mediamtx && \
    echo "MediaMTX ${MEDIAMTX_VERSION} installation completed for ${MEDIAMTX_ARCH}"

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
ENV DEBUGLOGGING="false"

# Expose the ports (TCP for ONVIF, UDP for WS-Discovery) - respects build-time configuration
EXPOSE ${ONVIF_PORT} ${RTSP_OUTPUT_PORT} 3702/udp

# Set the default command to kick off the entrypoint script
# This script will handle the configuration and start the MediaMTX server
# and the ONVIF Media Transcoder application.
CMD ["/entrypoint.sh"]
