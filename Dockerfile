# Use the official Rust Alpine image as base
FROM rust:alpine

# Install build tools, ffmpeg, wsdd, and development libraries
RUN apk add --no-cache ffmpeg build-base musl-dev wsdd


# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Copy source code
COPY . /app
WORKDIR /app

# Build the Rust crate
RUN cargo build --release

# Set environment variables with default values
ENV INPUT_URL="https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8"
ENV OUTPUT_PORT="8554"
ENV RTSP_PATH="stream"
ENV ONVIF_PORT="8080"
ENV DEVICE_NAME="FFmpeg-ONVIF-Emulator"

# Expose the ports
EXPOSE 8080 8554

# Set the default command to run both commands
CMD ["/entrypoint.sh"]
