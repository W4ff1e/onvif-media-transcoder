#!/bin/bash

# ONVIF Media Transcoder Environment Variable Manager
# This script helps set and cleanup environment variables for local development
#
# USAGE:
#   To set variables: source ./scripts/env-setup.sh set
#   To cleanup:       source ./scripts/env-setup.sh cleanup  
#   To check status:  source ./scripts/env-setup.sh status
#   To show help:     ./scripts/env-setup.sh help
#
# Note: Use 'source' (or '.') to modify the current shell's environment

# Detect if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    SCRIPT_SOURCED=false
    SCRIPT_NAME="$(basename "$0")"
else
    SCRIPT_SOURCED=true
    SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}")"
fi

ENV_BACKUP_FILE="/tmp/onvif_env_backup_$$"

# Default values based on Dockerfile and tasks.json
DEFAULT_RTSP_STREAM_URL="rtsp://10.0.1.10:8554/stream"
DEFAULT_ONVIF_PORT="8080"
DEFAULT_DEVICE_NAME="Local-ONVIF-Transcoder"
DEFAULT_ONVIF_USERNAME="admin"
DEFAULT_ONVIF_PASSWORD="onvif-rust"
DEFAULT_CONTAINER_IP="10.0.1.10"
DEFAULT_WS_DISCOVERY_ENABLED="true"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: source $SCRIPT_NAME <command>  [recommended]"
    echo "   or: ./$SCRIPT_NAME <command>"
    echo
    echo "Commands:"
    echo "  set      Set environment variables for ONVIF Media Transcoder"
    echo "  cleanup  Remove environment variables and restore previous values"
    echo "  status   Show current environment variable status"
    echo "  help     Show this help message"
    echo
    echo "IMPORTANT: To modify your current shell environment, use 'source':"
    echo "  source $SCRIPT_NAME set      # Sets variables in current shell"
    echo "  source $SCRIPT_NAME cleanup  # Restores variables in current shell"
    echo
    echo "Environment Variables Set:"
    echo "  RTSP_STREAM_URL       - RTSP input stream URL"
    echo "  ONVIF_PORT           - Port for ONVIF web service"
    echo "  DEVICE_NAME          - Name of the ONVIF device"
    echo "  ONVIF_USERNAME       - Username for ONVIF authentication"
    echo "  ONVIF_PASSWORD       - Password for ONVIF authentication"
    echo "  CONTAINER_IP         - IP address for the container/service"
    echo "  WS_DISCOVERY_ENABLED - Enable/disable WS-Discovery"
    echo
    echo "Default Values:"
    echo "  RTSP_STREAM_URL:       $DEFAULT_RTSP_STREAM_URL"
    echo "  ONVIF_PORT:           $DEFAULT_ONVIF_PORT"
    echo "  DEVICE_NAME:          $DEFAULT_DEVICE_NAME"
    echo "  ONVIF_USERNAME:       $DEFAULT_ONVIF_USERNAME"
    echo "  ONVIF_PASSWORD:       $DEFAULT_ONVIF_PASSWORD"
    echo "  CONTAINER_IP:         $DEFAULT_CONTAINER_IP"
    echo "  WS_DISCOVERY_ENABLED: $DEFAULT_WS_DISCOVERY_ENABLED"
}

backup_env_var() {
    local var_name="$1"
    local current_value="${!var_name}"
    
    if [ -n "$current_value" ]; then
        echo "export $var_name=\"$current_value\"" >> "$ENV_BACKUP_FILE"
    else
        echo "unset $var_name" >> "$ENV_BACKUP_FILE"
    fi
}

