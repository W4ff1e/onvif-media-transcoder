#!/usr/bin/env bash
# End-to-end test of a built image without any external network dependency.
#
# A second MediaMTX instance inside the container acts as the input source,
# fed by an ffmpeg test pattern, so the real entrypoint runs unmodified.
#
# Usage: scripts/e2e-test.sh [IMAGE]   (default: onvif-media-transcoder:test)
set -euo pipefail

IMAGE="${1:-onvif-media-transcoder:test}"
NAME="omt-e2e-$$"
ONVIF_PORT_HOST="${ONVIF_PORT_HOST:-18080}"
RTSP_PORT_HOST="${RTSP_PORT_HOST:-18554}"
WSD_PORT_HOST="${WSD_PORT_HOST:-13702}"
USER_NAME="tester"
PASSWORD="e2e-p@ss&word"
DEVICE="E2E Camera"

pass() { echo "PASS  $*"; }
fail() { echo "FAIL  $*"; FAILED=1; }
FAILED=0

cleanup() {
    docker rm -f "$NAME" > /dev/null 2>&1 || true
}
trap cleanup EXIT

# The wrapper starts the in-container source, then hands over to the real
# entrypoint exactly as a production container would run it.
read -r -d '' SOURCE_WRAPPER <<'EOF' || true
set -e
printf 'rtspAddress: :9554\nrtpAddress: :9700\nrtcpAddress: :9701\nrtmp: no\nhls: no\nwebrtc: no\nsrt: no\nmoq: no\napi: no\nlogLevel: warn\npaths:\n  src: {}\n' > /tmp/src.yml
mediamtx /tmp/src.yml > /tmp/src-mediamtx.log 2>&1 &
sleep 2
ffmpeg -nostdin -loglevel error -re -f lavfi -i testsrc=size=640x360:rate=15 \
  -c:v libx264 -preset ultrafast -tune zerolatency -g 30 -pix_fmt yuv420p \
  -f rtsp -rtsp_transport tcp rtsp://127.0.0.1:9554/src > /tmp/ffmpeg.log 2>&1 &
sleep 3
export INPUT_URL=rtsp://127.0.0.1:9554/src
exec /entrypoint.sh
EOF

echo "== starting $IMAGE as $NAME"
docker run -d --name "$NAME" \
    -p "${ONVIF_PORT_HOST}:8080" -p "${RTSP_PORT_HOST}:8554" -p "${WSD_PORT_HOST}:3702/udp" \
    -e ONVIF_USERNAME="$USER_NAME" -e ONVIF_PASSWORD="$PASSWORD" -e DEVICE_NAME="$DEVICE" \
    -e INPUT_CHECK=fail \
    --entrypoint sh "$IMAGE" -c "$SOURCE_WRAPPER" > /dev/null

echo "== waiting for the ONVIF service"
for i in $(seq 1 60); do
    if curl -fsS -m 2 "http://127.0.0.1:${ONVIF_PORT_HOST}/" > /dev/null 2>&1; then
        pass "ONVIF service answered after ~$((i * 2))s"
        break
    fi
    if [ "$(docker inspect -f '{{.State.Running}}' "$NAME")" != "true" ]; then
        docker logs "$NAME" 2>&1 | tail -40
        echo "FAIL  container exited during startup"
        exit 1
    fi
    if [ "$i" -eq 60 ]; then
        docker logs "$NAME" 2>&1 | tail -40
        echo "FAIL  ONVIF service did not start within 120s"
        exit 1
    fi
    sleep 2
done

ONVIF="http://127.0.0.1:${ONVIF_PORT_HOST}"
soap() { printf '<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body>%s</s:Body></s:Envelope>' "$1"; }
post() { # post <path> <body> [curl args...]
    local path="$1" body="$2"; shift 2
    curl -sS -m 10 -X POST "${ONVIF}${path}" -H 'Content-Type: application/soap+xml' --data-binary "$body" "$@"
}
code() { post "$1" "$2" -o /dev/null -w '%{http_code}' "${@:3}"; }

