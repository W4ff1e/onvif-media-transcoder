#!/bin/sh
# Container entrypoint: validates the environment, generates the MediaMTX
# configuration, and supervises MediaMTX plus the ONVIF service.
#
# The ONVIF service reads its own settings from the environment, so this
# script only has to derive the values that depend on the container
# (reachable IP, RTSP URL) and drive MediaMTX.
set -eu

log() { printf '%s %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*"; }
die() { log "ERROR: $*"; exit 1; }

# ---------------------------------------------------------------------------
# Defaults (mirrored in the Dockerfile; present here so the script also works
# outside the image).
# ---------------------------------------------------------------------------
INPUT_URL="${INPUT_URL:-}"
RTSP_OUTPUT_PORT="${RTSP_OUTPUT_PORT:-8554}"
RTSP_PATH="${RTSP_PATH:-/stream}"
ONVIF_PORT="${ONVIF_PORT:-8080}"
DEVICE_NAME="${DEVICE_NAME:-ONVIF-Media-Transcoder}"
ONVIF_USERNAME="${ONVIF_USERNAME:-admin}"
ONVIF_PASSWORD="${ONVIF_PASSWORD:-onvif-rust}"
WS_DISCOVERY_ENABLED="${WS_DISCOVERY_ENABLED:-true}"
RTSP_AUTH_ENABLED="${RTSP_AUTH_ENABLED:-true}"
INPUT_CHECK="${INPUT_CHECK:-warn}"
# DEBUGLOGGING is the historical spelling; keep accepting it.
DEBUG_LOGGING="${DEBUG_LOGGING:-${DEBUGLOGGING:-false}}"
CONTAINER_IP="${CONTAINER_IP:-}"
MEDIAMTX_CONFIG="${MEDIAMTX_CONFIG:-/tmp/mediamtx.yml}"

# ---------------------------------------------------------------------------
# Validation
# ---------------------------------------------------------------------------
is_port() {
    case "$1" in
        ''|*[!0-9]*) return 1 ;;
    esac
    [ "$1" -ge 1 ] && [ "$1" -le 65535 ]
}

is_bool() {
    case "$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')" in
        true|false|yes|no|on|off|1|0|y|n) return 0 ;;
        *) return 1 ;;
    esac
}

is_true() {
    case "$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')" in
        true|yes|on|1|y) return 0 ;;
        *) return 1 ;;
    esac
}

is_ipv4() {
    printf '%s' "$1" | grep -Eq '^([0-9]{1,3}\.){3}[0-9]{1,3}$'
}

[ -n "$INPUT_URL" ] || die "INPUT_URL must be set (example: https://host/live/stream.m3u8)"
printf '%s' "$INPUT_URL" | grep -Eq '^[a-zA-Z][a-zA-Z0-9+.-]*://' \
    || die "INPUT_URL must be a URL with a scheme (rtsp://, rtsps://, rtmp://, http(s):// HLS, udp://, srt://). MediaMTX cannot read local files directly; serve them over HTTP or RTSP instead. Got: $INPUT_URL"

is_port "$RTSP_OUTPUT_PORT" || die "RTSP_OUTPUT_PORT must be 1-65535, got: $RTSP_OUTPUT_PORT"
is_port "$ONVIF_PORT" || die "ONVIF_PORT must be 1-65535, got: $ONVIF_PORT"
[ "$RTSP_OUTPUT_PORT" != "$ONVIF_PORT" ] || die "RTSP_OUTPUT_PORT and ONVIF_PORT must differ"

