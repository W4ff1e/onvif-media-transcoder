# Local development

How to build, run and test the ONVIF service outside the container.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.87 or newer
- Docker, for the image and the end-to-end test
- Optional: `ffmpeg`/`ffprobe` for snapshots and stream probing, and
  [MediaMTX](https://github.com/bluenviron/mediamtx) if you want a real RTSP
  stream behind the service

## Running

Every option is a flag and an environment variable; `cargo run -- --help`
lists them.

```bash
# Flags
cargo run -- \
  --rtsp-stream-url rtsp://127.0.0.1:8554/stream \
  --onvif-port 8080 \
  --device-name "Dev Camera" \
  --onvif-username admin --onvif-password onvif-rust \
  --container-ip 127.0.0.1 \
  --ws-discovery-enabled --debug

# Environment variables (what the container does)
RTSP_STREAM_URL=rtsp://127.0.0.1:8554/stream ONVIF_PORT=8080 \
DEVICE_NAME="Dev Camera" WS_DISCOVERY_ENABLED=true cargo run
```

| Flag | Environment variable | Default |
| :--- | :--- | :--- |
| `--rtsp-stream-url` | `RTSP_STREAM_URL` | `rtsp://127.0.0.1:8554/stream` |
| `--onvif-port` | `ONVIF_PORT` | `8080` |
| `--device-name` | `DEVICE_NAME` | `ONVIF-Media-Transcoder` |
| `--onvif-username` | `ONVIF_USERNAME` | `admin` |
| `--onvif-password` | `ONVIF_PASSWORD` | `onvif-rust` |
| `--container-ip` | `CONTAINER_IP` | `127.0.0.1` |
| `--ws-discovery-enabled` | `WS_DISCOVERY_ENABLED` | `false` |
| `--debug` | `DEBUG_LOGGING` | `false` |

The service starts without a stream behind the RTSP URL; SOAP operations work,
`GetStreamUri` returns the configured URL, and snapshots return HTTP 502 until
the stream exists. `RUST_LOG` (for example `RUST_LOG=onvif_media_transcoder=debug`)
overrides `--debug`.

### With a local stream

```bash
# Terminal 1: an RTSP server
mediamtx

# Terminal 2: a test pattern published to it
ffmpeg -re -f lavfi -i testsrc=size=1280x720:rate=25 -c:v libx264 -preset ultrafast \
  -f rtsp rtsp://127.0.0.1:8554/stream

# Terminal 3: the ONVIF service
cargo run -- --debug
```

## Tests

```bash
cargo test                       # unit tests plus socket-level integration tests
cargo test -- --nocapture        # with log output
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo deny check                 # advisories, licenses, bans (cargo install cargo-deny)
```

The integration tests in `tests/integration_test.rs` start the real HTTP
server on a random port and exercise Basic and Digest authentication, replay
protection, split TCP writes and large bodies.

For the container:

```bash
docker build -t onvif-media-transcoder:test .
scripts/e2e-test.sh onvif-media-transcoder:test
```

## Manual requests

```bash
# Public operation, no credentials
curl -s -X POST http://localhost:8080/onvif/device_service \
  -H 'Content-Type: application/soap+xml' \
  -d '<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body><GetCapabilities/></s:Body></s:Envelope>'

# HTTP Digest
curl -s --digest -u admin:onvif-rust -X POST http://localhost:8080/onvif/media_service \
  -H 'Content-Type: application/soap+xml' \
  -d '<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body><GetProfiles/></s:Body></s:Envelope>'

# WS-Security PasswordText
curl -s -X POST http://localhost:8080/onvif/media_service \
  -H 'Content-Type: application/soap+xml' \
  -d '<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><s:Header><wsse:Security><wsse:UsernameToken><wsse:Username>admin</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordText">onvif-rust</wsse:Password></wsse:UsernameToken></wsse:Security></s:Header><s:Body><GetProfiles/></s:Body></s:Envelope>'
```

## VS Code

`.vscode/tasks.json` provides build, test, clippy and run tasks, and
`.vscode/launch.json` provides debug configurations for the
[CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
extension. Press `F5` to build and debug with the default flags.

## Troubleshooting

- **Port already in use**: change `--onvif-port`, or stop the other process.
  WS-Discovery shares UDP 3702 with other responders via `SO_REUSEADDR`.
- **WS-Discovery does nothing locally**: the responder joins the multicast
  group on `--container-ip`; use the address of a real interface rather than
  `127.0.0.1` when probing from another machine.
- **Authentication fails**: with Digest, the `uri` in the `Authorization`
  header must match the request path, and nonces expire after 5 minutes.