echo "== ONVIF operations"
[ "$(code /onvif/device_service "$(soap '<GetCapabilities/>')")" = 200 ] \
    && pass "GetCapabilities without credentials" || fail "GetCapabilities without credentials"
[ "$(code /onvif/media_service "$(soap '<GetProfiles/>')")" = 401 ] \
    && pass "GetProfiles without credentials is refused" || fail "GetProfiles without credentials should be 401"
[ "$(code /onvif/media_service "$(soap '<GetProfiles/>')" --digest -u "$USER_NAME:$PASSWORD")" = 200 ] \
    && pass "GetProfiles with HTTP Digest" || fail "GetProfiles with HTTP Digest"
[ "$(code /onvif/media_service "$(soap '<GetProfiles/>')" --digest -u "$USER_NAME:wrong")" = 401 ] \
    && pass "wrong password is refused" || fail "wrong password should be 401"

STREAM_URI=$(post /onvif/media_service "$(soap '<GetStreamUri><ProfileToken>Profile_1</ProfileToken></GetStreamUri>')" -u "$USER_NAME:$PASSWORD" | grep -o '<tt:Uri>[^<]*' | sed 's/<tt:Uri>//')
case "$STREAM_URI" in
    rtsp://*:8554/stream) pass "GetStreamUri returns $STREAM_URI" ;;
    *) fail "GetStreamUri returned '$STREAM_URI'" ;;
esac

DEVINFO=$(post /onvif/device_service "$(soap '<GetDeviceInformation/>')" -u "$USER_NAME:$PASSWORD")
echo "$DEVINFO" | grep -q "<tds:Model>E2E Camera</tds:Model>" \
    && pass "GetDeviceInformation reports the device name" || fail "GetDeviceInformation model mismatch"

WSSE=$(cat <<EOF
<?xml version="1.0"?><s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><s:Header><wsse:Security><wsse:UsernameToken><wsse:Username>${USER_NAME}</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordText">${PASSWORD/&/&amp;}</wsse:Password></wsse:UsernameToken></wsse:Security></s:Header><s:Body><GetVideoEncoderConfigurations/></s:Body></s:Envelope>
EOF
)
[ "$(code /onvif/media_service "$WSSE")" = 200 ] \
    && pass "WS-Security UsernameToken with wsse prefixes" || fail "WS-Security UsernameToken"

echo "== stream and snapshot"
sleep 5   # give the background probe a moment to fill in the real profile
PROFILE=$(post /onvif/media_service "$(soap '<GetProfiles/>')" -u "$USER_NAME:$PASSWORD")
echo "$PROFILE" | grep -q "<tt:Width>640</tt:Width>" \
    && pass "profile reports the probed resolution 640x360" || fail "profile did not report probed resolution: $(echo "$PROFILE" | grep -o '<tt:Width>[^<]*' | head -1)"

if docker exec "$NAME" sh -c 'ffprobe -v error -rtsp_transport tcp -show_entries stream=codec_name -of csv=p=0 rtsp://127.0.0.1:8554/stream' > /dev/null 2>&1; then
    fail "RTSP output is readable without credentials"
else
    pass "RTSP output refuses anonymous clients"
fi
ENC_PASS=$(printf '%s' "$PASSWORD" | sed 's/@/%40/g; s/&/%26/g; s/:/%3A/g; s/\//%2F/g')
CODEC=$(docker exec "$NAME" sh -c "ffprobe -v error -rtsp_transport tcp -show_entries stream=codec_name -of csv=p=0 'rtsp://${USER_NAME}:${ENC_PASS}@127.0.0.1:8554/stream'" 2>/dev/null | head -1)
[ "$CODEC" = "h264" ] && pass "RTSP output plays with the ONVIF credentials (h264)" || fail "RTSP output with credentials returned '$CODEC'"

