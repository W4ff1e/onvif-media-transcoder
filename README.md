# ONVIF Media Transcoder

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/W4ff1e/onvif-media-transcoder/ci.yml?branch=main&label=CI)](https://github.com/W4ff1e/onvif-media-transcoder/actions/workflows/ci.yml)
[![Docker Hub](https://img.shields.io/docker/pulls/w4ff1e/onvif-media-transcoder?logo=docker)](https://hub.docker.com/r/w4ff1e/onvif-media-transcoder)
[![Docker Image Version](https://img.shields.io/docker/v/w4ff1e/onvif-media-transcoder?logo=docker&sort=semver)](https://hub.docker.com/r/w4ff1e/onvif-media-transcoder/tags)
[![Docker Image Size](https://img.shields.io/docker/image-size/w4ff1e/onvif-media-transcoder/latest?logo=docker)](https://hub.docker.com/r/w4ff1e/onvif-media-transcoder)
[![Rust](https://img.shields.io/badge/rust-1.87%2B-orange.svg)](https://www.rust-lang.org)
[![Security](https://img.shields.io/badge/security-policy-red.svg)](SECURITY.md)

Turns any stream MediaMTX can pull (HLS, RTSP, RTMP, SRT, UDP) into an ONVIF
Profile S camera with WS-Discovery, so NVRs that only speak ONVIF, such as
UniFi Protect's third-party camera support, can adopt it like a real camera.

> **AI-assisted project**: much of this code was written with AI tools and then
> reviewed and tested. Review it yourself before relying on it, and keep the
> ONVIF and RTSP ports off untrusted networks.

## How it works

```text
 HLS / RTSP / RTMP / SRT          ┌──────────────────────── container ────────────────────────┐
 source stream  ───────────────▶  │  MediaMTX  ── re-mux, no re-encode ──▶  RTSP :8554 (auth) │
                                  │      ▲                                                     │
                                  │      │ probes codec/resolution, grabs snapshots (ffmpeg)   │
                                  │  ONVIF service (Rust) ─▶ SOAP :8080  ─▶ device + media ops │
                                  │  WS-Discovery responder ─▶ UDP 3702 multicast              │
                                  └────────────────────────────────────────────────────────────┘
                                                      ▲            ▲              ▲
                                            NVR discovers,  reads profiles,  plays RTSP with
                                            (Probe/Hello)   stream + snapshot   the same credentials
```

- **MediaMTX** pulls the input and serves it as RTSP without re-encoding.
- The **ONVIF service** answers device and media SOAP operations, authenticates
  with HTTP Basic, HTTP Digest or WS-Security, and serves JPEG snapshots.
- The **WS-Discovery responder** answers probes so the camera appears in the
  NVR's discovery list.

## Quick start

```bash
docker run --rm --network host \
  -e INPUT_URL="https://your-stream.example/live/index.m3u8" \
  -e DEVICE_NAME="Front Door" \
  -e ONVIF_USERNAME="camera" \
  -e ONVIF_PASSWORD="change-me-please" \
  w4ff1e/onvif-media-transcoder:latest
```

Then add a third-party ONVIF camera in your NVR. It should be discovered
automatically; otherwise enter `http://<host-ip>:8080/onvif/device_service`
with the username and password above. The same credentials are used for the
RTSP stream.

Other ways to run it:

```bash
git clone https://github.com/W4ff1e/onvif-media-transcoder.git
cd onvif-media-transcoder
scripts/quick-start.sh setup     # creates .env from examples/.env.example
scripts/quick-start.sh run       # builds the image and runs it
```

or with Compose using [`examples/docker-compose.yml`](examples/README.md).

`--network host` is recommended: WS-Discovery is multicast, and the device
advertises its own address, which must be reachable by the NVR. Without host
networking, publish `8080/tcp`, `8554/tcp`, `3702/udp` and set `CONTAINER_IP`
to the Docker host's LAN address.

### Image tags

- `latest`: latest release
- `unstable`: latest commit on `main` that passed CI
- `0.31.0`, `0.31`: specific releases

## Configuration

Everything is configured through environment variables.

| Variable | Default | Description |
| :--- | :--- | :--- |
| `INPUT_URL` | Demo HLS stream | Source stream. Any URL MediaMTX can pull (`http(s)://` HLS, `rtsp://`, `rtsps://`, `rtmp://`, `srt://`, `udp://`). Local files are not supported. |
| `INPUT_CHECK` | `warn` | Behaviour when the input is unreachable at start: `warn` (MediaMTX keeps retrying), `fail` (exit), `skip` |
| `RTSP_OUTPUT_PORT` | `8554` | RTSP port for the re-muxed stream |
| `RTSP_PATH` | `/stream` | RTSP path |
| `RTSP_AUTH_ENABLED` | `true` | Require the ONVIF credentials for RTSP playback |
| `ONVIF_PORT` | `8080` | ONVIF HTTP/SOAP port |
| `DEVICE_NAME` | `ONVIF-Media-Transcoder` | Name shown in discovery and as the model |
| `ONVIF_USERNAME` | `admin` | Username for ONVIF, RTSP and snapshots |
| `ONVIF_PASSWORD` | `onvif-rust` | Password. **Change it.** At least 6 characters. |
| `WS_DISCOVERY_ENABLED` | `true` | Answer WS-Discovery probes on UDP 3702 |
| `CONTAINER_IP` | auto-detected | IPv4 address advertised to clients |
| `DEBUG_LOGGING` | `false` | Verbose logging. Logs credentials; development only. `DEBUGLOGGING` is accepted as an alias. |
| `RUST_LOG` | unset | Fine-grained log filter, e.g. `onvif_media_transcoder=debug`; overrides `DEBUG_LOGGING` |

Boolean variables accept `true/false`, `yes/no`, `on/off` and `1/0`.

## ONVIF compatibility

Profile S device with one fixed media profile (`Profile_1`). The profile's
codec, resolution, frame rate and bitrate are probed from the live stream
with ffprobe; until the probe succeeds, sensible defaults are reported.

**Device service** (`/onvif/device_service`): `GetCapabilities`, `GetServices`,
`GetServiceCapabilities`, `GetSystemDateAndTime`, `GetDeviceInformation`,
`GetHostname`, `GetScopes`, `GetWsdlUrl`.

**Media service** (`/onvif/media_service`): `GetServiceCapabilities`,
`GetProfiles`, `GetProfile`, `GetStreamUri`, `GetSnapshotUri`,
`GetVideoSources`, `GetVideoSourceConfigurations`, `GetVideoSourceConfiguration`,
`GetVideoEncoderConfigurations`, `GetVideoEncoderConfiguration`,
`GetVideoEncoderConfigurationOptions`, `GetAudioSourceConfigurations`,
`GetAudioEncoderConfigurations` (no audio is exposed).

Operations are matched on the SOAP body, so either service path accepts either
set. Anything else returns a `ter:ActionNotSupported` fault. `/snapshot.jpg`
serves a JPEG frame.

### Authentication

`GetCapabilities`, `GetServices`, `GetServiceCapabilities`,
`GetSystemDateAndTime`, `GetHostname` and `GetWsdlUrl` are public, as the ONVIF
specification requires for discovery. Everything else, including snapshots,
needs credentials via one of:

- HTTP Digest (RFC 7616, `qop=auth`, server-issued nonces with replay protection)
- HTTP Basic
- WS-Security UsernameToken (`PasswordDigest` with a 5 minute `Created` window, or `PasswordText`)

RTSP playback uses the same credentials (Basic) unless `RTSP_AUTH_ENABLED=false`.
The `GetStreamUri` response contains the plain RTSP URL; ONVIF clients apply the
device credentials themselves.

### Discovery

WS-Discovery (April 2005) on `239.255.255.250:3702`. The device sends `Hello`
on start and every 60 s, answers `Probe` messages for
`NetworkVideoTransmitter`/`Device` types, and sends `Bye` on shutdown. The
endpoint reference is derived from `DEVICE_NAME`, so the same name is
recognised as the same device across restarts.

## Testing

```bash
# Unit and socket-level tests
cargo test

# Full container test with an in-container source (no internet needed)
docker build -t onvif-media-transcoder:test .
scripts/e2e-test.sh onvif-media-transcoder:test
```

Manual checks against a running container:

```bash
# Public operation
curl -s -X POST http://localhost:8080/onvif/device_service \
  -H 'Content-Type: application/soap+xml' \
  -d '<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body><GetCapabilities/></s:Body></s:Envelope>'

# Protected operation with HTTP Digest
curl -s --digest -u admin:onvif-rust -X POST http://localhost:8080/onvif/media_service \
  -H 'Content-Type: application/soap+xml' \
  -d '<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body><GetProfiles/></s:Body></s:Envelope>'

# Stream and snapshot
ffprobe -rtsp_transport tcp rtsp://admin:onvif-rust@localhost:8554/stream
curl --digest -u admin:onvif-rust -o snapshot.jpg http://localhost:8080/snapshot.jpg
```

## Troubleshooting

- **Not discovered**: use `--network host`; make sure UDP 3702 multicast is
  allowed between the NVR and the host; check the logs for `sent Hello`.
- **Discovered but adoption fails**: the advertised address must be reachable
  from the NVR. Check the `container ip` log line and set `CONTAINER_IP` if
  it is wrong.
- **Stream will not play**: the RTSP stream needs the ONVIF credentials.
  Test with `ffprobe rtsp://user:pass@host:8554/stream`. Set
  `RTSP_AUTH_ENABLED=false` to compare.
- **Input unreachable**: the container logs a warning and MediaMTX retries.
  Set `INPUT_CHECK=fail` if you prefer a hard failure.
- **More detail**: `DEBUG_LOGGING=true` or `RUST_LOG=debug`.

## Development

Requirements: Rust 1.87 or newer, Docker. ffmpeg/ffprobe are optional locally
(snapshots and stream probing fall back gracefully without them).

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- --help         # every flag has an environment variable equivalent
```

See [docs/LOCAL_DEVELOPMENT.md](docs/LOCAL_DEVELOPMENT.md) for running the
service outside the container and the VS Code setup.

```text
├── src/
│   ├── main.rs              # start-up, signal handling, thread orchestration
│   ├── config.rs            # clap configuration (flags and environment)
│   ├── identity.rs          # device identity shared by discovery and SOAP
│   ├── ws_discovery.rs      # WS-Discovery responder
│   └── onvif/
│       ├── mod.rs           # HTTP server, authentication gate, dispatch
│       ├── auth.rs          # Basic, Digest and WS-Security validation
│       ├── soap.rs          # SOAP parsing and envelope/fault builders
│       ├── responses.rs     # ONVIF response templates
│       ├── stream_info.rs   # ffprobe-based stream description
│       ├── snapshot.rs      # ffmpeg snapshot capture
│       └── process.rs       # external commands with timeouts
├── tests/                   # socket-level integration tests
├── scripts/                 # build, publish, quick start, end-to-end test
├── examples/                # Compose files and .env template
├── Dockerfile               # multi-stage build, non-root runtime image
└── entrypoint.sh            # validation, MediaMTX config, supervision
```

## Contributing

Pull requests are welcome. For larger changes, open an issue first. CI runs
`cargo fmt`, `clippy -D warnings`, tests, `cargo deny`, shellcheck, hadolint,
markdownlint and the container end-to-end test; please make sure they pass.

## Security

Change the default credentials, keep the ONVIF and RTSP ports on a trusted
network, and read [SECURITY.md](SECURITY.md) for details and how to report
vulnerabilities.

## License

MIT. See [LICENSE](LICENSE). Provided as is, without warranty.

## Authors

- [@W4ff1e](https://github.com/W4ff1e): initial work and maintenance, with
  AI-assisted development

## Stats

![Alt](https://repobeats.axiom.co/api/embed/f19d8fae5d95fd971fe46aa847f9f23b9e278420.svg "Repobeats analytics image")
