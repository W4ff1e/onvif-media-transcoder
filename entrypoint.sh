#!/bin/sh

echo "========================================"
echo "ONVIF Media Transcoder Starting"
echo "========================================"

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
    
    # Validate INPUT_URL format (FFmpeg supports many protocols)
    if [ -n "$INPUT_URL" ]; then
        # Check if it's a valid URL/path format that FFmpeg can handle
        # FFmpeg supports: http(s)://, rtsp://, rtmp://, udp://, tcp://, file paths, etc.
        if echo "$INPUT_URL" | grep -qE '^[a-zA-Z]+://' || [ -f "$INPUT_URL" ] || echo "$INPUT_URL" | grep -qE '^/'; then
            echo "INFO: INPUT_URL format accepted: $INPUT_URL"
        else
            echo "WARNING: INPUT_URL format may not be supported by FFmpeg: $INPUT_URL"
            echo "         FFmpeg supports: http(s)://, rtsp://, rtmp://, udp://, tcp://, file paths, etc."
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

# Configuration summary
# We'll re-encode the input stream to a proper ONVIF-compatible RTSP stream

# Get the container's actual IP address (not 0.0.0.0)
CONTAINER_IP=$(hostname -i | awk '{print $1}' 2>/dev/null)
if [ -z "$CONTAINER_IP" ] || [ "$CONTAINER_IP" = "0.0.0.0" ] || [ "$CONTAINER_IP" = "127.0.0.1" ]; then
    # Fallback methods to get container IP
    echo "INFO: Primary IP detection failed, trying fallback methods..."
    
    # Try ip route method
    CONTAINER_IP=$(ip route get 1 2>/dev/null | awk '{print $7; exit}')
    if [ -z "$CONTAINER_IP" ] || [ "$CONTAINER_IP" = "0.0.0.0" ]; then
        # Try hostname -I method
        CONTAINER_IP=$(hostname -I 2>/dev/null | awk '{print $1}')
    fi
    
    # Try parsing /proc/net/route
    if [ -z "$CONTAINER_IP" ] || [ "$CONTAINER_IP" = "0.0.0.0" ]; then
        CONTAINER_IP=$(awk '/^[0-9A-F]{8}\s+00000000/ {print $1}' /proc/net/route 2>/dev/null | head -n1 | sed 's/\(..\)\(..\)\(..\)\(..\)/printf "%d.%d.%d.%d" 0x\4 0x\3 0x\2 0x\1/' | sh 2>/dev/null)
    fi
    
    # Final fallback to Docker bridge network detection
    if [ -z "$CONTAINER_IP" ] || [ "$CONTAINER_IP" = "0.0.0.0" ]; then
        CONTAINER_IP=$(ip addr show eth0 2>/dev/null | grep 'inet ' | awk '{print $2}' | cut -d'/' -f1)
    fi
    
    # Ultimate fallback
    if [ -z "$CONTAINER_IP" ] || [ "$CONTAINER_IP" = "0.0.0.0" ]; then
        echo "WARNING: Could not determine container IP, using localhost"
        CONTAINER_IP="127.0.0.1"
    fi
fi

echo "INFO: Container IP detected as: $CONTAINER_IP"

RTSP_OUTPUT_URL="rtsp://${CONTAINER_IP}:${RTSP_OUTPUT_PORT}${RTSP_PATH}"

echo "Configuration validated successfully:"
echo "  Input URL: ${INPUT_URL}"
echo "  Container IP: ${CONTAINER_IP}"
echo "  RTSP Output: ${RTSP_OUTPUT_URL}"
echo "  ONVIF Port: ${ONVIF_PORT}"
echo "  Device Name: ${DEVICE_NAME}"
echo "  ONVIF Username: ${ONVIF_USERNAME}"
echo "  ONVIF Password: [HIDDEN]"

# Export environment variables for Rust application
export RTSP_STREAM_URL="rtsp://${CONTAINER_IP}:${RTSP_OUTPUT_PORT}${RTSP_PATH}"
export ONVIF_PORT="${ONVIF_PORT}"
export DEVICE_NAME="${DEVICE_NAME}"
export ONVIF_USERNAME="${ONVIF_USERNAME}"
export ONVIF_PASSWORD="${ONVIF_PASSWORD}"

