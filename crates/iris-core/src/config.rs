//! crates/iris-core/src/config.rs
//!
//! Persistent user configuration, device preferences, and camera modes.

use crate::device::Resolution;
use crate::frame::GridMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    Photo,
    Video,
}

impl CameraMode {
    pub fn label(&self) -> &'static str {
        match self {
            CameraMode::Photo => "Photo Mode",
            CameraMode::Video => "Video Mode",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerDuration {
    Off,
    Delay3s,
    Delay5s,
    Delay10s,
}

impl TimerDuration {
    pub fn seconds(&self) -> u32 {
        match self {
            TimerDuration::Off => 0,
            TimerDuration::Delay3s => 3,
            TimerDuration::Delay5s => 5,
            TimerDuration::Delay10s => 10,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimerDuration::Off => "Timer: Off",
            TimerDuration::Delay3s => "Timer: 3 seconds",
            TimerDuration::Delay5s => "Timer: 5 seconds",
            TimerDuration::Delay10s => "Timer: 10 seconds",
        }
    }

    pub fn cycle_next(&self) -> Self {
        match self {
            TimerDuration::Off => TimerDuration::Delay3s,
            TimerDuration::Delay3s => TimerDuration::Delay5s,
            TimerDuration::Delay5s => TimerDuration::Delay10s,
            TimerDuration::Delay10s => TimerDuration::Off,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IrisConfig {
    pub active_device_id: String,
    pub resolution: Resolution,
    pub mode: CameraMode,
    pub mirror_mode: bool,
    pub timer: TimerDuration,
    pub grid: GridMode,
    pub flash_enabled: bool,
    pub shutter_sound: bool,
    pub photo_directory: String,
    pub video_directory: String,
}

impl Default for IrisConfig {
    fn default() -> Self {
        IrisConfig {
            active_device_id: "builtin_0".to_string(),
            resolution: Resolution::Res720p,
            mode: CameraMode::Photo,
            mirror_mode: true,
            timer: TimerDuration::Off,
            grid: GridMode::Off,
            flash_enabled: true,
            shutter_sound: true,
            photo_directory: "/home/firoze/Pictures/Camera".to_string(),
            video_directory: "/home/firoze/Videos/Camera".to_string(),
        }
    }
}

impl IrisConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
