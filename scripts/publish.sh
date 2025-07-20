#!/bin/bash

set -e

# Configuration
IMAGE_NAME="onvif-media-transcoder"
DEFAULT_TAG="latest"
DEFAULT_REGISTRY="docker.io"

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
    echo "Publish Docker image for ONVIF Media Transcoder to registry"
    echo ""
    echo "Options:"
    echo "  -t, --tag TAG          Image tag to publish (default: ${DEFAULT_TAG})"
    echo "  -r, --registry REGISTRY Registry to publish to (default: ${DEFAULT_REGISTRY})"
    echo "  -u, --username USERNAME Registry username (required for most registries)"
    echo "  -p, --password PASSWORD Registry password (will prompt if not provided)"
    echo "      --additional-tags TAGS Additional tags to push (comma-separated)"
    echo "      --skip-build       Skip building image before publish"
    echo "      --dry-run          Show what would be published without actually doing it"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  DOCKER_REGISTRY        Default registry"
    echo "  DOCKER_USERNAME        Default username"
    echo "  DOCKER_PASSWORD        Default password"
    echo ""
    echo "Examples:"
    echo "  $0 -u myuser                    # Publish to Docker Hub"
    echo "  $0 -r ghcr.io -u myuser -t v1.0.0  # Publish to GitHub Container Registry"
    echo "  $0 -u myuser --additional-tags v1.0.0,stable  # Publish with multiple tags"
    echo "  $0 --dry-run -u myuser          # Show what would be published"
}

# Parse command line arguments
TAG="$DEFAULT_TAG"
REGISTRY="${DOCKER_REGISTRY:-$DEFAULT_REGISTRY}"
USERNAME="${DOCKER_USERNAME:-}"
PASSWORD="${DOCKER_PASSWORD:-}"
ADDITIONAL_TAGS=""
SKIP_BUILD=""
DRY_RUN=""

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
        -u|--username)
            USERNAME="$2"
            shift 2
            ;;
        -p|--password)
            PASSWORD="$2"
            shift 2
            ;;
        --additional-tags)
            ADDITIONAL_TAGS="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD="true"
            shift
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
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

# Validate required parameters
if [[ -z "$USERNAME" ]]; then
    print_error "Username is required. Use -u/--username or set DOCKER_USERNAME environment variable"
    exit 1
fi

# Construct image names
LOCAL_IMAGE="${IMAGE_NAME}:${TAG}"
REMOTE_IMAGE="${REGISTRY}/${USERNAME}/${IMAGE_NAME}:${TAG}"

# Prepare list of all tags to push
TAGS_TO_PUSH=("$TAG")
if [[ -n "$ADDITIONAL_TAGS" ]]; then
    IFS=',' read -ra ADDITIONAL_TAG_ARRAY <<< "$ADDITIONAL_TAGS"
    TAGS_TO_PUSH+=("${ADDITIONAL_TAG_ARRAY[@]}")
fi

# Display publish information
echo "========================================"
echo "ONVIF Media Transcoder - Docker Publish"
echo "========================================"
print_status "Local image: $LOCAL_IMAGE"
print_status "Remote image: $REMOTE_IMAGE"
print_status "Registry: $REGISTRY"
print_status "Username: $USERNAME"
print_status "Tags to publish: ${TAGS_TO_PUSH[*]}"
if [[ -n "$DRY_RUN" ]]; then
    print_warning "DRY RUN MODE - No actual publishing will occur"
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

# Build image if not skipping
if [[ -z "$SKIP_BUILD" ]]; then
    print_status "Building image before publish..."
    
    # Check if build script exists
    if [[ -f "./build.sh" ]]; then
        if [[ -n "$DRY_RUN" ]]; then
            print_status "Would execute: ./build.sh -t $TAG"
        else
            if ! ./build.sh -t "$TAG"; then
                print_error "Build failed"
                exit 1
            fi
        fi
    else
        # Fallback to direct docker build
        if [[ -n "$DRY_RUN" ]]; then
            print_status "Would execute: docker build -t $LOCAL_IMAGE ."
        else
            print_status "Building with docker build..."
            if ! docker build -t "$LOCAL_IMAGE" .; then
                print_error "Build failed"
                exit 1
            fi
        fi
    fi
    echo ""
fi

# Check if local image exists
if [[ -z "$DRY_RUN" ]] && ! docker image inspect "$LOCAL_IMAGE" &> /dev/null; then
    print_error "Local image not found: $LOCAL_IMAGE"
    print_error "Please build the image first or remove --skip-build flag"
    exit 1
fi

# Get password if not provided
if [[ -z "$PASSWORD" ]] && [[ -z "$DRY_RUN" ]]; then
    echo -n "Enter password for $USERNAME: "
    read -s PASSWORD
    echo ""
    echo ""
fi

# Login to registry
if [[ -n "$DRY_RUN" ]]; then
    print_status "Would login to registry: $REGISTRY"
else
    print_status "Logging in to registry: $REGISTRY"
    if ! echo "$PASSWORD" | docker login "$REGISTRY" --username "$USERNAME" --password-stdin; then
        print_error "Failed to login to registry"
        exit 1
    fi
    print_success "Successfully logged in to $REGISTRY"
fi

echo ""

# Tag and push all versions
for tag in "${TAGS_TO_PUSH[@]}"; do
    TAGGED_REMOTE_IMAGE="${REGISTRY}/${USERNAME}/${IMAGE_NAME}:${tag}"
    
    # Tag the image
    if [[ -n "$DRY_RUN" ]]; then
        print_status "Would tag: $LOCAL_IMAGE -> $TAGGED_REMOTE_IMAGE"
    else
        print_status "Tagging: $LOCAL_IMAGE -> $TAGGED_REMOTE_IMAGE"
        if ! docker tag "$LOCAL_IMAGE" "$TAGGED_REMOTE_IMAGE"; then
            print_error "Failed to tag image"
            continue
        fi
    fi
    
    # Push the image
    if [[ -n "$DRY_RUN" ]]; then
        print_status "Would push: $TAGGED_REMOTE_IMAGE"
    else
        print_status "Pushing: $TAGGED_REMOTE_IMAGE"
        if docker push "$TAGGED_REMOTE_IMAGE"; then
            print_success "Successfully pushed: $TAGGED_REMOTE_IMAGE"
        else
            print_error "Failed to push: $TAGGED_REMOTE_IMAGE"
        fi
    fi
    echo ""
done

# Show final information
if [[ -z "$DRY_RUN" ]]; then
    echo ""
    print_status "Published images:"
    for tag in "${TAGS_TO_PUSH[@]}"; do
        echo "  ${REGISTRY}/${USERNAME}/${IMAGE_NAME}:${tag}"
    done
    
    echo ""
    print_status "Example pull command:"
    echo "docker pull ${REGISTRY}/${USERNAME}/${IMAGE_NAME}:${TAG}"
    
    echo ""
    print_status "Example run command:"
    echo "docker run --rm -p 8080:8080 -p 8554:8554 ${REGISTRY}/${USERNAME}/${IMAGE_NAME}:${TAG}"
fi

echo ""
echo "========================================"
if [[ -n "$DRY_RUN" ]]; then
    print_success "Dry run completed successfully!"
else
    print_success "Publish completed successfully!"
fi
echo "========================================"