# Function to manage FFmpeg logs with size capping
manage_ffmpeg_logs() {
    local log_file="/tmp/ffmpeg.log"
    local max_lines=1000
    local keep_head=500
    local keep_tail=500
    
    # Monitor log file size and truncate if needed
    while true; do
        sleep 30
        if [ -f "$log_file" ]; then
            local line_count=$(wc -l < "$log_file")
            if [ $line_count -gt $max_lines ]; then
                echo "$(date): FFmpeg log reached $line_count lines, truncating..." >> "$log_file"
                
                # Keep first 500 and last 500 lines
                head -n $keep_head "$log_file" > "${log_file}.tmp"
                echo "" >> "${log_file}.tmp"
                echo "=== LOG TRUNCATED $(date) ===" >> "${log_file}.tmp"
                echo "" >> "${log_file}.tmp"
                tail -n $keep_tail "$log_file" >> "${log_file}.tmp"
                mv "${log_file}.tmp" "$log_file"
                
                echo "$(date): FFmpeg log truncated, keeping first $keep_head and last $keep_tail lines" >> "$log_file"
            fi
        fi
    done
}

# Function to manage MediaMTX logs with size capping
manage_mediamtx_logs() {
    local log_file="/tmp/mediamtx.log"
    local max_lines=1000
    local keep_head=500
    local keep_tail=500
    
    # Monitor log file size and truncate if needed
    while true; do
        sleep 30
        if [ -f "$log_file" ]; then
            local line_count=$(wc -l < "$log_file")
            if [ $line_count -gt $max_lines ]; then
                echo "$(date): MediaMTX log reached $line_count lines, truncating..." >> "$log_file"
                
                # Keep first 500 and last 500 lines
                head -n $keep_head "$log_file" > "${log_file}.tmp"
                echo "" >> "${log_file}.tmp"
                echo "=== LOG TRUNCATED $(date) ===" >> "${log_file}.tmp"
                echo "" >> "${log_file}.tmp"
                tail -n $keep_tail "$log_file" >> "${log_file}.tmp"
                mv "${log_file}.tmp" "$log_file"
                
                echo "$(date): MediaMTX log truncated, keeping first $keep_head and last $keep_tail lines" >> "$log_file"
            fi
        fi
    done
}

# Function to dump FFmpeg logs on error
dump_ffmpeg_logs() {
    local log_file="/tmp/ffmpeg.log"
    if [ -f "$log_file" ]; then
        echo "========================================" 
        echo "FFmpeg Log (Last 100 lines):"
        echo "========================================"
        tail -n 100 "$log_file"
        echo "========================================"
    else
        echo "No FFmpeg log file found"
    fi
}

# Function to dump MediaMTX logs on error
dump_mediamtx_logs() {
    local log_file="/tmp/mediamtx.log"
    if [ -f "$log_file" ]; then
        echo "========================================" 
        echo "MediaMTX Log (Last 50 lines):"
        echo "========================================"
        tail -n 50 "$log_file"
        echo "========================================"
    else
        echo "No MediaMTX log file found"
    fi
}

# Create dynamic MediaMTX configuration with correct RTSP path and port
STREAM_NAME="${RTSP_PATH#/}"  # Remove leading slash
if [ -z "$STREAM_NAME" ] || [ "$STREAM_NAME" = "/" ]; then
    echo "INFO: RTSP_PATH resulted in empty stream name, using default 'stream'"
    STREAM_NAME="stream"  # Default if path is just "/" or empty
fi

# Validate stream name doesn't contain invalid characters
if echo "$STREAM_NAME" | grep -q '[^a-zA-Z0-9_-]'; then
    echo "WARNING: RTSP_PATH contains special characters that may cause issues: $STREAM_NAME"
    echo "         Recommended format: /stream or /camera1 etc."
fi

echo "INFO: MediaMTX stream will be available as: $STREAM_NAME"

# Update MediaMTX config with the correct stream path and port
sed -e "s/STREAM_PATH_PLACEHOLDER/${STREAM_NAME}/g" \
    -e "s/RTSP_PORT_PLACEHOLDER/${RTSP_OUTPUT_PORT}/g" \
    /etc/mediamtx.yml > /tmp/mediamtx.yml

# Start MediaMTX RTSP server
echo "Starting MediaMTX RTSP server..."
mediamtx /tmp/mediamtx.yml > /tmp/mediamtx.log 2>&1 &
MEDIAMTX_PID=$!

