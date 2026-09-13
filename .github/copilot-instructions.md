# Copilot instructions for ONVIF Media Transcoder

## What this project is

A Rust service plus MediaMTX, shipped as one container, that exposes a pulled
stream (HLS, RTSP, ...) as an ONVIF Profile S camera with WS-Discovery. The
primary client is UniFi Protect's third-party camera support; other ONVIF
NVRs should work too.

Key files: `src/onvif/mod.rs` (HTTP + dispatch), `src/onvif/auth.rs`,
`src/onvif/soap.rs`, `src/onvif/responses.rs`, `src/ws_discovery.rs`,
`entrypoint.sh`, `Dockerfile`, `scripts/e2e-test.sh`.

## Hard requirements

- `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`
  and `cargo test` must pass. CI enforces them, plus `cargo deny check`,
  shellcheck, hadolint, markdownlint and the container end-to-end test.
- Parse XML with `roxmltree`; never detect operations or credentials by
  substring search.
- Authentication: compare secrets with `auth::constant_time_eq`, never log
  passwords or digests above `debug`, keep Digest nonce/replay handling intact.
- Escape dynamic values in XML with `soap::xml_escape`.
- External commands go through `onvif::process::run_with_timeout`.
- Logging uses `tracing` macros; no `println!` outside `main`'s startup error path.
- Container changes must keep the image running as the non-root user and keep
  `scripts/e2e-test.sh` passing.

## Conventions

- Configuration is a `clap` struct in `src/config.rs`; every option has an
  environment variable. Boolean options accept true/false/yes/no/on/off/1/0.
- Errors: `Result<T, E>` with descriptive messages; `unwrap`/`expect` only in
  tests or for invariants that are documented at the call site.
- Tests live next to the code (`#[cfg(test)]`) and in `tests/` for
  socket-level behaviour. Every new SOAP response needs a well-formedness test.
- Keep responses schema-shaped (attribute vs element forms) and check the
  ONVIF specifications at <https://www.onvif.org/profiles/specifications/>.
