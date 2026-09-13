# Scripts

Helper scripts for building, running, testing and publishing the container.
All of them can be run from any directory; they operate on the repository root.

| Script | Purpose |
| :--- | :--- |
| `quick-start.sh` | Everyday commands: `setup`, `build`, `run`, `compose`, `stop`, `logs`, `test`, `clean` |
| `build.sh` | Build the image, optionally multi-platform with `--platform` and `--push` |
| `publish.sh` | Build and push tagged images to a registry |
| `e2e-test.sh` | Hermetic end-to-end test of a built image (also used by CI) |

## quick-start.sh

```bash
scripts/quick-start.sh setup     # copy examples/.env.example to .env, then edit it
scripts/quick-start.sh run       # build and run with host networking
scripts/quick-start.sh compose   # start via examples/docker-compose.yml
scripts/quick-start.sh test      # run the end-to-end test against the built image
```

`IMAGE`, `COMPOSE_FILE` and `ENV_FILE` environment variables override the
defaults (`onvif-media-transcoder`, `examples/docker-compose.yml`, `.env`).

## build.sh

```bash
scripts/build.sh                                  # local image onvif-media-transcoder:latest
scripts/build.sh -t v0.31.0 -r docker.io/myuser   # tagged, with a registry prefix
scripts/build.sh --platform linux/amd64,linux/arm64 --push -r docker.io/myuser
```

## publish.sh

```bash
docker login docker.io
scripts/publish.sh -u myuser -t v0.31.0 --additional-tags latest
```

Credentials are taken from an existing `docker login` session or from the
`DOCKER_USERNAME` and `DOCKER_PASSWORD` environment variables, never from
command-line arguments.

## e2e-test.sh

```bash
docker build -t onvif-media-transcoder:test .
scripts/e2e-test.sh onvif-media-transcoder:test
```

The test starts the image with a second MediaMTX instance and an ffmpeg test
pattern inside the container as the input, then checks authentication (Basic,
Digest, WS-Security), profiles, RTSP access with and without credentials,
snapshots, WS-Discovery, idle CPU and graceful shutdown. It needs Docker,
curl and (optionally, for the discovery probe) python3 on the host.