if ! kill -0 $MEDIAMTX_PID 2>/dev/null; then
    echo "ERROR: Failed to start MediaMTX RTSP server"
    echo "MediaMTX log output:"
    cat /tmp/mediamtx.log
    exit 1
fi
echo "MediaMTX started with PID: $MEDIAMTX_PID"

# Wait for MediaMTX to start and begin listening
echo "Waiting for MediaMTX to initialize..."
mediamtx_ready=false
for i in $(seq 1 20); do
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
    dump_mediamtx_logs
fi

# Start MediaMTX log management in background
manage_mediamtx_logs &
MEDIAMTX_LOG_MANAGER_PID=$!

# Start FFmpeg to re-encode input stream and push to MediaMTX RTSP server
echo "Starting FFmpeg stream re-encoding..."

# Create directories
mkdir -p /tmp

# Start log management in background for both FFmpeg and MediaMTX
manage_ffmpeg_logs &
FFMPEG_LOG_MANAGER_PID=$!

# Function to start FFmpeg with retry logic
start_ffmpeg_with_retry() {
    local max_retries=5
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        echo "FFmpeg attempt $((retry_count + 1)) of $max_retries..."
        
        # Start FFmpeg with ONVIF live streaming optimized settings
        # Optimized for real-time viewing compatibility (fixes green frame in live view)
        ffmpeg \
            -re \
            -rw_timeout 10000000 \
            -i "${INPUT_URL}" \
            -c:v libx264 \
            -preset ultrafast \
            -tune zerolatency \
            -profile:v baseline \
            -level 3.1 \
            -pix_fmt yuv420p \
            -vf "scale=960:540:flags=bilinear:force_original_aspect_ratio=disable,fps=15" \
            -g 15 \
            -keyint_min 15 \
            -sc_threshold 0 \
            -b:v 1500k \
            -maxrate 1800k \
            -bufsize 2M \
            -refs 1 \
            -x264opts "nal-hrd=cbr:force-cfr=1:intra-refresh=1:keyint=15:min-keyint=15" \
            -an \
            -f rtsp \
            -rtsp_transport tcp \
            -timeout 10000000 \
            "rtsp://localhost:${RTSP_OUTPUT_PORT}${RTSP_PATH}" \
            > /tmp/ffmpeg.log 2>&1 &
        
        FFMPEG_ENCODER_PID=$!
        
        # Give FFmpeg more time to start
        sleep 5
        
        if kill -0 $FFMPEG_ENCODER_PID 2>/dev/null; then
            echo "FFmpeg stream encoder started successfully with PID: $FFMPEG_ENCODER_PID"
            return 0
        else
            echo "FFmpeg startup attempt $((retry_count + 1)) failed"
            retry_count=$((retry_count + 1))
            if [ $retry_count -lt $max_retries ]; then
                echo "Retrying in 5 seconds..."
                sleep 5
            fi
        fi
    done
    
    echo "ERROR: Failed to start FFmpeg after $max_retries attempts"
    return 1
}

# Start FFmpeg with retry logic
if ! start_ffmpeg_with_retry; then
    dump_ffmpeg_logs
    dump_mediamtx_logs
    kill $MEDIAMTX_PID $MEDIAMTX_LOG_MANAGER_PID 2>/dev/null
    exit 1
fi

# Wait for FFmpeg to connect to MediaMTX and start streaming
echo "Waiting for FFmpeg to connect to MediaMTX..."
sleep 15

# Verify FFmpeg is still running
if ! kill -0 $FFMPEG_ENCODER_PID 2>/dev/null; then
    echo "ERROR: FFmpeg process died during startup"
    dump_ffmpeg_logs
    dump_mediamtx_logs
    kill $MEDIAMTX_PID $MEDIAMTX_LOG_MANAGER_PID 2>/dev/null
    exit 1
fi

# Test RTSP stream from MediaMTX
echo "Testing RTSP stream from MediaMTX..."
timeout 5 ffprobe -v quiet -select_streams v:0 -show_entries stream=codec_name -of csv=p=0 "${RTSP_OUTPUT_URL}" >/dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "RTSP stream test: SUCCESS"
else
    echo "WARNING: RTSP stream test failed, but continuing (stream may need more time to initialize)"
    echo "FFmpeg may still be connecting to MediaMTX..."
    echo "Check logs for details:"
    echo "  - FFmpeg: /tmp/ffmpeg.log"
    echo "  - MediaMTX: /tmp/mediamtx.log"
