//! crates/iris-core/src/lib.rs
//!
//! Iris Core: Sovereign Camera & Vision Engine for NovaOS
//!
//! Provides device discovery, resolution negotiation, video frame manipulation,
//! photo snapshot triggers, countdown coordination, and video recording sessions.

pub mod capture;
pub mod config;
pub mod device;
pub mod frame;
pub mod gallery;
pub mod hub;

pub use capture::{CaptureEngine, VideoRecordingSession};
pub use config::{CameraMode, IrisConfig, TimerDuration};
pub use device::{CameraDevice, DeviceKind, DeviceRegistry, PixelFormat, Resolution};
pub use frame::{GridMode, VideoFrame};
pub use gallery::{GalleryRepo, MediaItem, MediaKind};
pub use hub::CameraHub;
