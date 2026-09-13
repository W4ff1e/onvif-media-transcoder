# Examples

| File | Purpose |
| :--- | :--- |
| `docker-compose.yml` | Run the published Docker Hub image with host networking |
| `docker-compose.local.yml` | Build from this repository and run |
| `.env.example` | Every configuration variable with comments |

## Published image

```bash
cp examples/.env.example .env      # then edit INPUT_URL and the credentials
docker compose --env-file .env -f examples/docker-compose.yml up -d
docker compose -f examples/docker-compose.yml logs -f
docker compose -f examples/docker-compose.yml down
```

## Build from source

```bash
docker compose --env-file .env -f examples/docker-compose.local.yml up --build
```

## Networking

Both files use `network_mode: host`, which is what UniFi Protect and other
NVRs need: WS-Discovery is multicast, and the addresses the device advertises
must be reachable from the NVR.

If host networking is not possible, publish `8080/tcp`, `8554/tcp` and
`3702/udp` and set `CONTAINER_IP` to the Docker host's LAN address so the
advertised ONVIF and RTSP URLs point at the host rather than the container.
A commented example is included in `docker-compose.yml`.

## Health checks

Both files probe `GET /` on the ONVIF port. The image itself also defines a
`HEALTHCHECK`, so `docker ps` shows health without Compose as well.
