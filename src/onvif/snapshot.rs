//! JPEG snapshot capture from the RTSP stream using ffmpeg.

use crate::onvif::process::{run_with_timeout, ProcessError};
use std::sync::Mutex;
use std::time::Duration;
use tracing::debug;

/// Hard limit on how long a snapshot capture may take.
pub const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(10);

/// Serialises snapshot captures so that a burst of requests cannot fork
/// an unbounded number of ffmpeg processes.
static CAPTURE_LOCK: Mutex<()> = Mutex::new(());

/// Errors that can occur while capturing a snapshot.
#[derive(Debug)]
pub enum SnapshotError {
    /// ffmpeg is not installed or could not be started.
    Unavailable(std::io::Error),
    /// ffmpeg did not finish within [`SNAPSHOT_TIMEOUT`].
    Timeout,
    /// ffmpeg exited unsuccessfully or produced no image.
    Failed(String),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::Unavailable(e) => write!(f, "ffmpeg unavailable: {e}"),
            SnapshotError::Timeout => write!(f, "snapshot timed out"),
            SnapshotError::Failed(msg) => write!(f, "snapshot failed: {msg}"),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Captures a single JPEG frame from `rtsp_url`.
pub fn capture_jpeg(rtsp_url: &str) -> Result<Vec<u8>, SnapshotError> {
    let _guard = CAPTURE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    debug!("capturing snapshot");

    let output = run_with_timeout(
        "ffmpeg",
        &[
            "-nostdin",
            "-loglevel",
            "error",
            "-rtsp_transport",
            "tcp",
            "-i",
            rtsp_url,
            "-frames:v",
            "1",
            "-f",
            "image2",
            "-update",
            "1",
            "-",
        ],
        SNAPSHOT_TIMEOUT,
    )
    .map_err(|e| match e {
        ProcessError::Unavailable(e) => SnapshotError::Unavailable(e),
        ProcessError::Timeout => SnapshotError::Timeout,
    })?;

    if !output.success || output.stdout.is_empty() {
        return Err(SnapshotError::Failed(output.stderr));
    }

    debug!(bytes = output.stdout.len(), "snapshot captured");
    Ok(output.stdout)
}
