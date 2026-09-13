# syntax=docker/dockerfile:1
# check=skip=SecretsUsedInArgOrEnv

ARG RUST_VERSION=1.98.1
ARG ALPINE_VERSION=3.22
ARG MEDIAMTX_VERSION=v1.21.0

# ---------------------------------------------------------------------------
# Build stage. rust:*-alpine targets musl natively, so a single `cargo build`
# produces a static binary for whichever platform BuildKit is building.
# ---------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS builder

# hadolint ignore=DL3018
RUN apk add --no-cache build-base

WORKDIR /app

# Compile dependencies first so they are cached independently of src/.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && : > src/lib.rs
RUN --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release --locked

# Build the real application. `touch` ensures cargo sees the copied sources
# as newer than the placeholder build in the cache mount.
COPY src ./src
RUN --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    find src -type f -exec touch {} + && \
    cargo build --release --locked && \
    mkdir -p /out && cp target/release/onvif-media-transcoder /out/

# ---------------------------------------------------------------------------
# Runtime stage.
# ---------------------------------------------------------------------------
FROM alpine:${ALPINE_VERSION}

ARG TARGETARCH
ARG MEDIAMTX_VERSION

SHELL ["/bin/ash", "-eo", "pipefail", "-c"]

# ffmpeg provides ffprobe (input validation, stream probing) and snapshots;
# curl is used by the HEALTHCHECK and to fetch MediaMTX.
# hadolint ignore=DL3018
RUN apk add --no-cache ca-certificates curl ffmpeg tzdata

# Install MediaMTX and verify it against the published checksums.
WORKDIR /tmp
RUN case "${TARGETARCH}" in \
        amd64|arm64) ;; \
        *) echo "Unsupported architecture: ${TARGETARCH} (supported: amd64, arm64)" && exit 1 ;; \
    esac && \
    ARCHIVE="mediamtx_${MEDIAMTX_VERSION}_linux_${TARGETARCH}.tar.gz" && \
    BASE="https://github.com/bluenviron/mediamtx/releases/download/${MEDIAMTX_VERSION}" && \
    curl -fsSL -o "${ARCHIVE}" "${BASE}/${ARCHIVE}" && \
    curl -fsSL -o checksums.sha256 "${BASE}/checksums.sha256" && \
    EXPECTED="$(grep -E "[ *]${ARCHIVE}\$" checksums.sha256 | awk '{print $1}')" && \
    ACTUAL="$(sha256sum "${ARCHIVE}" | awk '{print $1}')" && \
    if [ -z "${EXPECTED}" ] || [ "${EXPECTED}" != "${ACTUAL}" ]; then \
        echo "MediaMTX checksum mismatch: expected '${EXPECTED}' got '${ACTUAL}'" && exit 1; \
    fi && \
    tar -xzf "${ARCHIVE}" -C /usr/local/bin mediamtx && \
    rm -f "${ARCHIVE}" checksums.sha256 && \
    mediamtx --version
WORKDIR /

# Run as an unprivileged user. Ports below 1024 need --cap-add NET_BIND_SERVICE.
RUN addgroup -S -g 10001 onvif && adduser -S -D -H -G onvif -u 10001 onvif

COPY --chmod=755 entrypoint.sh /entrypoint.sh
COPY --from=builder /out/onvif-media-transcoder /usr/local/bin/onvif-media-transcoder

# Defaults; every value can be overridden at run time. See README.md.
# hadolint ignore=DL3064
ENV INPUT_URL="https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8" \
    RTSP_OUTPUT_PORT="8554" \
    RTSP_PATH="/stream" \
    ONVIF_PORT="8080" \
    DEVICE_NAME="ONVIF-Media-Transcoder" \
    ONVIF_USERNAME="admin" \
    ONVIF_PASSWORD="onvif-rust" \
    WS_DISCOVERY_ENABLED="true" \
    RTSP_AUTH_ENABLED="true" \
    INPUT_CHECK="warn" \
    DEBUG_LOGGING="false"

USER 10001:10001

EXPOSE 8080/tcp 8554/tcp 3702/udp

HEALTHCHECK --interval=30s --timeout=5s --start-period=40s --retries=3 \
    CMD ["/bin/sh", "-c", "curl -fsS \"http://127.0.0.1:${ONVIF_PORT}/\" > /dev/null || exit 1"]

ENTRYPOINT ["/entrypoint.sh"]