fi

# Start ONVIF web service (Rust application)
echo "Starting ONVIF device discovery..."
echo "Container IP: ${CONTAINER_IP}"
echo "INFO: WS-Discovery implemented in Rust (integrated with ONVIF service)"
echo "      ONVIF service will be available at: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"

# Verify the Rust binary exists
if [ ! -f "/usr/local/bin/onvif-media-transcoder" ]; then
    echo "ERROR: Rust binary not found at /usr/local/bin/onvif-media-transcoder"
    dump_ffmpeg_logs
    dump_mediamtx_logs
    kill $FFMPEG_ENCODER_PID $MEDIAMTX_PID $FFMPEG_LOG_MANAGER_PID $MEDIAMTX_LOG_MANAGER_PID 2>/dev/null
    exit 1
fi

/usr/local/bin/onvif-media-transcoder &
ONVIF_SERVICE_PID=$!
if ! kill -0 $ONVIF_SERVICE_PID 2>/dev/null; then
    echo "ERROR: Failed to start ONVIF service"
    dump_ffmpeg_logs
    dump_mediamtx_logs
    kill $FFMPEG_ENCODER_PID $MEDIAMTX_PID $FFMPEG_LOG_MANAGER_PID $MEDIAMTX_LOG_MANAGER_PID 2>/dev/null
    exit 1
fi
echo "ONVIF service started with PID: $ONVIF_SERVICE_PID"

echo "========================================"
echo "ONVIF Media Transcoder is ready!"
echo "Container IP: ${CONTAINER_IP}"
echo "Device discovery: Enabled via native Rust WS-Discovery on UDP port 3702"
echo "RTSP Stream: ${RTSP_OUTPUT_URL}"
echo "ONVIF Endpoint: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"
echo "Note: Input stream is re-encoded and served via MediaMTX RTSP server"
echo "Log files:"
echo "  - FFmpeg: /tmp/ffmpeg.log (capped at 1000 lines)"
echo "  - MediaMTX: /tmp/mediamtx.log (capped at 1000 lines)"
echo "========================================"

# Function to handle shutdown
cleanup_services() {
    echo "Shutting down services..."
    kill $ONVIF_SERVICE_PID 2>/dev/null
    kill $FFMPEG_ENCODER_PID 2>/dev/null  
    kill $MEDIAMTX_PID 2>/dev/null
    kill $FFMPEG_LOG_MANAGER_PID 2>/dev/null
    kill $MEDIAMTX_LOG_MANAGER_PID 2>/dev/null
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
            dump_mediamtx_logs
            cleanup_services
            exit 1
        fi
        
        if ! kill -0 $FFMPEG_ENCODER_PID 2>/dev/null; then
            echo "ERROR: FFmpeg encoder process died (PID: $FFMPEG_ENCODER_PID)"
            dump_ffmpeg_logs
            cleanup_services
            exit 1
        fi
        
        if ! kill -0 $ONVIF_SERVICE_PID 2>/dev/null; then
            echo "ERROR: ONVIF service died (PID: $ONVIF_SERVICE_PID)"
            cleanup_services
            exit 1
        fi
        
        if ! kill -0 $FFMPEG_LOG_MANAGER_PID 2>/dev/null; then
            echo "WARNING: FFmpeg log manager died (PID: $FFMPEG_LOG_MANAGER_PID), restarting..."
            manage_ffmpeg_logs &
            FFMPEG_LOG_MANAGER_PID=$!
            echo "FFmpeg log manager restarted with PID: $FFMPEG_LOG_MANAGER_PID"
        fi
        
        if ! kill -0 $MEDIAMTX_LOG_MANAGER_PID 2>/dev/null; then
            echo "WARNING: MediaMTX log manager died (PID: $MEDIAMTX_LOG_MANAGER_PID), restarting..."
            manage_mediamtx_logs &
            MEDIAMTX_LOG_MANAGER_PID=$!
            echo "MediaMTX log manager restarted with PID: $MEDIAMTX_LOG_MANAGER_PID"
        fi
    done
}

# Start monitoring in background
monitor_all_services &
MONITOR_PID=$!

# Wait for all background processes
wait