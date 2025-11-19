#!/bin/sh

# Set stack size limits to help with musl runtime
ulimit -s 16384 2>/dev/null || echo "Warning: Could not set stack size limit"

echo "========================================"
echo "ONVIF Media Transcoder Starting"
echo "========================================"

export RUST_BACKTRACE=1
# Validation function - all environment variables should be set in Dockerfile
validate_env() {
    local errors=0
    
    if [ -z "$INPUT_URL" ]; then
        echo "ERROR: INPUT_URL environment variable is not set"
        echo "       Example: https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8"
        errors=$((errors + 1))
    fi
    
    if [ -z "$RTSP_OUTPUT_PORT" ]; then
        echo "ERROR: RTSP_OUTPUT_PORT environment variable is not set"
        errors=$((errors + 1))
    elif ! echo "$RTSP_OUTPUT_PORT" | grep -qE '^[0-9]+$' || [ "$RTSP_OUTPUT_PORT" -lt 1 ] || [ "$RTSP_OUTPUT_PORT" -gt 65535 ]; then
        echo "ERROR: RTSP_OUTPUT_PORT must be a valid port number (1-65535), got: $RTSP_OUTPUT_PORT"
        errors=$((errors + 1))
    fi
    
    if [ -z "$RTSP_PATH" ]; then
        echo "ERROR: RTSP_PATH environment variable is not set"
        errors=$((errors + 1))
    else
        # Automatically add leading slash if missing
        case "$RTSP_PATH" in
            /*) 
                # Already has leading slash, keep as is
                ;;
            *)
                # Missing leading slash, add it
                echo "INFO: Adding leading slash to RTSP_PATH: '$RTSP_PATH' -> '/$RTSP_PATH'"
                RTSP_PATH="/$RTSP_PATH"
                export RTSP_PATH
                ;;
        esac
    fi
    
    if [ -z "$ONVIF_PORT" ]; then
        echo "ERROR: ONVIF_PORT environment variable is not set"
        errors=$((errors + 1))
    elif ! echo "$ONVIF_PORT" | grep -qE '^[0-9]+$' || [ "$ONVIF_PORT" -lt 1 ] || [ "$ONVIF_PORT" -gt 65535 ]; then
        echo "ERROR: ONVIF_PORT must be a valid port number (1-65535), got: $ONVIF_PORT"
        errors=$((errors + 1))
    fi
    
    if [ -z "$DEVICE_NAME" ]; then
        echo "ERROR: DEVICE_NAME environment variable is not set"
        errors=$((errors + 1))
    elif [ ${#DEVICE_NAME} -lt 3 ]; then
        echo "ERROR: DEVICE_NAME must be at least 3 characters long, got: $DEVICE_NAME"
        errors=$((errors + 1))
    fi
    
    if [ -z "$ONVIF_USERNAME" ]; then
        echo "ERROR: ONVIF_USERNAME environment variable is not set"
        errors=$((errors + 1))
    elif [ ${#ONVIF_USERNAME} -lt 3 ]; then
        echo "ERROR: ONVIF_USERNAME must be at least 3 characters long, got: $ONVIF_USERNAME"
        errors=$((errors + 1))
    fi
    
    if [ -z "$ONVIF_PASSWORD" ]; then
        echo "ERROR: ONVIF_PASSWORD environment variable is not set"
        errors=$((errors + 1))
    elif [ ${#ONVIF_PASSWORD} -lt 6 ]; then
        echo "ERROR: ONVIF_PASSWORD must be at least 6 characters long, got: $ONVIF_PASSWORD"
        errors=$((errors + 1))
    fi
    
    if [ -z "$WS_DISCOVERY_ENABLED" ]; then
        echo "ERROR: WS_DISCOVERY_ENABLED environment variable is not set"
        errors=$((errors + 1))
    else
        # Normalize and validate WS_DISCOVERY_ENABLED value
        WS_DISCOVERY_ENABLED=$(echo "$WS_DISCOVERY_ENABLED" | tr '[:upper:]' '[:lower:]')
        case "$WS_DISCOVERY_ENABLED" in
            "true"|"1"|"yes"|"on"|"enabled")
                WS_DISCOVERY_ENABLED="true"
                export WS_DISCOVERY_ENABLED
                echo "INFO: WS-Discovery is ENABLED"
                ;;
            "false"|"0"|"no"|"off"|"disabled")
                WS_DISCOVERY_ENABLED="false"
                export WS_DISCOVERY_ENABLED
                echo "INFO: WS-Discovery is DISABLED"
                ;;
            *)
                echo "ERROR: WS_DISCOVERY_ENABLED must be 'true' or 'false' (case insensitive), got: $WS_DISCOVERY_ENABLED"
                echo "       Accepted values: true/false, 1/0, yes/no, on/off, enabled/disabled"
                errors=$((errors + 1))
                ;;
        esac
    fi
    
    # Validate and normalize DEBUGLOGGING
    if [ -z "$DEBUGLOGGING" ]; then
        DEBUGLOGGING="false"  # Default to false when not set
        export DEBUGLOGGING
        echo "INFO: Debug logging is DISABLED (default)"
    else
        # Normalize and validate DEBUGLOGGING value
        DEBUGLOGGING=$(echo "$DEBUGLOGGING" | tr '[:upper:]' '[:lower:]')
        case "$DEBUGLOGGING" in
            "true"|"1"|"yes"|"on"|"enabled")
                DEBUGLOGGING="true"
                export DEBUGLOGGING
                echo "INFO: Debug logging is ENABLED"
                ;;
            "false"|"0"|"no"|"off"|"disabled")
                DEBUGLOGGING="false"
                export DEBUGLOGGING
                echo "INFO: Debug logging is DISABLED"
                ;;
            *)
                echo "ERROR: DEBUGLOGGING must be 'true' or 'false' (case insensitive), got: $DEBUGLOGGING"
                echo "       Accepted values: true/false, 1/0, yes/no, on/off, enabled/disabled"
                errors=$((errors + 1))
                ;;
        esac
    fi
    
    # Validate INPUT_URL format (MediaMTX supports many protocols)
    if [ -n "$INPUT_URL" ]; then
        # Check if it's a valid URL/path format that MediaMTX can handle
        # MediaMTX supports: http(s)://, rtsp://, rtmp://, udp://, tcp://, file paths, etc.
        if echo "$INPUT_URL" | grep -qE '^[a-zA-Z]+://' || [ -f "$INPUT_URL" ] || echo "$INPUT_URL" | grep -qE '^/'; then
            echo "INFO: INPUT_URL format accepted: $INPUT_URL"
        else
            echo "WARNING: INPUT_URL format may not be supported by MediaMTX: $INPUT_URL"
            echo "         MediaMTX supports: http(s)://, rtsp://, rtmp://, udp://, tcp://, file paths, etc."
        fi
    fi
    
    if [ $errors -gt 0 ]; then
        echo "========================================" 
        echo "CONFIGURATION ERRORS FOUND: $errors error(s)"
        echo "Please set all required environment variables correctly."
        echo "========================================"
        exit 1
    fi
}

# Validate environment variables
echo "Validating environment variables..."
validate_env

# Validate INPUT_URL reachability
if [ -n "$INPUT_URL" ]; then
    echo "INFO: Validating input stream: $INPUT_URL"
    echo "INFO: This may take a few seconds..."
    
    # Use ffprobe to check if the stream is readable
    # -v error: Only show errors
    # -show_format: Try to read container format
    # -rw_timeout: Timeout in microseconds (10 seconds) for read/write operations (for network streams)
    # timeout command: Hard timeout for the whole process (15 seconds)
    if ! timeout 15s ffprobe -v error -show_format -rw_timeout 10000000 -i "$INPUT_URL" > /dev/null 2>&1; then
         echo "ERROR: Unable to connect to INPUT_URL: $INPUT_URL"
         echo "       Please check if the stream is online and accessible."
         echo "       Validation failed. Exiting."
         exit 1
    fi
    echo "INFO: Input stream is valid and reachable."
fi

# Configuration summary
# MediaMTX will handle the input stream and serve it as an ONVIF-compatible RTSP stream

# Get the container's actual IP address (not 0.0.0.0)
# Allow user to override with CONTAINER_IP environment variable
if [ -z "$CONTAINER_IP" ]; then
    CONTAINER_IP=$(hostname -i | awk '{print $1}' 2>/dev/null)
    
    # Fallback if hostname -i fails or returns loopback
    if [ -z "$CONTAINER_IP" ] || [ "$CONTAINER_IP" = "127.0.0.1" ]; then
        CONTAINER_IP=$(ip route get 1 2>/dev/null | awk '{print $7; exit}')
    fi
    
    if [ -z "$CONTAINER_IP" ]; then
        echo "WARNING: Could not determine container IP, defaulting to 127.0.0.1"
        CONTAINER_IP="127.0.0.1"
    fi
fi

echo "INFO: Container IP detected as: $CONTAINER_IP"

# Handle root path case explicitly to ensure consistency
if [ "$RTSP_PATH" = "/" ]; then
    echo "INFO: RTSP_PATH is '/', defaulting to '/stream' for internal consistency"
    RTSP_PATH="/stream"
    export RTSP_PATH
fi

RTSP_OUTPUT_URL="rtsp://${CONTAINER_IP}:${RTSP_OUTPUT_PORT}${RTSP_PATH}"

echo "Configuration validated successfully:"
echo "  Input URL: ${INPUT_URL}"
echo "  Container IP: ${CONTAINER_IP}"
echo "  RTSP Output: ${RTSP_OUTPUT_URL}"
echo "  ONVIF Port: ${ONVIF_PORT}"
echo "  Device Name: ${DEVICE_NAME}"
echo "  ONVIF Username: ${ONVIF_USERNAME}"
echo "  ONVIF Password: [HIDDEN]"
echo "  WS-Discovery: ${WS_DISCOVERY_ENABLED}"

# Calculate the RTSP stream URL for the Rust application
export RTSP_STREAM_URL="rtsp://${CONTAINER_IP}:${RTSP_OUTPUT_PORT}${RTSP_PATH}"

# Create dynamic MediaMTX configuration with correct RTSP path and port
STREAM_NAME="${RTSP_PATH#/}"  # Remove leading slash
if [ -z "$STREAM_NAME" ]; then
    echo "INFO: RTSP_PATH resulted in empty stream name, using default 'stream'"
    STREAM_NAME="stream"
fi

# Validate stream name doesn't contain invalid characters
if echo "$STREAM_NAME" | grep -q '[^a-zA-Z0-9/_-]'; then
    echo "WARNING: RTSP_PATH contains special characters that may cause issues: $STREAM_NAME"
    echo "         Recommended format: /stream or /camera1 etc."
fi

echo "INFO: MediaMTX stream will be available as: $STREAM_NAME"

# Update MediaMTX config with the correct stream path and port
# Generate unique RTP/RTCP ports based on RTSP port to avoid conflicts between containers
RTP_PORT=$((RTSP_OUTPUT_PORT + 1000))  # e.g., 8554 -> 9554
RTCP_PORT=$((RTSP_OUTPUT_PORT + 1001)) # e.g., 8554 -> 9555
RTMP_PORT=$((RTSP_OUTPUT_PORT + 2000)) # e.g., 8554 -> 10554

echo "INFO: MediaMTX ports will be - RTSP: ${RTSP_OUTPUT_PORT}, RTP: ${RTP_PORT}, RTCP: ${RTCP_PORT}, RTMP: ${RTMP_PORT}"

# Use a safer delimiter (~) for sed to avoid issues with URLs containing / or |
# We assume ~ is rare in URLs, but we should be careful.
# Alternatively, we can escape the input.
sed -e "s~STREAM_PATH_PLACEHOLDER~${STREAM_NAME}~g" \
    -e "s~RTSP_PORT_PLACEHOLDER~${RTSP_OUTPUT_PORT}~g" \
    -e "s~RTP_PORT_PLACEHOLDER~${RTP_PORT}~g" \
    -e "s~RTCP_PORT_PLACEHOLDER~${RTCP_PORT}~g" \
    -e "s~RTMP_PORT_PLACEHOLDER~${RTMP_PORT}~g" \
    -e "s~SOURCE_PLACEHOLDER~${INPUT_URL}~g" \
    /etc/mediamtx.yml > /tmp/mediamtx.yml

echo "INFO: MediaMTX configuration generated successfully"
echo "INFO: MediaMTX Paths Configuration:"
grep -A 5 "paths:" /tmp/mediamtx.yml

# Start MediaMTX RTSP server
# Log to stdout/stderr directly so we can see issues
echo "Starting MediaMTX RTSP server..."
mediamtx /tmp/mediamtx.yml &
MEDIAMTX_PID=$!

if ! kill -0 $MEDIAMTX_PID 2>/dev/null; then
    echo "ERROR: Failed to start MediaMTX RTSP server"
    exit 1
fi
echo "MediaMTX started with PID: $MEDIAMTX_PID"

# Wait for MediaMTX to start and begin listening
echo "Waiting for MediaMTX to initialize..."
mediamtx_ready=false
for i in $(seq 1 10); do
    sleep 2
    if netstat -ln 2>/dev/null | grep -q ":${RTSP_OUTPUT_PORT} " || ss -ln 2>/dev/null | grep -q ":${RTSP_OUTPUT_PORT} "; then
        echo "MediaMTX is listening on port ${RTSP_OUTPUT_PORT}"
        mediamtx_ready=true
        break
    fi
    echo "Waiting for MediaMTX to start listening... (attempt $i/20)"
done

if [ "$mediamtx_ready" = "false" ]; then
    echo "WARNING: MediaMTX may not be ready, but continuing..."
fi

# Check if MediaMTX is still running before proceeding
if ! kill -0 $MEDIAMTX_PID 2>/dev/null; then
    echo "ERROR: MediaMTX process died during startup (PID: $MEDIAMTX_PID)"
    echo "MediaMTX exit status: $(wait $MEDIAMTX_PID 2>/dev/null; echo $?)"
    exit 1
fi

# Create directories
mkdir -p /tmp

# MediaMTX is now ready to serve streams
echo "MediaMTX RTSP server is ready to serve streams"
echo "Stream will be available at: ${RTSP_OUTPUT_URL}"

# Verify the Rust binary exists
if [ ! -f "/usr/local/bin/onvif-media-transcoder" ]; then
    echo "ERROR: Rust binary not found at /usr/local/bin/onvif-media-transcoder"
    kill $MEDIAMTX_PID 2>/dev/null
    exit 1
fi

# Prepare flags for the ONVIF service
WS_DISCOVERY_FLAG=$([ "$WS_DISCOVERY_ENABLED" = "true" ] && echo "--ws-discovery-enabled")
DEBUG_FLAG=$([ "$DEBUGLOGGING" = "true" ] && echo "--debug")

# Start the ONVIF Media Transcoder
echo "Starting ONVIF Media Transcoder..."
echo "Command: /usr/local/bin/onvif-media-transcoder -r \"$RTSP_STREAM_URL\" -P \"$ONVIF_PORT\" -n \"$DEVICE_NAME\" -u \"$ONVIF_USERNAME\" -p \"$ONVIF_PASSWORD\" --container-ip \"$CONTAINER_IP\" $WS_DISCOVERY_FLAG $DEBUG_FLAG"

# Start ONVIF service in background so we can monitor it
/usr/local/bin/onvif-media-transcoder \
    -r "$RTSP_STREAM_URL" \
    -P "$ONVIF_PORT" \
    -n "$DEVICE_NAME" \
    -u "$ONVIF_USERNAME" \
    -p "$ONVIF_PASSWORD" \
    --container-ip "$CONTAINER_IP" \
    $WS_DISCOVERY_FLAG \
    $DEBUG_FLAG &
ONVIF_SERVICE_PID=$!

echo "ONVIF service started with PID: $ONVIF_SERVICE_PID"

echo "========================================"
echo "ONVIF Media Transcoder is ready!"
echo "Container IP: ${CONTAINER_IP}"
if [ "$WS_DISCOVERY_ENABLED" = "true" ]; then
    echo "Device discovery: Enabled via native Rust WS-Discovery on UDP port 3702"
else
    echo "Device discovery: Disabled (WS-Discovery is turned off)"
fi
echo "RTSP Stream: ${RTSP_OUTPUT_URL}"
echo "ONVIF Endpoint: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"
echo "Note: Input stream is served via MediaMTX RTSP server"
echo "========================================"

# Function to handle shutdown
cleanup_services() {
    echo "Shutting down services..."
    kill $ONVIF_SERVICE_PID 2>/dev/null
    kill $MEDIAMTX_PID 2>/dev/null
    wait
    echo "All services stopped."
}

trap cleanup_services SIGTERM SIGINT

# Monitor all processes
monitor_all_services() {
    while true; do
        sleep 10
        
        if ! kill -0 $MEDIAMTX_PID 2>/dev/null; then
            echo "ERROR: MediaMTX process died (PID: $MEDIAMTX_PID)"
            cleanup_services
            exit 1
        fi
        
        if ! kill -0 $ONVIF_SERVICE_PID 2>/dev/null; then
            echo "ERROR: ONVIF service died (PID: $ONVIF_SERVICE_PID)"
            echo "ONVIF service output should be visible in the main log output above"
            cleanup_services
            exit 1
        fi
    done
}

# Start monitoring in background
monitor_all_services &
MONITOR_PID=$!

# Wait for all background processes
wait