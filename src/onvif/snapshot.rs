//! JPEG snapshot capture from the RTSP stream using ffmpeg.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

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

    let mut child = Command::new("ffmpeg")
        .args([
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
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SnapshotError::Unavailable)?;

    // Drain stdout on a helper thread so a large frame cannot deadlock the
    // pipe while we wait for the process with a timeout.
    let mut stdout = child.stdout.take().expect("stdout piped");
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= SNAPSHOT_TIMEOUT => {
                warn!("ffmpeg snapshot exceeded timeout, killing");
                let _ = child.kill();
                let _ = child.wait();
                return Err(SnapshotError::Timeout);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(SnapshotError::Unavailable(e)),
        }
    };

    let image = reader.join().unwrap_or_default();
    if !status.success() || image.is_empty() {
        let mut stderr = String::new();
        if let Some(mut err) = child.stderr.take() {
            let _ = err.read_to_string(&mut stderr);
        }
        return Err(SnapshotError::Failed(stderr.trim().to_string()));
    }

    debug!(bytes = image.len(), "snapshot captured");
    Ok(image)
}