SNAP=$(mktemp)
SNAP_CODE=$(curl -sS -m 30 --digest -u "$USER_NAME:$PASSWORD" -o "$SNAP" -w '%{http_code}' "${ONVIF}/snapshot.jpg")
if [ "$SNAP_CODE" = 200 ] && [ "$(head -c 2 "$SNAP" | od -An -tx1 | tr -d ' \n')" = "ffd8" ]; then
    pass "snapshot.jpg returns a JPEG ($(wc -c < "$SNAP") bytes)"
else
    fail "snapshot.jpg returned HTTP $SNAP_CODE"
fi
rm -f "$SNAP"
[ "$(curl -sS -m 10 -o /dev/null -w '%{http_code}' "${ONVIF}/snapshot.jpg")" = 401 ] \
    && pass "snapshot.jpg requires credentials" || fail "snapshot.jpg should require credentials"

echo "== WS-Discovery"
if command -v python3 > /dev/null 2>&1; then
    if python3 - "$WSD_PORT_HOST" <<'EOF'
import socket, sys, re
port = int(sys.argv[1])
probe = ('<?xml version="1.0"?><e:Envelope xmlns:e="http://www.w3.org/2003/05/soap-envelope" '
         'xmlns:w="http://schemas.xmlsoap.org/ws/2005/08/addressing" xmlns:d="http://schemas.xmlsoap.org/ws/2005/04/discovery" '
         'xmlns:dn="http://www.onvif.org/ver10/network/wsdl"><e:Header><w:MessageID>uuid:e2e-probe-1</w:MessageID>'
         '<w:To>urn:schemas-xmlsoap-org:ws:2005:04:discovery</w:To>'
         '<w:Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</w:Action></e:Header>'
         '<e:Body><d:Probe><d:Types>dn:NetworkVideoTransmitter</d:Types></d:Probe></e:Body></e:Envelope>')
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.settimeout(4)
s.sendto(probe.encode(), ("127.0.0.1", port))
try:
    data, _ = s.recvfrom(65535)
except socket.timeout:
    print("no ProbeMatch received"); sys.exit(1)
text = data.decode(errors="replace")
ok = "ProbeMatches" in text and re.search(r"<wsa:RelatesTo>uuid:e2e-probe-1</wsa:RelatesTo>", text)
print("ProbeMatch ok" if ok else text[:300]); sys.exit(0 if ok else 1)
EOF
    then pass "unicast Probe is answered with a matching RelatesTo"; else fail "WS-Discovery probe"; fi
else
    echo "SKIP  python3 not available for the discovery probe"
fi

echo "== resource usage and shutdown"
CPU=$(docker stats --no-stream --format '{{.CPUPerc}}' "$NAME" | tr -d '%')
echo "      container CPU: ${CPU}%"
awk -v c="$CPU" 'BEGIN { exit !(c < 50) }' && pass "idle CPU below 50%" || fail "idle CPU is ${CPU}%"

START=$(date +%s.%N)
docker stop "$NAME" > /dev/null
END=$(date +%s.%N)
STOP_SECONDS=$(awk -v s="$START" -v e="$END" 'BEGIN { printf "%.1f", e - s }')
EXIT_CODE=$(docker inspect -f '{{.State.ExitCode}}' "$NAME")
if [ "$EXIT_CODE" = 0 ] && awk -v t="$STOP_SECONDS" 'BEGIN { exit !(t < 8) }'; then
    pass "graceful stop in ${STOP_SECONDS}s with exit code 0"
else
    fail "stop took ${STOP_SECONDS}s, exit code ${EXIT_CODE}"
fi
docker logs "$NAME" 2>&1 | grep -q "sent Bye" && pass "WS-Discovery Bye sent on shutdown" || fail "no Bye on shutdown"

if [ "$FAILED" -ne 0 ]; then
    echo "== container logs (tail)"
    docker logs "$NAME" 2>&1 | tail -60
    echo "RESULT: FAILED"
    exit 1
fi
echo "RESULT: ALL PASSED"
