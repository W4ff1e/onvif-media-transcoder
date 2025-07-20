#!/bin/sh

echo "========================================"
echo "FFmpeg ONVIF Camera Emulator Starting"
echo "========================================"

# Validation function - all environment variables should be set in Dockerfile
validate_env() {
    local errors=0
    
    if [[ -z "$INPUT_URL" ]]; then
        echo "ERROR: INPUT_URL environment variable is not set"
        echo "       Example: https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$OUTPUT_PORT" ]]; then
        echo "ERROR: OUTPUT_PORT environment variable is not set"
        errors=$((errors + 1))
    elif ! [[ "$OUTPUT_PORT" =~ ^[0-9]+$ ]] || [ "$OUTPUT_PORT" -lt 1 ] || [ "$OUTPUT_PORT" -gt 65535 ]; then
        echo "ERROR: OUTPUT_PORT must be a valid port number (1-65535), got: $OUTPUT_PORT"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$RTSP_OUTPUT_PORT" ]]; then
        echo "ERROR: RTSP_OUTPUT_PORT environment variable is not set"
        errors=$((errors + 1))
    elif ! [[ "$RTSP_OUTPUT_PORT" =~ ^[0-9]+$ ]] || [ "$RTSP_OUTPUT_PORT" -lt 1 ] || [ "$RTSP_OUTPUT_PORT" -gt 65535 ]; then
        echo "ERROR: RTSP_OUTPUT_PORT must be a valid port number (1-65535), got: $RTSP_OUTPUT_PORT"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$RTSP_PATH" ]]; then
        echo "ERROR: RTSP_PATH environment variable is not set"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$ONVIF_PORT" ]]; then
        echo "ERROR: ONVIF_PORT environment variable is not set"
        errors=$((errors + 1))
    elif ! [[ "$ONVIF_PORT" =~ ^[0-9]+$ ]] || [ "$ONVIF_PORT" -lt 1 ] || [ "$ONVIF_PORT" -gt 65535 ]; then
        echo "ERROR: ONVIF_PORT must be a valid port number (1-65535), got: $ONVIF_PORT"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$DEVICE_NAME" ]]; then
        echo "ERROR: DEVICE_NAME environment variable is not set"
        errors=$((errors + 1))
    elif [[ ${#DEVICE_NAME} -lt 3 ]]; then
        echo "ERROR: DEVICE_NAME must be at least 3 characters long, got: $DEVICE_NAME"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$ONVIF_USERNAME" ]]; then
        echo "ERROR: ONVIF_USERNAME environment variable is not set"
        errors=$((errors + 1))
    elif [[ ${#ONVIF_USERNAME} -lt 3 ]]; then
        echo "ERROR: ONVIF_USERNAME must be at least 3 characters long, got: $ONVIF_USERNAME"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$ONVIF_PASSWORD" ]]; then
        echo "ERROR: ONVIF_PASSWORD environment variable is not set"
        errors=$((errors + 1))
    elif [[ ${#ONVIF_PASSWORD} -lt 6 ]]; then
        echo "ERROR: ONVIF_PASSWORD must be at least 6 characters long, got: $ONVIF_PASSWORD"
        errors=$((errors + 1))
    fi
    
    # Validate INPUT_URL format (FFmpeg supports many protocols)
    if [[ -n "$INPUT_URL" ]]; then
        # Check if it's a valid URL/path format that FFmpeg can handle
        # FFmpeg supports: http(s)://, rtsp://, rtmp://, udp://, tcp://, file paths, etc.
        if [[ "$INPUT_URL" =~ ^[a-zA-Z]+:// ]] || [[ -f "$INPUT_URL" ]] || [[ "$INPUT_URL" =~ ^/ ]]; then
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
CONTAINER_IP=$(hostname -i | awk '{print $1}')
if [[ -z "$CONTAINER_IP" ]] || [[ "$CONTAINER_IP" == "0.0.0.0" ]]; then
    # Fallback methods to get container IP
    CONTAINER_IP=$(ip route get 1 | awk '{print $7; exit}' 2>/dev/null)
    if [[ -z "$CONTAINER_IP" ]]; then
        CONTAINER_IP=$(hostname -I | awk '{print $1}')
    fi
    if [[ -z "$CONTAINER_IP" ]]; then
        echo "WARNING: Could not determine container IP, using localhost"
        CONTAINER_IP="localhost"
    fi
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
echo "----------------------------------------"

# Export environment variables for Rust application
export RTSP_INPUT="${RTSP_OUTPUT_URL}"
export ONVIF_PORT="${ONVIF_PORT}"
export DEVICE_NAME="${DEVICE_NAME}"
export ONVIF_USERNAME="${ONVIF_USERNAME}"
export ONVIF_PASSWORD="${ONVIF_PASSWORD}"

# Start WS-Discovery service for ONVIF device discovery
echo "Starting ONVIF device discovery..."
echo "Container IP: $CONTAINER_IP"
echo "INFO: WS-Discovery implemented in Rust (integrated with ONVIF service)"
echo "      ONVIF service is available at: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"
WSDD_PID=0

# Function to manage FFmpeg logs with size capping
manage_ffmpeg_log() {
    local log_file="/tmp/ffmpeg.log"
    local max_lines=1000
    local keep_head=500
    local keep_tail=500
    
    # Monitor log file size and truncate if needed
    while true; do
        sleep 30
        if [[ -f "$log_file" ]]; then
            local line_count=$(wc -l < "$log_file")
            if [[ $line_count -gt $max_lines ]]; then
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

# Function to dump FFmpeg log on error
dump_ffmpeg_log() {
    local log_file="/tmp/ffmpeg.log"
    if [[ -f "$log_file" ]]; then
        echo "========================================" 
        echo "FFmpeg Log (Last 100 lines):"
        echo "========================================"
        tail -n 100 "$log_file"
        echo "========================================"
    else
        echo "No FFmpeg log file found"
    fi
}

# Start ffmpeg to re-encode input stream to ONVIF-compatible RTSP
echo "Starting FFmpeg RTSP server..."
echo "Re-encoding: ${INPUT_URL} -> rtsp://0.0.0.0:${RTSP_OUTPUT_PORT}${RTSP_PATH}"

# Create log directory
mkdir -p /tmp

# Start log management in background
manage_ffmpeg_log &
LOG_MANAGER_PID=$!

# Start FFmpeg with RTSP server output using the simpler format
# Listen on all interfaces (0.0.0.0) for the RTSP server
ffmpeg \
    -re \
    -i "${INPUT_URL}" \
    -c:v libx264 \
    -preset veryfast \
    -tune zerolatency \
    -profile:v baseline \
    -level 3.1 \
    -pix_fmt yuv420p \
    -g 30 \
    -keyint_min 30 \
    -sc_threshold 0 \
    -b:v 2M \
    -maxrate 4M \
    -bufsize 8M \
    -c:a aac \
    -b:a 128k \
    -ar 48000 \
    -ac 2 \
    -f rtsp \
    -rtsp_transport tcp \
    "rtsp://0.0.0.0:${RTSP_OUTPUT_PORT}${RTSP_PATH}" \
    > /tmp/ffmpeg.log 2>&1 &

FFMPEG_PID=$!
if ! kill -0 $FFMPEG_PID 2>/dev/null; then
    echo "ERROR: Failed to start FFmpeg RTSP server"
    dump_ffmpeg_log
    kill $WSDD_PID 2>/dev/null
    exit 1
fi
echo "FFmpeg RTSP server started with PID: $FFMPEG_PID"

# Wait for ffmpeg to establish the RTSP server
echo "Waiting for FFmpeg to establish RTSP server..."
sleep 8

# Verify FFmpeg is still running
if ! kill -0 $FFMPEG_PID 2>/dev/null; then
    echo "ERROR: FFmpeg process died during startup"
    dump_ffmpeg_log
    kill $WSDD_PID 2>/dev/null
    exit 1
fi

# Test RTSP connection
echo "Testing RTSP server connectivity..."
timeout 5 ffprobe -v quiet -select_streams v:0 -show_entries stream=codec_name -of csv=p=0 "${RTSP_OUTPUT_URL}" >/dev/null 2>&1
if [[ $? -eq 0 ]]; then
    echo "RTSP server test: SUCCESS"
else
    echo "WARNING: RTSP server test failed, but continuing (stream may need more time to initialize)"
    echo "FFmpeg may still be starting up..."
fi

# Start ONVIF web service (Rust application)
echo "Starting ONVIF web service..."
echo "ONVIF endpoints will be available at: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"

# Verify the Rust binary exists
if [[ ! -f "/usr/local/bin/ffmpeg-onvif-emulator" ]]; then
    echo "ERROR: Rust binary not found at /usr/local/bin/ffmpeg-onvif-emulator"
    dump_ffmpeg_log
    kill $FFMPEG_PID $WSDD_PID $LOG_MANAGER_PID 2>/dev/null
    exit 1
fi

/usr/local/bin/ffmpeg-onvif-emulator &
ONVIF_PID=$!
if ! kill -0 $ONVIF_PID 2>/dev/null; then
    echo "ERROR: Failed to start ONVIF service"
    dump_ffmpeg_log
    kill $FFMPEG_PID $WSDD_PID $LOG_MANAGER_PID 2>/dev/null
    exit 1
fi
echo "ONVIF service started with PID: $ONVIF_PID"

echo "========================================"
echo "Services Status:"
echo "  WS-Discovery: Integrated with ONVIF service"
echo "  FFmpeg (RTSP Server): PID $FFMPEG_PID"
echo "  FFmpeg Log Manager: PID $LOG_MANAGER_PID"
echo "  ONVIF Service (with WS-Discovery): PID $ONVIF_PID"
echo "========================================"
echo "ONVIF Camera Emulator is ready!"
echo "Container IP: ${CONTAINER_IP}"
echo "Device discovery: Enabled via native Rust WS-Discovery"
echo "RTSP Stream: ${RTSP_OUTPUT_URL}"
echo "ONVIF Endpoint: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"
echo "Note: Input stream is re-encoded to ONVIF-compatible RTSP"
echo "FFmpeg logs: /tmp/ffmpeg.log (capped at 1000 lines)"
echo "========================================"

# Function to handle shutdown
cleanup() {
    echo "Shutting down services..."
    kill $ONVIF_PID 2>/dev/null
    kill $FFMPEG_PID 2>/dev/null  
    if [[ $WSDD_PID -ne 0 ]]; then
        kill $WSDD_PID 2>/dev/null
    fi
    kill $LOG_MANAGER_PID 2>/dev/null
    wait
    echo "All services stopped."
}

trap cleanup SIGTERM SIGINT

# Monitor all processes
monitor_processes() {
    while true; do
        sleep 10
        
        # Skip WSDD monitoring since it's disabled
        if [[ $WSDD_PID -ne 0 ]] && ! kill -0 $WSDD_PID 2>/dev/null; then
            echo "ERROR: WSDD process died (PID: $WSDD_PID)"
            cleanup
            exit 1
        fi
        
        if ! kill -0 $FFMPEG_PID 2>/dev/null; then
            echo "ERROR: FFmpeg process died (PID: $FFMPEG_PID)"
            dump_ffmpeg_log
            cleanup
            exit 1
        fi
        
        if ! kill -0 $ONVIF_PID 2>/dev/null; then
            echo "ERROR: ONVIF service died (PID: $ONVIF_PID)"
            cleanup
            exit 1
        fi
        
        if ! kill -0 $LOG_MANAGER_PID 2>/dev/null; then
            echo "WARNING: Log manager died (PID: $LOG_MANAGER_PID), restarting..."
            manage_ffmpeg_log &
            LOG_MANAGER_PID=$!
        fi
    done
}

# Start monitoring in background
monitor_processes &
MONITOR_PID=$!

# Wait for all background processes
wait