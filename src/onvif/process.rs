//! Running external tools (ffmpeg, ffprobe) with a hard timeout.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tracing::warn;

/// Output of a finished external command.
#[derive(Debug)]
pub struct Output {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

/// Errors from running an external command.
#[derive(Debug)]
pub enum ProcessError {
    /// The binary is missing or could not be started.
    Unavailable(std::io::Error),
    /// The command did not finish within the timeout and was killed.
    Timeout,
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::Unavailable(e) => write!(f, "command unavailable: {e}"),
            ProcessError::Timeout => write!(f, "command timed out"),
        }
    }
}

impl std::error::Error for ProcessError {}

/// Runs `program` with `args`, capturing stdout and stderr, killing it if it
/// exceeds `timeout`.
pub fn run_with_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<Output, ProcessError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ProcessError::Unavailable)?;

    // Drain both pipes on helper threads so a chatty process cannot block
    // on a full pipe while we wait with a timeout.
    let mut stdout = child.stdout.take().expect("stdout piped");
    let stdout_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let mut stderr = child.stderr.take().expect("stderr piped");
    let stderr_reader = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stderr.read_to_string(&mut buf);
        buf
    });

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                warn!(program, "external command exceeded timeout, killing");
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProcessError::Timeout);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(ProcessError::Unavailable(e)),
        }
    };

    Ok(Output {
        success: status.success(),
        stdout: stdout_reader.join().unwrap_or_default(),
        stderr: stderr_reader.join().unwrap_or_default().trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_output_and_status() {
        let out = run_with_timeout(
            "sh",
            &["-c", "echo hi; echo err >&2; exit 3"],
            Duration::from_secs(5),
        )
        .unwrap();
        assert!(!out.success);
        assert_eq!(out.stdout, b"hi\n");
        assert_eq!(out.stderr, "err");
    }

    #[test]
    fn kills_on_timeout() {
        let started = Instant::now();
        let err =
            run_with_timeout("sh", &["-c", "sleep 5"], Duration::from_millis(200)).unwrap_err();
        assert!(matches!(err, ProcessError::Timeout));
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn missing_binary_is_unavailable() {
        let err = run_with_timeout(
            "definitely-not-a-real-binary-xyz",
            &[],
            Duration::from_secs(1),
        )
        .unwrap_err();
        assert!(matches!(err, ProcessError::Unavailable(_)));
    }
}