# Normalise the RTSP path: leading slash, no trailing slash, non-empty.
case "$RTSP_PATH" in
    /*) ;;
    *) RTSP_PATH="/$RTSP_PATH" ;;
esac
RTSP_PATH="$(printf '%s' "$RTSP_PATH" | sed 's:/*$::')"
[ -n "$RTSP_PATH" ] || RTSP_PATH="/stream"
STREAM_NAME="${RTSP_PATH#/}"
printf '%s' "$STREAM_NAME" | grep -Eq '^[A-Za-z0-9][A-Za-z0-9_./-]*$' \
    || die "RTSP_PATH may only contain letters, digits, '_', '-', '.' and '/', got: $RTSP_PATH"

[ -n "$(printf '%s' "$DEVICE_NAME" | tr -d '[:space:]')" ] || die "DEVICE_NAME must not be empty"
[ -n "$ONVIF_USERNAME" ] || die "ONVIF_USERNAME must not be empty"
[ -n "$ONVIF_PASSWORD" ] || die "ONVIF_PASSWORD must not be empty"
[ "${#ONVIF_PASSWORD}" -ge 6 ] || die "ONVIF_PASSWORD must be at least 6 characters"
if [ "$ONVIF_USERNAME" = "admin" ] && [ "$ONVIF_PASSWORD" = "onvif-rust" ]; then
    log "WARNING: using the default credentials admin/onvif-rust; set ONVIF_USERNAME and ONVIF_PASSWORD"
fi

is_bool "$WS_DISCOVERY_ENABLED" || die "WS_DISCOVERY_ENABLED must be true/false, got: $WS_DISCOVERY_ENABLED"
is_bool "$RTSP_AUTH_ENABLED" || die "RTSP_AUTH_ENABLED must be true/false, got: $RTSP_AUTH_ENABLED"
is_bool "$DEBUG_LOGGING" || die "DEBUG_LOGGING must be true/false, got: $DEBUG_LOGGING"
case "$INPUT_CHECK" in
    warn|fail|skip) ;;
    *) die "INPUT_CHECK must be warn, fail or skip, got: $INPUT_CHECK" ;;
esac

# ---------------------------------------------------------------------------
# Reachable IPv4 address, advertised in WS-Discovery and ONVIF responses.
# ---------------------------------------------------------------------------
detect_ipv4() {
    for addr in $(hostname -i 2>/dev/null); do
        if is_ipv4 "$addr" && [ "$addr" != "127.0.0.1" ]; then
            printf '%s' "$addr"
            return 0
        fi
    done
    ip -4 route get 1.1.1.1 2>/dev/null \
        | awk '{ for (i = 1; i <= NF; i++) if ($i == "src") { print $(i + 1); exit } }'
}

if [ -z "$CONTAINER_IP" ]; then
    CONTAINER_IP="$(detect_ipv4 || true)"
    if [ -z "$CONTAINER_IP" ]; then
        log "WARNING: could not determine an IPv4 address; defaulting to 127.0.0.1. Set CONTAINER_IP explicitly."
        CONTAINER_IP="127.0.0.1"
    fi
fi
is_ipv4 "$CONTAINER_IP" || die "CONTAINER_IP must be an IPv4 address, got: $CONTAINER_IP"

RTSP_STREAM_URL="rtsp://${CONTAINER_IP}:${RTSP_OUTPUT_PORT}${RTSP_PATH}"
RTP_PORT=$((RTSP_OUTPUT_PORT + 1000))
RTCP_PORT=$((RTSP_OUTPUT_PORT + 1001))

log "ONVIF Media Transcoder starting"
log "  input:          $INPUT_URL"
log "  container ip:   $CONTAINER_IP"
log "  rtsp output:    $RTSP_STREAM_URL (auth: $RTSP_AUTH_ENABLED)"
log "  onvif port:     $ONVIF_PORT"
log "  device name:    $DEVICE_NAME"
log "  username:       $ONVIF_USERNAME"
log "  ws-discovery:   $WS_DISCOVERY_ENABLED"
log "  debug logging:  $DEBUG_LOGGING"

# ---------------------------------------------------------------------------
# Optional input reachability check. MediaMTX reconnects on its own, so a
# failure is a warning by default (INPUT_CHECK=fail restores the hard stop).
# ---------------------------------------------------------------------------
if [ "$INPUT_CHECK" != "skip" ]; then
    log "checking input stream (up to 15s)..."
    if timeout 15 ffprobe -v error -rw_timeout 10000000 -show_format -i "$INPUT_URL" > /dev/null 2>&1; then
        log "input stream is reachable"
    elif [ "$INPUT_CHECK" = "fail" ]; then
        die "input stream is not reachable: $INPUT_URL"
    else
        log "WARNING: input stream is not reachable yet; MediaMTX will keep retrying"
    fi
fi

# ---------------------------------------------------------------------------
# MediaMTX configuration. Values are emitted as single-quoted YAML scalars so
# any character in URLs or passwords is safe.
# ---------------------------------------------------------------------------
yaml_quote() { printf "'%s'" "$(printf '%s' "$1" | sed "s/'/''/g")"; }

if is_true "$RTSP_AUTH_ENABLED"; then
    AUTH_USERS="  - user: $(yaml_quote "$ONVIF_USERNAME")
    pass: $(yaml_quote "$ONVIF_PASSWORD")
    ips: []
    permissions:
      - action: read
        path: $(yaml_quote "$STREAM_NAME")"
else
    AUTH_USERS="  - user: any
    pass:
    ips: []
    permissions:
      - action: read
        path: $(yaml_quote "$STREAM_NAME")"
fi

cat > "$MEDIAMTX_CONFIG" <<EOF
# Generated by entrypoint.sh; do not edit.
logLevel: info
logDestinations: [stdout]
readTimeout: 10s
writeTimeout: 10s
writeQueueSize: 1024
udpMaxPayloadSize: 1472

api: no
metrics: no
pprof: no
playback: no
rtmp: no
hls: no
webrtc: no
srt: no
moq: no

rtsp: yes
rtspTransports: [udp, tcp]
rtspEncryption: "no"
rtspAddress: :${RTSP_OUTPUT_PORT}
rtpAddress: :${RTP_PORT}
rtcpAddress: :${RTCP_PORT}
rtspAuthMethods: [basic]

authMethod: internal
authInternalUsers:
${AUTH_USERS}

paths:
  $(yaml_quote "$STREAM_NAME"):
    source: $(yaml_quote "$INPUT_URL")
    sourceOnDemand: no
    rtspTransport: tcp
    record: no
EOF
log "MediaMTX configuration written to $MEDIAMTX_CONFIG"

# ---------------------------------------------------------------------------
# Process supervision
# ---------------------------------------------------------------------------
MTX_PID=""
ONVIF_PID=""

shutdown() {
    log "shutting down..."
    [ -n "$ONVIF_PID" ] && kill -TERM "$ONVIF_PID" 2>/dev/null || true
    [ -n "$MTX_PID" ] && kill -TERM "$MTX_PID" 2>/dev/null || true
    [ -n "$ONVIF_PID" ] && wait "$ONVIF_PID" 2>/dev/null || true
    [ -n "$MTX_PID" ] && wait "$MTX_PID" 2>/dev/null || true
    log "stopped"
    exit 0
}
trap shutdown TERM INT

log "starting MediaMTX..."
mediamtx "$MEDIAMTX_CONFIG" &
MTX_PID=$!

# Wait for the RTSP listener so clients never see a half-started device.
attempt=0
until netstat -ltn 2>/dev/null | grep -q ":${RTSP_OUTPUT_PORT} "; do
    attempt=$((attempt + 1))
    kill -0 "$MTX_PID" 2>/dev/null || die "MediaMTX exited during startup"
    [ "$attempt" -le 40 ] || { log "WARNING: MediaMTX is not listening on ${RTSP_OUTPUT_PORT} after 20s; continuing"; break; }
    sleep 0.5
done
log "MediaMTX is listening on :${RTSP_OUTPUT_PORT}"

log "starting ONVIF service..."
export RTSP_STREAM_URL CONTAINER_IP ONVIF_PORT DEVICE_NAME ONVIF_USERNAME ONVIF_PASSWORD \
    WS_DISCOVERY_ENABLED DEBUG_LOGGING
onvif-media-transcoder &
ONVIF_PID=$!

log "ready: ONVIF http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/device_service, RTSP ${RTSP_STREAM_URL}"

# Supervise both processes; exit non-zero if either dies so the container
# runtime can restart us. `sleep` runs in the background so trapped signals
# interrupt the wait immediately.
while kill -0 "$MTX_PID" 2>/dev/null && kill -0 "$ONVIF_PID" 2>/dev/null; do
    sleep 1 &
    wait $! || true
done

if ! kill -0 "$MTX_PID" 2>/dev/null; then
    log "ERROR: MediaMTX exited unexpectedly"
else
    log "ERROR: ONVIF service exited unexpectedly"
fi
kill -TERM "$MTX_PID" "$ONVIF_PID" 2>/dev/null || true
wait "$MTX_PID" 2>/dev/null || true
wait "$ONVIF_PID" 2>/dev/null || true
exit 1