set_env_vars() {
    if [ "$SCRIPT_SOURCED" = false ]; then
        echo -e "${YELLOW}WARNING: Script is not being sourced!${NC}"
        echo -e "${YELLOW}Environment variables will only be set in this subshell.${NC}"
        echo -e "${YELLOW}To set variables in your current shell, use: source $SCRIPT_NAME set${NC}"
        echo
    fi
    
    echo -e "${BLUE}Setting ONVIF Media Transcoder environment variables...${NC}"
    
    # Create backup file
    rm -f "$ENV_BACKUP_FILE"
    touch "$ENV_BACKUP_FILE"
    
    # Backup existing values
    backup_env_var "RTSP_STREAM_URL"
    backup_env_var "ONVIF_PORT"
    backup_env_var "DEVICE_NAME"
    backup_env_var "ONVIF_USERNAME"
    backup_env_var "ONVIF_PASSWORD"
    backup_env_var "CONTAINER_IP"
    backup_env_var "WS_DISCOVERY_ENABLED"
    
    # Set new values
    export RTSP_STREAM_URL="$DEFAULT_RTSP_STREAM_URL"
    export ONVIF_PORT="$DEFAULT_ONVIF_PORT"
    export DEVICE_NAME="$DEFAULT_DEVICE_NAME"
    export ONVIF_USERNAME="$DEFAULT_ONVIF_USERNAME"
    export ONVIF_PASSWORD="$DEFAULT_ONVIF_PASSWORD"
    export CONTAINER_IP="$DEFAULT_CONTAINER_IP"
    export WS_DISCOVERY_ENABLED="$DEFAULT_WS_DISCOVERY_ENABLED"
    
    echo -e "${GREEN}Environment variables set successfully!${NC}"
    echo
    echo "Current values:"
    echo "  RTSP_STREAM_URL:       $RTSP_STREAM_URL"
    echo "  ONVIF_PORT:           $ONVIF_PORT"
    echo "  DEVICE_NAME:          $DEVICE_NAME"
    echo "  ONVIF_USERNAME:       $ONVIF_USERNAME"
    echo "  ONVIF_PASSWORD:       [HIDDEN]"
    echo "  CONTAINER_IP:         $CONTAINER_IP"
    echo "  WS_DISCOVERY_ENABLED: $WS_DISCOVERY_ENABLED"
    echo
    echo -e "${YELLOW}Backup created at: $ENV_BACKUP_FILE${NC}"
    if [ "$SCRIPT_SOURCED" = true ]; then
        echo -e "${YELLOW}Run 'source $SCRIPT_NAME cleanup' to restore previous values${NC}"
        echo -e "${GREEN}You can now run: cargo run${NC}"
    else
        echo -e "${YELLOW}Run 'source $SCRIPT_NAME cleanup' to restore previous values${NC}"
        echo -e "${RED}Note: Variables are only set in this subshell - use 'source' to set in current shell${NC}"
    fi
}

cleanup_env_vars() {
    if [ "$SCRIPT_SOURCED" = false ]; then
        echo -e "${YELLOW}WARNING: Script is not being sourced!${NC}"
        echo -e "${YELLOW}Environment variables will only be modified in this subshell.${NC}"
        echo -e "${YELLOW}To cleanup variables in your current shell, use: source $SCRIPT_NAME cleanup${NC}"
        echo
    fi
    
    if [ ! -f "$ENV_BACKUP_FILE" ]; then
        echo -e "${RED}No backup file found. Nothing to cleanup.${NC}"
        echo "Backup file expected at: $ENV_BACKUP_FILE"
        return 1
    fi
    
    echo -e "${BLUE}Restoring previous environment variables...${NC}"
    
    # Source the backup file to restore previous values
    source "$ENV_BACKUP_FILE"
    
    # Remove the backup file
    rm -f "$ENV_BACKUP_FILE"
    
    echo -e "${GREEN}Environment variables restored successfully!${NC}"
    echo
    echo "Restored values:"
    echo "  RTSP_STREAM_URL:       ${RTSP_STREAM_URL:-[unset]}"
    echo "  ONVIF_PORT:           ${ONVIF_PORT:-[unset]}"
    echo "  DEVICE_NAME:          ${DEVICE_NAME:-[unset]}"
    echo "  ONVIF_USERNAME:       ${ONVIF_USERNAME:-[unset]}"
    echo "  ONVIF_PASSWORD:       ${ONVIF_PASSWORD:+[set]}${ONVIF_PASSWORD:-[unset]}"
    echo "  CONTAINER_IP:         ${CONTAINER_IP:-[unset]}"
    echo "  WS_DISCOVERY_ENABLED: ${WS_DISCOVERY_ENABLED:-[unset]}"
}

