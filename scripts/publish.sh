#!/usr/bin/env bash
# Build (unless --skip-build) and push the image to a registry.
#
# Credentials are never accepted on the command line: log in beforehand with
# `docker login`, or set DOCKER_USERNAME and DOCKER_PASSWORD in the environment
# (DOCKER_PASSWORD is read from stdin by docker login).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

IMAGE_NAME="onvif-media-transcoder"
TAG="latest"
REGISTRY="${DOCKER_REGISTRY:-docker.io}"
USERNAME="${DOCKER_USERNAME:-}"
ADDITIONAL_TAGS=""
PLATFORM="${PLATFORM:-linux/amd64,linux/arm64}"
SKIP_BUILD=0
DRY_RUN=0

usage() {
    cat <<EOF
Usage: scripts/publish.sh -u USERNAME [options]

Options:
  -u, --username USER       Registry namespace/user (or DOCKER_USERNAME)
  -r, --registry HOST       Registry (default: $REGISTRY, or DOCKER_REGISTRY)
  -t, --tag TAG             Primary tag (default: $TAG)
      --additional-tags T   Comma-separated extra tags, e.g. v0.31.0,stable
      --platform LIST       Platforms (default: $PLATFORM)
      --skip-build          Push an already built local image instead
      --dry-run             Print what would happen
  -h, --help                Show this help

Authentication: run 'docker login <registry>' first, or export DOCKER_USERNAME
and DOCKER_PASSWORD. Passwords are never taken as arguments.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        -u|--username) USERNAME="$2"; shift 2 ;;
        -r|--registry) REGISTRY="$2"; shift 2 ;;
        -t|--tag) TAG="$2"; shift 2 ;;
        --additional-tags) ADDITIONAL_TAGS="$2"; shift 2 ;;
        --platform) PLATFORM="$2"; shift 2 ;;
        --skip-build) SKIP_BUILD=1; shift ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
    esac
done

[ -n "$USERNAME" ] || { echo "a username is required (-u or DOCKER_USERNAME)" >&2; exit 1; }

REMOTE="${REGISTRY}/${USERNAME}/${IMAGE_NAME}"
TAGS=("$TAG")
if [ -n "$ADDITIONAL_TAGS" ]; then
    IFS=',' read -r -a extra <<< "$ADDITIONAL_TAGS"
    TAGS+=("${extra[@]}")
fi

run() {
    if [ "$DRY_RUN" -eq 1 ]; then
        echo "would run: $*"
    else
        "$@"
    fi
}

echo "Publishing ${REMOTE} with tags: ${TAGS[*]} (platforms: ${PLATFORM})"

if [ -n "${DOCKER_PASSWORD:-}" ]; then
    if [ "$DRY_RUN" -eq 1 ]; then
        echo "would run: docker login $REGISTRY --username $USERNAME --password-stdin"
    else
        printf '%s' "$DOCKER_PASSWORD" | docker login "$REGISTRY" --username "$USERNAME" --password-stdin
    fi
fi

tag_args=()
for t in "${TAGS[@]}"; do
    tag_args+=(-t "${REMOTE}:${t}")
done

if [ "$SKIP_BUILD" -eq 1 ]; then
    LOCAL="${IMAGE_NAME}:${TAG}"
    docker image inspect "$LOCAL" > /dev/null 2>&1 || { echo "local image $LOCAL not found; build it first or drop --skip-build" >&2; exit 1; }
    for t in "${TAGS[@]}"; do
        run docker tag "$LOCAL" "${REMOTE}:${t}"
        run docker push "${REMOTE}:${t}"
    done
else
    run docker buildx build --platform "$PLATFORM" "${tag_args[@]}" --push .
fi

echo "Done. Pull with: docker pull ${REMOTE}:${TAG}"
