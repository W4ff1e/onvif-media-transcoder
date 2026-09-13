//! Describes the video stream that is exposed to ONVIF clients.
//!
//! Real values are probed from the RTSP stream with ffprobe in the
//! background; until that succeeds (or if ffprobe is unavailable) sensible
//! defaults are reported.

use crate::onvif::process::run_with_timeout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, info, warn};

/// How long one ffprobe attempt may take.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
/// Delay between failed probe attempts.
pub const PROBE_RETRY_INTERVAL: Duration = Duration::from_secs(10);

/// Video encoding as reported to ONVIF clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoEncoding {
    H264,
    H265,
    Jpeg,
    Mpeg4,
}

impl VideoEncoding {
    /// The `tt:VideoEncoding` enumeration value.
    pub fn as_onvif(self) -> &'static str {
        match self {
            VideoEncoding::H264 => "H264",
            VideoEncoding::H265 => "H265",
            VideoEncoding::Jpeg => "JPEG",
            VideoEncoding::Mpeg4 => "MPEG4",
        }
    }

    fn from_ffprobe(codec: &str) -> Option<Self> {
        match codec {
            "h264" => Some(VideoEncoding::H264),
            "hevc" | "h265" => Some(VideoEncoding::H265),
            "mjpeg" => Some(VideoEncoding::Jpeg),
            "mpeg4" => Some(VideoEncoding::Mpeg4),
            _ => None,
        }
    }
}

/// Properties of the exposed video stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInfo {
    pub encoding: VideoEncoding,
    pub width: u32,
    pub height: u32,
    pub framerate: u32,
    /// Bitrate limit in kbit/s.
    pub bitrate_kbps: u32,
    /// H.264 profile (Baseline, Main, Extended, High), if known.
    pub h264_profile: Option<String>,
    /// Whether these values came from a probe of the live stream.
    pub probed: bool,
}

impl Default for StreamInfo {
    fn default() -> Self {
        Self {
            encoding: VideoEncoding::H264,
            width: 1920,
            height: 1080,
            framerate: 25,
            bitrate_kbps: 4000,
            h264_profile: Some("Main".to_string()),
            probed: false,
        }
    }
}

impl StreamInfo {
    /// Parses `key=value` lines as produced by
    /// `ffprobe -show_entries stream=... -of default=noprint_wrappers=1`.
    pub fn from_ffprobe_output(output: &str) -> Option<Self> {
        let mut info = StreamInfo::default();
        let mut codec = None;
        let mut width = None;
        let mut height = None;
        let mut framerate = None;
        let mut bitrate = None;

        for line in output.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "codec_name" => codec = VideoEncoding::from_ffprobe(value),
                "width" => width = value.parse::<u32>().ok(),
                "height" => height = value.parse::<u32>().ok(),
                "r_frame_rate" | "avg_frame_rate" => {
                    if let Some(fps) = parse_rate(value) {
                        // Prefer avg_frame_rate when both are present and sane.
                        if key.trim() == "avg_frame_rate" || framerate.is_none() {
                            framerate = Some(fps);
                        }
                    }
                }
                "bit_rate" => bitrate = value.parse::<u64>().ok().map(|b| (b / 1000) as u32),
                "profile" => {
                    info.h264_profile = normalise_h264_profile(value);
                }
                _ => {}
            }
        }

        let (Some(codec), Some(width), Some(height)) = (codec, width, height) else {
            return None;
        };
        if width == 0 || height == 0 {
            return None;
        }

        info.encoding = codec;
        info.width = width;
        info.height = height;
        if let Some(fps) = framerate.filter(|f| *f > 0) {
            info.framerate = fps;
        }
        if let Some(kbps) = bitrate.filter(|b| *b > 0) {
            info.bitrate_kbps = kbps;
        }
        if codec != VideoEncoding::H264 {
            info.h264_profile = None;
        }
        info.probed = true;
        Some(info)
    }
}

/// Parses an ffprobe rational such as `30000/1001` or `25/1` into rounded fps.
fn parse_rate(value: &str) -> Option<u32> {
    let (num, den) = value.split_once('/').unwrap_or((value, "1"));
    let num: f64 = num.trim().parse().ok()?;
    let den: f64 = den.trim().parse().ok()?;
    if den == 0.0 || num <= 0.0 {
        return None;
    }
    Some((num / den).round() as u32)
}

/// Maps ffprobe H.264 profile names onto the ONVIF `tt:H264Profile` enum.
fn normalise_h264_profile(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let profile = if lower.contains("baseline") {
        "Baseline"
    } else if lower.contains("main") {
        "Main"
    } else if lower.contains("extended") {
        "Extended"
    } else if lower.contains("high") {
        "High"
    } else {
        return None;
    };
    Some(profile.to_string())
}

/// Shared, lazily probed stream description.
#[derive(Debug, Default)]
pub struct StreamProbe {
    info: RwLock<StreamInfo>,
}

