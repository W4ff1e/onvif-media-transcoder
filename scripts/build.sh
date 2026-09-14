#!/usr/bin/env bash
# Build the Docker image, optionally for several platforms.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

IMAGE_NAME="onvif-media-transcoder"
TAG="latest"
REGISTRY=""
PLATFORM=""
NO_CACHE=0
PUSH=0

usage() {
    cat <<EOF
Usage: scripts/build.sh [options]

Options:
  -t, --tag TAG            Image tag (default: $TAG)
  -r, --registry PREFIX    Registry/namespace prefix, e.g. docker.io/myuser
      --platform LIST      Target platforms, e.g. linux/amd64,linux/arm64
      --push               Push after building (required for multi-platform)
      --no-cache           Build without cache
  -h, --help               Show this help

Examples:
  scripts/build.sh
  scripts/build.sh -t v0.31.0 -r docker.io/myuser
  scripts/build.sh --platform linux/amd64,linux/arm64 --push -r docker.io/myuser
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        -t|--tag) TAG="$2"; shift 2 ;;
        -r|--registry) REGISTRY="$2"; shift 2 ;;
        --platform) PLATFORM="$2"; shift 2 ;;
        --push) PUSH=1; shift ;;
        --no-cache) NO_CACHE=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
    esac
done

FULL_IMAGE="${IMAGE_NAME}:${TAG}"
[ -n "$REGISTRY" ] && FULL_IMAGE="${REGISTRY}/${FULL_IMAGE}"

command -v docker > /dev/null 2>&1 || { echo "docker is not installed" >&2; exit 1; }
docker info > /dev/null 2>&1 || { echo "the Docker daemon is not running" >&2; exit 1; }

args=(buildx build -t "$FULL_IMAGE")
[ "$NO_CACHE" -eq 1 ] && args+=(--no-cache)
if [ -n "$PLATFORM" ]; then
    args+=(--platform "$PLATFORM")
    if [ "$PUSH" -eq 1 ]; then
        args+=(--push)
    else
        echo "note: multi-platform images cannot be loaded locally; add --push to publish them" >&2
    fi
elif [ "$PUSH" -eq 1 ]; then
    args+=(--push)
else
    args+=(--load)
fi
args+=(.)

echo "Building $FULL_IMAGE"
docker "${args[@]}"
echo "Done: $FULL_IMAGE"
[ "$PUSH" -eq 1 ] || echo "Run it with: docker run --rm --network host $FULL_IMAGE"
