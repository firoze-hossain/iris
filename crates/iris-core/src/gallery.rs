//! crates/iris-core/src/gallery.rs
//!
//! Media library and recent captures gallery repository for Iris Camera.

use crate::device::Resolution;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Photo,
    Video,
}

impl MediaKind {
    pub fn extension(&self) -> &'static str {
        match self {
            MediaKind::Photo => "ppm", // Lossless Netpbm PPM
            MediaKind::Video => "raw", // Raw video stream frames
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaItem {
    pub id: u64,
    pub filename: String,
    pub filepath: String,
    pub kind: MediaKind,
    pub resolution: Resolution,
    pub timestamp_epoch: u64,
    pub size_bytes: u64,
    pub duration_seconds: Option<u32>, // Only for video clips
}

impl MediaItem {
    pub fn new_photo(
        id: u64,
        filename: &str,
        filepath: &str,
        resolution: Resolution,
        timestamp: u64,
        size_bytes: u64,
    ) -> Self {
        MediaItem {
            id,
            filename: filename.to_string(),
            filepath: filepath.to_string(),
            kind: MediaKind::Photo,
            resolution,
            timestamp_epoch: timestamp,
            size_bytes,
            duration_seconds: None,
        }
    }

    pub fn new_video(
        id: u64,
        filename: &str,
        filepath: &str,
        resolution: Resolution,
        timestamp: u64,
        size_bytes: u64,
        duration_seconds: u32,
    ) -> Self {
        MediaItem {
            id,
            filename: filename.to_string(),
            filepath: filepath.to_string(),
            kind: MediaKind::Video,
            resolution,
            timestamp_epoch: timestamp,
            size_bytes,
            duration_seconds: Some(duration_seconds),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GalleryRepo {
    pub items: Vec<MediaItem>,
    pub next_id: u64,
}

impl Default for GalleryRepo {
    fn default() -> Self {
        let mut repo = GalleryRepo {
            items: Vec::new(),
            next_id: 1,
        };
        repo.populate_initial_captures();
        repo
    }
}

impl GalleryRepo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn populate_initial_captures(&mut self) {
        let base_time = 1732435000;

        self.items.push(MediaItem::new_photo(
            1,
            "IMG_20261122_183602.ppm",
            "/home/firoze/Pictures/Camera/IMG_20261122_183602.ppm",
            Resolution::Res720p,
            base_time - 3600,
            2_764_800,
        ));

        self.items.push(MediaItem::new_photo(
            2,
            "IMG_20261123_091522.ppm",
            "/home/firoze/Pictures/Camera/IMG_20261123_091522.ppm",
            Resolution::Res1080p,
            base_time - 1800,
            6_220_800,
        ));

        self.items.push(MediaItem::new_video(
            3,
            "VID_20261124_140210.raw",
            "/home/firoze/Videos/Camera/VID_20261124_140210.raw",
            Resolution::Res1080p,
            base_time - 600,
            45_800_000,
            15, // 15 seconds
        ));

        self.next_id = 4;
    }

    pub fn add_photo(&mut self, filename: &str, filepath: &str, resolution: Resolution, size: u64, timestamp: u64) -> MediaItem {
        let item = MediaItem::new_photo(self.next_id, filename, filepath, resolution, timestamp, size);
        self.next_id += 1;
        self.items.insert(0, item.clone());
        item
    }

    pub fn add_video(&mut self, filename: &str, filepath: &str, resolution: Resolution, size: u64, duration: u32, timestamp: u64) -> MediaItem {
        let item = MediaItem::new_video(self.next_id, filename, filepath, resolution, timestamp, size, duration);
        self.next_id += 1;
        self.items.insert(0, item.clone());
        item
    }

    pub fn latest_item(&self) -> Option<&MediaItem> {
        self.items.first()
    }

    pub fn delete_item(&mut self, id: u64) -> bool {
        if let Some(pos) = self.items.iter().position(|i| i.id == id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_addition_and_ordering() {
        let mut gallery = GalleryRepo::new();
        let initial_count = gallery.items.len();

        let new_item = gallery.add_photo("NEW_PHOTO.ppm", "/tmp/NEW_PHOTO.ppm", Resolution::Res4K, 1000, 2000);
        assert_eq!(gallery.items.len(), initial_count + 1);
        assert_eq!(gallery.latest_item().unwrap().id, new_item.id);
        assert_eq!(gallery.latest_item().unwrap().resolution, Resolution::Res4K);
    }
}
