#!/bin/bash

# ONVIF Media Transcoder Run Script
# This script sets the required environment variables and runs cargo run

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values based on Dockerfile and tasks.json
DEFAULT_RTSP_STREAM_URL="rtsp://127.0.0.1:8554/stream"
DEFAULT_ONVIF_PORT="8080"
DEFAULT_DEVICE_NAME="Local-ONVIF-Transcoder"
DEFAULT_ONVIF_USERNAME="admin"
DEFAULT_ONVIF_PASSWORD="onvif-rust"
DEFAULT_CONTAINER_IP="127.0.0.1"
DEFAULT_WS_DISCOVERY_ENABLED="true"

echo -e "${BLUE}ONVIF Media Transcoder - Direct Run${NC}"
echo -e "${BLUE}===================================${NC}"
echo

# Set environment variables for this process
export RTSP_STREAM_URL="$DEFAULT_RTSP_STREAM_URL"
export ONVIF_PORT="$DEFAULT_ONVIF_PORT"
export DEVICE_NAME="$DEFAULT_DEVICE_NAME"
export ONVIF_USERNAME="$DEFAULT_ONVIF_USERNAME"
export ONVIF_PASSWORD="$DEFAULT_ONVIF_PASSWORD"
export CONTAINER_IP="$DEFAULT_CONTAINER_IP"
export WS_DISCOVERY_ENABLED="$DEFAULT_WS_DISCOVERY_ENABLED"

echo -e "${GREEN}Environment variables set:${NC}"
echo "  RTSP_STREAM_URL:       $RTSP_STREAM_URL"
echo "  ONVIF_PORT:           $ONVIF_PORT"
echo "  DEVICE_NAME:          $DEVICE_NAME"
echo "  ONVIF_USERNAME:       $ONVIF_USERNAME"
echo "  ONVIF_PASSWORD:       [HIDDEN]"
echo "  CONTAINER_IP:         $CONTAINER_IP"
echo "  WS_DISCOVERY_ENABLED: $WS_DISCOVERY_ENABLED"
echo

echo -e "${BLUE}Starting cargo run...${NC}"
echo

# Change to project root directory (parent of scripts)
cd "$(dirname "$0")/.." || {
    echo -e "${RED}Error: Could not change to project root directory${NC}"
    exit 1
}

# Run cargo with the environment variables
exec cargo run
