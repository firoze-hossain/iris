//! crates/iris-core/src/capture.rs
//!
//! Photo snapshot capture engine, countdown timer coordination,
//! and live video recording session state.

use crate::config::TimerDuration;
use crate::device::Resolution;
use crate::frame::VideoFrame;

#[derive(Clone, Debug)]
pub struct VideoRecordingSession {
    pub is_recording: bool,
    pub is_paused: bool,
    pub start_time_epoch: u64,
    pub frame_count: u64,
    pub resolution: Resolution,
    pub output_filename: String,
    pub output_filepath: String,
}

impl VideoRecordingSession {
    pub fn new(start_epoch: u64, resolution: Resolution, directory: &str) -> Self {
        let filename = format!("VID_{}.raw", start_epoch);
        let filepath = format!("{}/{}", directory, filename);
        VideoRecordingSession {
            is_recording: true,
            is_paused: false,
            start_time_epoch: start_epoch,
            frame_count: 0,
            resolution,
            output_filename: filename,
            output_filepath: filepath,
        }
    }

    pub fn elapsed_seconds(&self, current_epoch: u64) -> u32 {
        if current_epoch >= self.start_time_epoch {
            (current_epoch - self.start_time_epoch) as u32
        } else {
            0
        }
    }

    pub fn format_duration(seconds: u32) -> String {
        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        let s = seconds % 60;
        if h > 0 {
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else {
            format!("{:02}:{:02}", m, s)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CaptureEngine {
    pub countdown_remaining_ms: Option<u32>, // In milliseconds
    pub flash_remaining_frames: u32,
    pub active_recording: Option<VideoRecordingSession>,
    pub last_snapshot_taken: Option<VideoFrame>,
}

impl CaptureEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Triggers shutter action based on configured timer.
    /// Returns true if instant capture occurred, false if countdown armed or in progress.
    pub fn trigger_photo(&mut self, timer: TimerDuration) -> bool {
        let delay_sec = timer.seconds();
        if delay_sec > 0 {
            self.countdown_remaining_ms = Some(delay_sec * 1000);
            false
        } else {
            self.arm_flash();
            true
        }
    }

    /// Cancels active countdown if user clicks again during countdown
    pub fn cancel_countdown(&mut self) {
        self.countdown_remaining_ms = None;
    }

    /// Advances countdown clock by `delta_ms`. Returns true if countdown reached 0 (time to snap photo).
    pub fn tick_countdown(&mut self, delta_ms: u32) -> bool {
        if let Some(mut remaining) = self.countdown_remaining_ms {
            if remaining <= delta_ms {
                self.countdown_remaining_ms = None;
                self.arm_flash();
                true
            } else {
                remaining -= delta_ms;
                self.countdown_remaining_ms = Some(remaining);
                false
            }
        } else {
            false
        }
    }

    pub fn arm_flash(&mut self) {
        self.flash_remaining_frames = 4; // 4 frames of screen flash animation
    }

    pub fn tick_flash(&mut self) {
        if self.flash_remaining_frames > 0 {
            self.flash_remaining_frames -= 1;
        }
    }

    pub fn is_flashing(&self) -> bool {
        self.flash_remaining_frames > 0
    }

    pub fn start_video(&mut self, start_epoch: u64, resolution: Resolution, dir: &str) {
        self.active_recording = Some(VideoRecordingSession::new(start_epoch, resolution, dir));
    }

    pub fn stop_video(&mut self, current_epoch: u64) -> Option<(String, String, Resolution, u32, u64)> {
        if let Some(session) = self.active_recording.take() {
            let duration = session.elapsed_seconds(current_epoch).max(1);
            let estimated_size = session.frame_count * (session.resolution.width() * session.resolution.height() * 3) as u64;
            Some((
                session.output_filename,
                session.output_filepath,
                session.resolution,
                duration,
                estimated_size,
            ))
        } else {
            None
        }
    }

    pub fn is_recording(&self) -> bool {
        self.active_recording.as_ref().map(|s| s.is_recording).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_countdown_timer_progression() {
        let mut engine = CaptureEngine::new();
        assert!(!engine.trigger_photo(TimerDuration::Delay3s));
        assert_eq!(engine.countdown_remaining_ms, Some(3000));

        assert!(!engine.tick_countdown(1000));
        assert_eq!(engine.countdown_remaining_ms, Some(2000));

        assert!(!engine.tick_countdown(1500));
        assert_eq!(engine.countdown_remaining_ms, Some(500));

        // Final tick fires shutter
        assert!(engine.tick_countdown(600));
        assert_eq!(engine.countdown_remaining_ms, None);
        assert!(engine.is_flashing());
    }

    #[test]
    fn test_video_recording_lifecycle() {
        let mut engine = CaptureEngine::new();
        assert!(!engine.is_recording());

        engine.start_video(1000, Resolution::Res1080p, "/tmp");
        assert!(engine.is_recording());

        let result = engine.stop_video(1015);
        assert!(result.is_some());
        let (filename, _filepath, res, duration, _size) = result.unwrap();
        assert_eq!(filename, "VID_1000.raw");
        assert_eq!(res, Resolution::Res1080p);
        assert_eq!(duration, 15);
        assert!(!engine.is_recording());
    }
}