impl StreamProbe {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Current best knowledge of the stream.
    pub fn current(&self) -> StreamInfo {
        self.info.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Replaces the stream description (used by tests and the probe thread).
    pub fn set(&self, info: StreamInfo) {
        *self.info.write().unwrap_or_else(|e| e.into_inner()) = info;
    }

    /// Runs one ffprobe attempt against `rtsp_url`.
    pub fn probe_once(rtsp_url: &str) -> Result<StreamInfo, String> {
        let output = run_with_timeout(
            "ffprobe",
            &[
                "-v",
                "error",
                "-rtsp_transport",
                "tcp",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=codec_name,width,height,r_frame_rate,avg_frame_rate,profile,bit_rate",
                "-of",
                "default=noprint_wrappers=1",
                rtsp_url,
            ],
            PROBE_TIMEOUT,
        )
        .map_err(|e| e.to_string())?;
        if !output.success {
            return Err(if output.stderr.is_empty() {
                "ffprobe failed".to_string()
            } else {
                output.stderr
            });
        }
        StreamInfo::from_ffprobe_output(&String::from_utf8_lossy(&output.stdout))
            .ok_or_else(|| "ffprobe output did not describe a video stream".to_string())
    }

    /// Spawns a thread that probes the stream until it succeeds or `stop`
    /// is set, updating the shared description on success.
    pub fn start_background_probe(self: &Arc<Self>, rtsp_url: String, stop: Arc<AtomicBool>) {
        let probe = Arc::clone(self);
        let spawned = std::thread::Builder::new()
            .name("stream-probe".to_string())
            .spawn(move || {
                let mut attempt = 0u32;
                while !stop.load(Ordering::Relaxed) {
                    attempt += 1;
                    match StreamProbe::probe_once(&rtsp_url) {
                        Ok(info) => {
                            info!(
                                encoding = info.encoding.as_onvif(),
                                width = info.width,
                                height = info.height,
                                framerate = info.framerate,
                                bitrate_kbps = info.bitrate_kbps,
                                "stream probed"
                            );
                            probe.set(info);
                            return;
                        }
                        Err(e) => {
                            if attempt == 1 || attempt.is_multiple_of(6) {
                                warn!(attempt, error = %e, "stream probe failed, will retry");
                            } else {
                                debug!(attempt, error = %e, "stream probe failed");
                            }
                        }
                    }
                    // Sleep in small steps so shutdown stays responsive.
                    let mut slept = Duration::ZERO;
                    while slept < PROBE_RETRY_INTERVAL && !stop.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_millis(250));
                        slept += Duration::from_millis(250);
                    }
                }
            });
        if let Err(e) = spawned {
            warn!(error = %e, "could not start stream probe thread");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ffprobe_output() {
        let out = "codec_name=h264\nprofile=High\nwidth=1280\nheight=720\nr_frame_rate=30000/1001\navg_frame_rate=30000/1001\nbit_rate=2500000\n";
        let info = StreamInfo::from_ffprobe_output(out).unwrap();
        assert_eq!(info.encoding, VideoEncoding::H264);
        assert_eq!((info.width, info.height), (1280, 720));
        assert_eq!(info.framerate, 30);
        assert_eq!(info.bitrate_kbps, 2500);
        assert_eq!(info.h264_profile.as_deref(), Some("High"));
        assert!(info.probed);
    }

    #[test]
    fn falls_back_when_fields_are_missing_or_na() {
        let out = "codec_name=hevc\nprofile=Main\nwidth=3840\nheight=2160\nr_frame_rate=15/1\navg_frame_rate=0/0\nbit_rate=N/A\n";
        let info = StreamInfo::from_ffprobe_output(out).unwrap();
        assert_eq!(info.encoding, VideoEncoding::H265);
        assert_eq!(info.framerate, 15);
        assert_eq!(info.bitrate_kbps, StreamInfo::default().bitrate_kbps);
        assert_eq!(info.h264_profile, None);

        assert!(StreamInfo::from_ffprobe_output("codec_name=h264\n").is_none());
        assert!(StreamInfo::from_ffprobe_output("codec_name=av1\nwidth=1\nheight=1\n").is_none());
        assert!(StreamInfo::from_ffprobe_output("").is_none());
    }

    #[test]
    fn normalises_profiles_and_rates() {
        assert_eq!(
            normalise_h264_profile("Constrained Baseline").as_deref(),
            Some("Baseline")
        );
        assert_eq!(
            normalise_h264_profile("High 4:4:4 Predictive").as_deref(),
            Some("High")
        );
        assert_eq!(normalise_h264_profile("unknown"), None);
        assert_eq!(parse_rate("25/1"), Some(25));
        assert_eq!(parse_rate("24000/1001"), Some(24));
        assert_eq!(parse_rate("0/0"), None);
        assert_eq!(parse_rate("x"), None);
    }

    #[test]
    fn probe_defaults_until_set() {
        let probe = StreamProbe::new();
        assert!(!probe.current().probed);
        probe.set(StreamInfo {
            width: 640,
            ..StreamInfo::default()
        });
        assert_eq!(probe.current().width, 640);
    }
}
