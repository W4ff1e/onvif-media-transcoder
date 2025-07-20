#!/bin/sh

echo "========================================"
echo "FFmpeg ONVIF Camera Emulator Starting"
echo "========================================"

# Strict validation - all required environment variables must be set
validate_env() {
    local errors=0
    
    if [[ -z "$INPUT_URL" ]]; then
        echo "ERROR: INPUT_URL environment variable is not set"
        errors=$((errors + 1))
    fi
    
    if [[ -z "$OUTPUT_PORT" ]]; then
        echo "ERROR: OUTPUT_PORT environment variable is not set"
        errors=$((errors + 1))
    elif ! [[ "$OUTPUT_PORT" =~ ^[0-9]+$ ]] || [ "$OUTPUT_PORT" -lt 1 ] || [ "$OUTPUT_PORT" -gt 65535 ]; then
        echo "ERROR: OUTPUT_PORT must be a valid port number (1-65535), got: $OUTPUT_PORT"
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
    
    # Validate INPUT_URL format (basic check)
    if [[ -n "$INPUT_URL" ]] && ! [[ "$INPUT_URL" =~ ^https?:// ]]; then
        echo "WARNING: INPUT_URL should start with http:// or https://, got: $INPUT_URL"
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
RTSP_OUTPUT_URL="rtsp://0.0.0.0:${OUTPUT_PORT}/${RTSP_PATH}"
RTSP_INPUT_URL="rtsp://localhost:${OUTPUT_PORT}/${RTSP_PATH}"

echo "Configuration validated successfully:"
echo "  Input URL: ${INPUT_URL}"
echo "  RTSP Output: ${RTSP_OUTPUT_URL}"
echo "  ONVIF Port: ${ONVIF_PORT}"
echo "  Device Name: ${DEVICE_NAME}"
echo "----------------------------------------"

# Export environment variables for Rust application
export RTSP_INPUT="${RTSP_INPUT_URL}"
export ONVIF_PORT="${ONVIF_PORT}"
export DEVICE_NAME="${DEVICE_NAME}"

# Start WS-Discovery service for ONVIF device discovery
echo "Starting WSDD (WS-Discovery) service..."

# Get container IP for WSDD
CONTAINER_IP=$(hostname -i)
if [[ -z "$CONTAINER_IP" ]]; then
    echo "ERROR: Unable to determine container IP address"
    exit 1
fi

echo "Container IP: $CONTAINER_IP"

wsdd \
    --type NetworkVideoTransmitter \
    --xaddr "http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/device_service" \
    --scope "onvif://www.onvif.org/Profile/Streaming" \
    --endpoint ${DEVICE_NAME} &

WSDD_PID=$!
if ! kill -0 $WSDD_PID 2>/dev/null; then
    echo "ERROR: Failed to start WSDD service"
    exit 1
fi
echo "WSDD started with PID: $WSDD_PID"

# Start ffmpeg to convert input stream to RTSP
echo "Starting FFmpeg stream conversion..."
echo "Converting: ${INPUT_URL} -> ${RTSP_OUTPUT_URL}"

ffmpeg \
    -fflags +genpts \
    -re \
    -i "${INPUT_URL}" \
    -c:v libx264 \
    -preset veryfast \
    -crf 23 \
    -maxrate 4M \
    -bufsize 8M \
    -g 30 \
    -keyint_min 30 \
    -sc_threshold 0 \
    -c:a aac \
    -b:a 128k \
    -ar 48000 \
    -f rtsp \
    -rtsp_transport tcp \
    "${RTSP_OUTPUT_URL}" &

FFMPEG_PID=$!
if ! kill -0 $FFMPEG_PID 2>/dev/null; then
    echo "ERROR: Failed to start FFmpeg stream conversion"
    kill $WSDD_PID 2>/dev/null
    exit 1
fi
echo "FFmpeg started with PID: $FFMPEG_PID"

# Wait for ffmpeg to establish the stream
echo "Waiting for FFmpeg to establish RTSP stream..."
sleep 5

# Verify FFmpeg is still running
if ! kill -0 $FFMPEG_PID 2>/dev/null; then
    echo "ERROR: FFmpeg process died during startup"
    kill $WSDD_PID 2>/dev/null
    exit 1
fi

# Start ONVIF web service (Rust application)
echo "Starting ONVIF web service..."
echo "ONVIF endpoints will be available at: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"

# Ensure we are in /app directory
cd /app || { 
    echo "ERROR: Failed to change to /app directory"; 
    kill $FFMPEG_PID $WSDD_PID 2>/dev/null
    exit 1
}

# Verify the Rust binary exists
if [[ ! -f "./target/release/ffplay-onvif-emulator" ]]; then
    echo "ERROR: Rust binary not found at ./target/release/ffplay-onvif-emulator"
    kill $FFMPEG_PID $WSDD_PID 2>/dev/null
    exit 1
fi

./target/release/ffplay-onvif-emulator &
ONVIF_PID=$!
if ! kill -0 $ONVIF_PID 2>/dev/null; then
    echo "ERROR: Failed to start ONVIF service"
    kill $FFMPEG_PID $WSDD_PID 2>/dev/null
    exit 1
fi
echo "ONVIF service started with PID: $ONVIF_PID"

echo "========================================"
echo "Services Status:"
echo "  WSDD (Device Discovery): PID $WSDD_PID"
echo "  FFmpeg (Stream Converter): PID $FFMPEG_PID"
echo "  ONVIF Service: PID $ONVIF_PID"
echo "========================================"
echo "ONVIF Camera Emulator is ready!"
echo "Container IP: ${CONTAINER_IP}"
echo "Device discoverable via WSDD"
echo "Stream URI: ${RTSP_INPUT_URL}"
echo "ONVIF Endpoint: http://${CONTAINER_IP}:${ONVIF_PORT}/onvif/"
echo "========================================"

# Function to handle shutdown
cleanup() {
    echo "Shutting down services..."
    kill $ONVIF_PID 2>/dev/null
    kill $FFMPEG_PID 2>/dev/null  
    kill $WSDD_PID 2>/dev/null
    wait
    echo "All services stopped."
}

trap cleanup SIGTERM SIGINT

# Monitor all processes
monitor_processes() {
    while true; do
        sleep 10
        
        # Check if any process died
        if ! kill -0 $WSDD_PID 2>/dev/null; then
            echo "ERROR: WSDD process died (PID: $WSDD_PID)"
            cleanup
            exit 1
        fi
        
        if ! kill -0 $FFMPEG_PID 2>/dev/null; then
            echo "ERROR: FFmpeg process died (PID: $FFMPEG_PID)"
            cleanup
            exit 1
        fi
        
        if ! kill -0 $ONVIF_PID 2>/dev/null; then
            echo "ERROR: ONVIF service died (PID: $ONVIF_PID)"
            cleanup
            exit 1
        fi
    done
}

# Start monitoring in background
monitor_processes &
MONITOR_PID=$!

# Wait for all background processes
wait