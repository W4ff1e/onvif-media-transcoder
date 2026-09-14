#!/usr/bin/env bash
# Convenience commands for building, running and testing the container.
# Always operates on the repository root, wherever it is invoked from.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

IMAGE="${IMAGE:-onvif-media-transcoder}"
COMPOSE_FILE="${COMPOSE_FILE:-examples/docker-compose.yml}"
ENV_FILE="${ENV_FILE:-.env}"

info()    { printf '\033[0;34mℹ %s\033[0m\n' "$*"; }
success() { printf '\033[0;32m✓ %s\033[0m\n' "$*"; }
warn()    { printf '\033[1;33m⚠ %s\033[0m\n' "$*"; }
error()   { printf '\033[0;31m✗ %s\033[0m\n' "$*" >&2; }

usage() {
    cat <<EOF
Usage: scripts/quick-start.sh <command>

Commands:
  setup     Create $ENV_FILE from examples/.env.example
  build     Build the Docker image ($IMAGE)
  run       Build, then run with host networking using $ENV_FILE if present
  compose   Start with Docker Compose ($COMPOSE_FILE)
  stop      Stop Docker Compose services
  logs      Follow container logs
  test      Run the hermetic end-to-end test against the built image
  clean     Remove containers and the image
  help      Show this help

Environment: IMAGE, COMPOSE_FILE, ENV_FILE override the defaults above.
EOF
}

require_docker() {
    command -v docker > /dev/null 2>&1 || { error "docker is not installed or not in PATH"; exit 1; }
    docker info > /dev/null 2>&1 || { error "the Docker daemon is not running"; exit 1; }
}

setup_env() {
    if [ -f "$ENV_FILE" ]; then
        warn "$ENV_FILE already exists"
        read -r -p "Overwrite it? (y/N): " reply
        [[ "$reply" =~ ^[Yy]$ ]] || { info "keeping existing $ENV_FILE"; return; }
    fi
    cp examples/.env.example "$ENV_FILE"
    success "created $ENV_FILE; edit it to point INPUT_URL at your stream and set credentials"
}

build_image() {
    require_docker
    info "building $IMAGE..."
    docker build -t "$IMAGE" .
    success "built $IMAGE"
}

run_container() {
    require_docker
    local env_args=()
    if [ -f "$ENV_FILE" ]; then
        env_args=(--env-file "$ENV_FILE")
        info "using $ENV_FILE"
    else
        warn "$ENV_FILE not found; running with image defaults (demo stream, admin/onvif-rust)"
    fi
    info "starting with host networking (required for WS-Discovery); Ctrl+C stops it"
    docker run --rm --network host --name onvif-media-transcoder "${env_args[@]}" "$IMAGE"
}

compose_up() {
    require_docker
    [ -f "$ENV_FILE" ] || { warn "$ENV_FILE not found, creating it"; setup_env; }
    docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d
    success "started; use 'scripts/quick-start.sh logs' and 'scripts/quick-start.sh stop'"
}

compose_down() {
    require_docker
    docker compose -f "$COMPOSE_FILE" down
    success "stopped"
}

show_logs() {
    require_docker
    if docker ps --format '{{.Names}}' | grep -qx onvif-media-transcoder; then
        docker logs -f onvif-media-transcoder
    else
        error "no running container named onvif-media-transcoder"
        exit 1
    fi
}

run_tests() {
    require_docker
    docker image inspect "$IMAGE" > /dev/null 2>&1 || build_image
    scripts/e2e-test.sh "$IMAGE"
}

clean_up() {
    require_docker
    docker compose -f "$COMPOSE_FILE" down 2> /dev/null || true
    docker rm -f onvif-media-transcoder 2> /dev/null || true
    docker rmi "$IMAGE" 2> /dev/null || true
    success "cleaned up"
}

case "${1:-help}" in
    setup)   setup_env ;;
    build)   build_image ;;
    run)     build_image; run_container ;;
    compose) compose_up ;;
    stop)    compose_down ;;
    logs)    show_logs ;;
    test)    run_tests ;;
    clean)   clean_up ;;
    help|-h|--help) usage ;;
    *) error "unknown command: $1"; usage; exit 1 ;;
esac
