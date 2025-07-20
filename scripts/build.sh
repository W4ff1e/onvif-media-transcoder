#!/bin/bash

set -e

# Configuration
IMAGE_NAME="onvif-media-transcoder"
DEFAULT_TAG="latest"
DOCKERFILE="Dockerfile"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Build Docker image for ONVIF Media Transcoder"
    echo ""
    echo "Options:"
    echo "  -t, --tag TAG          Set image tag (default: ${DEFAULT_TAG})"
    echo "  -r, --registry REGISTRY Set registry prefix (e.g., docker.io/username)"
    echo "  -f, --file DOCKERFILE  Dockerfile to use (default: ${DOCKERFILE})"
    echo "      --no-cache         Build without using cache"
    echo "      --platform PLATFORM Target platform (e.g., linux/amd64,linux/arm64)"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                     # Build with default settings"
    echo "  $0 -t v1.0.0          # Build with specific tag"
    echo "  $0 -r docker.io/myuser -t v1.0.0  # Build with registry and tag"
    echo "  $0 --platform linux/amd64,linux/arm64  # Multi-platform build"
}

# Parse command line arguments
TAG="$DEFAULT_TAG"
REGISTRY=""
NO_CACHE=""
PLATFORM=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--tag)
            TAG="$2"
            shift 2
            ;;
        -r|--registry)
            REGISTRY="$2"
            shift 2
            ;;
        -f|--file)
            DOCKERFILE="$2"
            shift 2
            ;;
        --no-cache)
            NO_CACHE="--no-cache"
            shift
            ;;
        --platform)
            PLATFORM="--platform $2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Construct full image name
if [[ -n "$REGISTRY" ]]; then
    FULL_IMAGE_NAME="${REGISTRY}/${IMAGE_NAME}:${TAG}"
else
    FULL_IMAGE_NAME="${IMAGE_NAME}:${TAG}"
fi

# Validate Dockerfile exists
if [[ ! -f "$DOCKERFILE" ]]; then
    print_error "Dockerfile not found: $DOCKERFILE"
    exit 1
fi

# Display build information
echo "========================================"
echo "ONVIF Media Transcoder - Docker Build"
echo "========================================"
print_status "Image name: $FULL_IMAGE_NAME"
print_status "Dockerfile: $DOCKERFILE"
print_status "Build context: $(pwd)"
if [[ -n "$PLATFORM" ]]; then
    print_status "Platform: ${PLATFORM#--platform }"
fi
if [[ -n "$NO_CACHE" ]]; then
    print_warning "Building without cache"
fi
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed or not in PATH"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    print_error "Docker daemon is not running"
    exit 1
fi

# Start build
print_status "Starting Docker build..."
echo ""

# Construct Docker build command
BUILD_CMD="docker build $NO_CACHE $PLATFORM -f $DOCKERFILE -t $FULL_IMAGE_NAME ."

# Show the command being executed
print_status "Executing: $BUILD_CMD"
echo ""

# Execute the build
if eval "$BUILD_CMD"; then
    echo ""
    print_success "Docker image built successfully!"
    print_success "Image: $FULL_IMAGE_NAME"
    
    # Show image information
    echo ""
    print_status "Image information:"
    docker images "$FULL_IMAGE_NAME" --format "table {{.Repository}}\t{{.Tag}}\t{{.ID}}\t{{.CreatedAt}}\t{{.Size}}"
    
    # Show example run command
    echo ""
    print_status "Example run command:"
    echo "docker run --rm -p 8080:8080 -p 8554:8554 $FULL_IMAGE_NAME"
    
else
    echo ""
    print_error "Docker build failed!"
    exit 1
fi

echo ""
echo "========================================"
print_success "Build completed successfully!"
echo "========================================"
