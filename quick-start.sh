#!/bin/bash

# ONVIF Media Transcoder Quick Start Script
# This script provides easy commands for common operations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  ONVIF Media Transcoder${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

show_help() {
    print_header
    echo
    echo "Usage: ./quick-start.sh [COMMAND]"
    echo
    echo "Commands:"
    echo "  setup       - Create .env file from template"
    echo "  build       - Build the Docker image"
    echo "  run         - Run with default configuration"
    echo "  compose     - Start using Docker Compose"
    echo "  stop        - Stop running containers"
    echo "  logs        - Show container logs"
    echo "  test        - Test ONVIF endpoints"
    echo "  clean       - Remove containers and images"
    echo "  help        - Show this help message"
    echo
    echo "Examples:"
    echo "  ./quick-start.sh setup    # Create configuration file"
    echo "  ./quick-start.sh run      # Quick start with defaults"
    echo "  ./quick-start.sh compose  # Start with Docker Compose"
    echo
}

setup_env() {
    print_info "Setting up environment configuration..."
    
    if [ -f ".env" ]; then
        print_warning ".env file already exists"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Keeping existing .env file"
            return 0
        fi
    fi
    
    cp .env.example .env
    print_success ".env file created from template"
    print_info "Edit .env file to customize your configuration"
}

build_image() {
    print_info "Building Docker image..."
    docker build -t onvif-media-transcoder .
    print_success "Docker image built successfully"
}

run_container() {
    print_info "Starting ONVIF Media Transcoder..."
    
    # Check if .env exists
    if [ ! -f ".env" ]; then
        print_warning ".env file not found, using defaults"
    fi
    
    print_info "Starting container with host networking..."
    docker run --rm --network host --name onvif-media-transcoder onvif-media-transcoder
}

compose_up() {
    print_info "Starting with Docker Compose..."
    
    if [ ! -f ".env" ]; then
        print_warning ".env file not found, creating from template..."
        setup_env
    fi
    
    docker-compose up -d
    print_success "Services started in background"
    print_info "Use './quick-start.sh logs' to view logs"
    print_info "Use './quick-start.sh stop' to stop services"
}

compose_down() {
    print_info "Stopping Docker Compose services..."
    docker-compose down
    print_success "Services stopped"
}

show_logs() {
    print_info "Showing container logs..."
    if docker-compose ps -q onvif-media-transcoder > /dev/null 2>&1; then
        docker-compose logs -f onvif-media-transcoder
    elif docker ps --filter "name=onvif-media-transcoder" --format "table {{.Names}}" | grep -q onvif-media-transcoder; then
        docker logs -f onvif-media-transcoder
    else
        print_error "No running container found"
        exit 1
    fi
}

test_endpoints() {
    print_info "Testing ONVIF endpoints..."
    
    # Test device capabilities (no auth required)
    print_info "Testing GetCapabilities endpoint..."
    if curl -s -X POST http://localhost:8080/onvif/device_service \
        -H "Content-Type: application/soap+xml" \
        -d '<?xml version="1.0"?><soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body><GetCapabilities/></soap:Body></soap:Envelope>' \
        > /dev/null; then
        print_success "ONVIF service is responding"
    else
        print_error "ONVIF service is not responding"
    fi
    
    # Test RTSP stream
    print_info "Testing RTSP stream..."
    if command -v ffprobe > /dev/null 2>&1; then
        if ffprobe -v quiet -select_streams v:0 -show_entries stream=codec_name -of csv=p=0 rtsp://localhost:8554/stream > /dev/null 2>&1; then
            print_success "RTSP stream is available"
        else
            print_warning "RTSP stream may not be ready yet"
        fi
    else
        print_warning "ffprobe not found, skipping RTSP test"
    fi
    
    print_info "Service URLs:"
    echo "  ONVIF: http://localhost:8080/onvif/device_service"
    echo "  RTSP:  rtsp://localhost:8554/stream"
}

clean_up() {
    print_info "Cleaning up containers and images..."
    
    # Stop and remove containers
    docker-compose down 2>/dev/null || true
    docker stop onvif-media-transcoder 2>/dev/null || true
    docker rm onvif-media-transcoder 2>/dev/null || true
    
    # Remove image
    docker rmi onvif-media-transcoder 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Main script logic
case "${1:-help}" in
    setup)
        setup_env
        ;;
    build)
        build_image
        ;;
    run)
        build_image
        run_container
        ;;
    compose)
        compose_up
        ;;
    stop)
        compose_down
        ;;
    logs)
        show_logs
        ;;
    test)
        test_endpoints
        ;;
    clean)
        clean_up
        ;;
    help|*)
        show_help
        ;;
esac
