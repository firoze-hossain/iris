//! crates/iris-core/src/hub.rs
//!
//! Central coordinator for Iris Camera: hardware feeds, resolution negotiation,
//! shutter triggers, video recording sessions, and media library synchronization.

use crate::capture::CaptureEngine;
use crate::config::{CameraMode, IrisConfig};
use crate::device::{DeviceRegistry, Resolution};
use crate::frame::{GridMode, VideoFrame};
use crate::gallery::{GalleryRepo, MediaItem};

#[derive(Clone, Debug)]
pub struct CameraHub {
    pub registry: DeviceRegistry,
    pub config: IrisConfig,
    pub capture: CaptureEngine,
    pub gallery: GalleryRepo,
    pub current_frame: Option<VideoFrame>,
    pub show_settings_modal: bool,
    pub status_message: Option<String>,
}

impl Default for CameraHub {
    fn default() -> Self {
        let registry = DeviceRegistry::default();
        let config = IrisConfig::default();
        let capture = CaptureEngine::default();
        let gallery = GalleryRepo::default();

        let initial_frame = Some(VideoFrame::generate_synthetic(
            config.resolution.width(),
            config.resolution.height(),
            0,
            0,
        ));

        CameraHub {
            registry,
            config,
            capture,
            gallery,
            current_frame: initial_frame,
            show_settings_modal: false,
            status_message: Some("Iris Camera ready".to_string()),
        }
    }
}

impl CameraHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances internal clock by `delta_ms` and updates live synthetic/video frame.
    pub fn tick(&mut self, delta_ms: u32, current_epoch: u64) {
        // 1. Tick countdown
        if self.capture.tick_countdown(delta_ms) {
            // Countdown finished! Execute photo snapshot
            self.execute_photo_snapshot(current_epoch);
        }

        // 2. Tick flash animation
        self.capture.tick_flash();

        // 3. Tick active recording frame counter
        if let Some(session) = self.capture.active_recording.as_mut() {
            session.frame_count += 1;
        }

        // 4. Update synthetic camera frame
        let f_idx = self.current_frame.as_ref().map(|f| f.frame_idx + 1).unwrap_or(0);
        let mut frame = VideoFrame::generate_synthetic(
            self.config.resolution.width(),
            self.config.resolution.height(),
            f_idx,
            current_epoch * 1_000_000,
        );

        if self.config.mirror_mode {
            frame.flip_horizontal();
        }

        if self.config.grid != GridMode::Off {
            frame.apply_composition_grid(self.config.grid);
        }

        self.current_frame = Some(frame);
    }

    /// Handles shutter button press (Photo snapshot or Video record toggle)
    pub fn trigger_shutter(&mut self, current_epoch: u64) -> Option<MediaItem> {
        match self.config.mode {
            CameraMode::Photo => {
                if self.capture.trigger_photo(self.config.timer) {
                    self.execute_photo_snapshot(current_epoch)
                } else {
                    self.status_message = Some(format!(
                        "Countdown armed: {}s",
                        self.config.timer.seconds()
                    ));
                    None
                }
            }
            CameraMode::Video => {
                if self.capture.is_recording() {
                    // Stop video recording
                    if let Some((filename, filepath, res, duration, size)) = self.capture.stop_video(current_epoch) {
                        let item = self.gallery.add_video(
                            &filename,
                            &filepath,
                            res,
                            size,
                            duration,
                            current_epoch,
                        );
                        self.status_message = Some(format!("Recorded video: {} ({})", filename, res.short_name()));
                        Some(item)
                    } else {
                        None
                    }
                } else {
                    // Start video recording
                    self.capture.start_video(
                        current_epoch,
                        self.config.resolution,
                        &self.config.video_directory,
                    );
                    self.status_message = Some(format!(
                        "Recording started at {}",
                        self.config.resolution.short_name()
                    ));
                    None
                }
            }
        }
    }

    fn execute_photo_snapshot(&mut self, current_epoch: u64) -> Option<MediaItem> {
        let filename = format!("IMG_{}.ppm", current_epoch);
        let filepath = format!("{}/{}", self.config.photo_directory, filename);
        let size = (self.config.resolution.width() * self.config.resolution.height() * 3) as u64;

        if let Some(frame) = self.current_frame.as_ref() {
            self.capture.last_snapshot_taken = Some(frame.clone());
        }

        let item = self.gallery.add_photo(
            &filename,
            &filepath,
            self.config.resolution,
            size,
            current_epoch,
        );

        self.status_message = Some(format!("Captured photo: {} ({})", filename, self.config.resolution.short_name()));
        Some(item)
    }

    pub fn set_resolution(&mut self, res: Resolution) {
        self.config.resolution = res;
        self.status_message = Some(format!("Resolution set to {}", res.label()));
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        if self.capture.is_recording() {
            // Cannot change mode while recording
            return;
        }
        self.config.mode = mode;
        self.status_message = Some(format!("Switched to {}", mode.label()));
    }

    pub fn toggle_mode(&mut self) {
        if self.config.mode == CameraMode::Photo {
            self.set_mode(CameraMode::Video);
        } else {
            self.set_mode(CameraMode::Photo);
        }
    }

    pub fn cycle_timer(&mut self) {
        self.config.timer = self.config.timer.cycle_next();
        self.status_message = Some(self.config.timer.label().to_string());
    }

    pub fn cycle_grid(&mut self) {
        self.config.grid = match self.config.grid {
            GridMode::Off => GridMode::RuleOfThirds,
            GridMode::RuleOfThirds => GridMode::GoldenRatio,
            GridMode::GoldenRatio => GridMode::Crosshair,
            GridMode::Crosshair => GridMode::Off,
        };
        self.status_message = Some(self.config.grid.label().to_string());
    }

    pub fn toggle_mirror(&mut self) {
        self.config.mirror_mode = !self.config.mirror_mode;
        self.status_message = Some(format!(
            "Mirror mode: {}",
            if self.config.mirror_mode { "ON" } else { "OFF" }
        ));
    }

    pub fn switch_camera(&mut self) {
        if let Some(dev) = self.registry.cycle_next_camera() {
            self.config.active_device_id = dev.id.clone();
            self.status_message = Some(format!("Switched camera to: {}", dev.name));
        }
    }

    pub fn toggle_settings(&mut self) {
        self.show_settings_modal = !self.show_settings_modal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_shutter_photo_and_video() {
        let mut hub = CameraHub::new();
        assert_eq!(hub.config.mode, CameraMode::Photo);

        // Take instant photo
        let photo = hub.trigger_shutter(1000);
        assert!(photo.is_some());
        assert_eq!(hub.gallery.items.len(), 4); // 3 initial + 1 new

        // Switch to video mode
        hub.toggle_mode();
        assert_eq!(hub.config.mode, CameraMode::Video);

        // Start video recording
        let rec_start = hub.trigger_shutter(2000);
        assert!(rec_start.is_none());
        assert!(hub.capture.is_recording());

        // Stop video recording
        let rec_stop = hub.trigger_shutter(2010);
        assert!(rec_stop.is_some());
        assert!(!hub.capture.is_recording());
        assert_eq!(rec_stop.unwrap().duration_seconds, Some(10));
    }

    #[test]
    fn test_resolution_and_camera_switch() {
        let mut hub = CameraHub::new();
        hub.set_resolution(Resolution::Res4K);
        assert_eq!(hub.config.resolution, Resolution::Res4K);

        hub.switch_camera();
        assert_eq!(hub.registry.active_device_idx, 1);
        assert_eq!(hub.config.active_device_id, "usb_cam_1");
    }
}