show_status() {
    echo -e "${BLUE}Current ONVIF Environment Variable Status:${NC}"
    echo
    
    local all_set=true
    
    # Check each required variable
    if [ -n "$RTSP_STREAM_URL" ]; then
        echo -e "  RTSP_STREAM_URL:       ${GREEN}✓${NC} $RTSP_STREAM_URL"
    else
        echo -e "  RTSP_STREAM_URL:       ${RED}✗ [not set]${NC}"
        all_set=false
    fi
    
    if [ -n "$ONVIF_PORT" ]; then
        echo -e "  ONVIF_PORT:           ${GREEN}✓${NC} $ONVIF_PORT"
    else
        echo -e "  ONVIF_PORT:           ${RED}✗ [not set]${NC}"
        all_set=false
    fi
    
    if [ -n "$DEVICE_NAME" ]; then
        echo -e "  DEVICE_NAME:          ${GREEN}✓${NC} $DEVICE_NAME"
    else
        echo -e "  DEVICE_NAME:          ${RED}✗ [not set]${NC}"
        all_set=false
    fi
    
    if [ -n "$ONVIF_USERNAME" ]; then
        echo -e "  ONVIF_USERNAME:       ${GREEN}✓${NC} $ONVIF_USERNAME"
    else
        echo -e "  ONVIF_USERNAME:       ${RED}✗ [not set]${NC}"
        all_set=false
    fi
    
    if [ -n "$ONVIF_PASSWORD" ]; then
        echo -e "  ONVIF_PASSWORD:       ${GREEN}✓${NC} [HIDDEN]"
    else
        echo -e "  ONVIF_PASSWORD:       ${RED}✗ [not set]${NC}"
        all_set=false
    fi
    
    if [ -n "$CONTAINER_IP" ]; then
        echo -e "  CONTAINER_IP:         ${GREEN}✓${NC} $CONTAINER_IP"
    else
        echo -e "  CONTAINER_IP:         ${RED}✗ [not set]${NC}"
        all_set=false
    fi
    
    if [ -n "$WS_DISCOVERY_ENABLED" ]; then
        echo -e "  WS_DISCOVERY_ENABLED: ${GREEN}✓${NC} $WS_DISCOVERY_ENABLED"
    else
        echo -e "  WS_DISCOVERY_ENABLED: ${RED}✗ [not set]${NC}"
        all_set=false
    fi
    
    echo
    if [ "$all_set" = true ]; then
        echo -e "${GREEN}All environment variables are set! Ready to run: cargo run${NC}"
    else
        echo -e "${YELLOW}Some environment variables are missing.${NC}"
        if [ "$SCRIPT_SOURCED" = true ]; then
            echo -e "${YELLOW}Run 'source $SCRIPT_NAME set' to configure them.${NC}"
        else
            echo -e "${YELLOW}Run 'source $SCRIPT_NAME set' to configure them.${NC}"
        fi
    fi
    
    # Check for backup file
    if [ -f "$ENV_BACKUP_FILE" ]; then
        echo -e "${BLUE}Backup file exists: $ENV_BACKUP_FILE${NC}"
        if [ "$SCRIPT_SOURCED" = true ]; then
            echo -e "${YELLOW}Run 'source $SCRIPT_NAME cleanup' to restore previous values${NC}"
        else
            echo -e "${YELLOW}Run 'source $SCRIPT_NAME cleanup' to restore previous values${NC}"
        fi
    fi
}

# Main script logic
case "${1:-}" in
    "set")
        set_env_vars
        ;;
    "cleanup")
        cleanup_env_vars
        ;;
    "status")
        show_status
        ;;
    "help"|"-h"|"--help")
        print_usage
        ;;
    "")
        echo -e "${RED}Error: No command specified${NC}"
        echo
        print_usage
        exit 1
        ;;
    *)
        echo -e "${RED}Error: Unknown command '$1'${NC}"
        echo
        print_usage
        exit 1
        ;;
esac
