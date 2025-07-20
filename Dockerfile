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

# Build dependencies (this layer will be cached unless Cargo.toml/Cargo.lock changes)
RUN cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src

# Build the actual application (only rebuilds if source code changes)
RUN cargo build --release

# Runtime stage - minimal image for running the application
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ffmpeg

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/ffmpeg-onvif-emulator /usr/local/bin/

# Set environment variables with default values
ENV INPUT_URL="https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8"
ENV OUTPUT_PORT="8554"
ENV RTSP_OUTPUT_PORT="8554"
ENV RTSP_PATH="/stream"
ENV ONVIF_PORT="8080"
ENV DEVICE_NAME="FFmpeg-ONVIF-Emulator"
ENV ONVIF_USERNAME="admin"
ENV ONVIF_PASSWORD="onvif-rust"

# Expose the ports (TCP for ONVIF, UDP for WS-Discovery)
EXPOSE 8080 8554 3702/udp

# Set the default command to run both commands
CMD ["/entrypoint.sh"]
