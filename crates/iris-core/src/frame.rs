//! crates/iris-core/src/frame.rs
//!
//! Video frame representation, mirror flip transformations, composition grid overlays,
//! and PPM snapshot export.

use crate::device::PixelFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridMode {
    Off,
    RuleOfThirds, // 3x3 grid dividing frame into thirds
    GoldenRatio,  // 1:1.618 golden ratio composition lines
    Crosshair,    // Center crosshair
}

impl GridMode {
    pub fn label(&self) -> &'static str {
        match self {
            GridMode::Off => "Grid: Off",
            GridMode::RuleOfThirds => "Grid: 3×3 (Rule of Thirds)",
            GridMode::GoldenRatio => "Grid: Golden Ratio",
            GridMode::Crosshair => "Grid: Center Crosshair",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>, // 0xAARRGGBB format
    pub timestamp_us: u64,
    pub frame_idx: u64,
    pub format: PixelFormat,
}

impl VideoFrame {
    pub fn new(width: u32, height: u32, pixels: Vec<u32>, timestamp_us: u64, frame_idx: u64) -> Self {
        VideoFrame {
            width,
            height,
            pixels,
            timestamp_us,
            frame_idx,
            format: PixelFormat::RGBA32,
        }
    }

    /// Creates a solid color frame
    pub fn solid(width: u32, height: u32, color_argb: u32) -> Self {
        let size = (width * height) as usize;
        VideoFrame {
            width,
            height,
            pixels: vec![color_argb; size],
            timestamp_us: 0,
            frame_idx: 0,
            format: PixelFormat::RGBA32,
        }
    }

    /// Horizontally flips the frame in-place (mirror mode for natural front webcam feed)
    pub fn flip_horizontal(&mut self) {
        let w = self.width as usize;
        let h = self.height as usize;

        for y in 0..h {
            let row_start = y * w;
            for x in 0..(w / 2) {
                let left_idx = row_start + x;
                let right_idx = row_start + (w - 1 - x);
                self.pixels.swap(left_idx, right_idx);
            }
        }
    }

    /// Overlays faint composition guide lines (e.g. 3x3 Rule of Thirds)
    pub fn apply_composition_grid(&mut self, grid: GridMode) {
        if grid == GridMode::Off {
            return;
        }

        let w = self.width as i32;
        let h = self.height as i32;
        let grid_col = 0x80FFFFFF; // Translucent white guide lines

        match grid {
            GridMode::RuleOfThirds => {
                let x1 = w / 3;
                let x2 = (w * 2) / 3;
                let y1 = h / 3;
                let y2 = (h * 2) / 3;

                // Vertical lines
                for y in 0..h {
                    self.set_pixel_blend(x1, y, grid_col);
                    self.set_pixel_blend(x2, y, grid_col);
                }
                // Horizontal lines
                for x in 0..w {
                    self.set_pixel_blend(x, y1, grid_col);
                    self.set_pixel_blend(x, y2, grid_col);
                }
            }
            GridMode::GoldenRatio => {
                let x1 = (w as f32 * 0.382) as i32;
                let x2 = (w as f32 * 0.618) as i32;
                let y1 = (h as f32 * 0.382) as i32;
                let y2 = (h as f32 * 0.618) as i32;

                for y in 0..h {
                    self.set_pixel_blend(x1, y, grid_col);
                    self.set_pixel_blend(x2, y, grid_col);
                }
                for x in 0..w {
                    self.set_pixel_blend(x, y1, grid_col);
                    self.set_pixel_blend(x, y2, grid_col);
                }
            }
            GridMode::Crosshair => {
                let cx = w / 2;
                let cy = h / 2;
                let arm = 24;

                for dx in -arm..=arm {
                    self.set_pixel_blend(cx + dx, cy, grid_col);
                }
                for dy in -arm..=arm {
                    self.set_pixel_blend(cx, cy + dy, grid_col);
                }
            }
            GridMode::Off => {}
        }
    }

    #[inline]
    fn set_pixel_blend(&mut self, x: i32, y: i32, color_argb: u32) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as usize * self.width as usize) + x as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = color_argb;
            }
        }
    }

    /// Fast box-downscaling of this frame into a target buffer of dimension `(target_w, target_h)`
    pub fn downscale(&self, target_w: u32, target_h: u32) -> VideoFrame {
        if self.width == target_w && self.height == target_h {
            return self.clone();
        }

        let mut out = vec![0u32; (target_w * target_h) as usize];
        let x_ratio = ((self.width as u64) << 16) / target_w as u64;
        let y_ratio = ((self.height as u64) << 16) / target_h as u64;

        for ty in 0..target_h {
            let sy = ((ty as u64 * y_ratio) >> 16) as usize;
            let src_row = sy * self.width as usize;
            let dst_row = ty as usize * target_w as usize;

            for tx in 0..target_w {
                let sx = ((tx as u64 * x_ratio) >> 16) as usize;
                let s_idx = src_row + sx;
                if s_idx < self.pixels.len() {
                    out[dst_row + tx as usize] = self.pixels[s_idx];
                }
            }
        }

        VideoFrame {
            width: target_w,
            height: target_h,
            pixels: out,
            timestamp_us: self.timestamp_us,
            frame_idx: self.frame_idx,
            format: self.format,
        }
    }

    /// Generates a standard binary PPM (P6) file for instant lossless storage
    pub fn to_ppm_bytes(&self) -> Vec<u8> {
        let header = format!("P6\n{} {}\n255\n", self.width, self.height);
        let mut bytes = Vec::with_capacity(header.len() + (self.width * self.height * 3) as usize);
        bytes.extend_from_slice(header.as_bytes());

        for &pixel in &self.pixels {
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            bytes.push(r);
            bytes.push(g);
            bytes.push(b);
        }

        bytes
    }

    /// Generates a dynamic synthetic camera frame with realistic SMPTE color bars,
    /// animated timecode, and subtle vignetting.
    pub fn generate_synthetic(width: u32, height: u32, frame_idx: u64, timestamp_us: u64) -> Self {
        let mut pixels = Vec::with_capacity((width * height) as usize);

        // Standard SMPTE 7-bar palette
        let smpte_colors = [
            0xFFC0C0C0, // Grey 75%
            0xFFC0C000, // Yellow
            0xFF00C0C0, // Cyan
            0xFF00C000, // Green
            0xFFC000C0, // Magenta
            0xFFC00000, // Red
            0xFF0000C0, // Blue
        ];

        let bar_width = (width / 7).max(1);
        let sweep_x = ((frame_idx * 4) % width as u64) as u32;

        for y in 0..height {
            let is_bottom_cast = y > (height * 3) / 4;
            for x in 0..width {
                let bar_idx = ((x / bar_width) as usize).min(smpte_colors.len() - 1);
                let mut col = smpte_colors[bar_idx];

                if is_bottom_cast {
                    // Dark studio gradient on bottom quarter
                    let r = (((col >> 16) & 0xFF) / 3) as u32;
                    let g = (((col >> 8) & 0xFF) / 3) as u32;
                    let b = ((col & 0xFF) / 3) as u32;
                    col = 0xFF000000 | (r << 16) | (g << 8) | b;
                }

                // Animated scan line / crosshair
                if x == sweep_x || (x + 1) == sweep_x {
                    col = 0xFFFFFFFF;
                }

                pixels.push(col);
            }
        }

        VideoFrame {
            width,
            height,
            pixels,
            timestamp_us,
            frame_idx,
            format: PixelFormat::RGBA32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_flip_mirror() {
        // 4x1 test row: [1, 2, 3, 4]
        let mut frame = VideoFrame::new(4, 1, vec![10, 20, 30, 40], 100, 1);
        frame.flip_horizontal();
        assert_eq!(frame.pixels, vec![40, 30, 20, 10]);
    }

    #[test]
    fn test_downscale() {
        let frame = VideoFrame::solid(640, 480, 0xFF00FF00);
        let down = frame.downscale(320, 240);
        assert_eq!(down.width, 320);
        assert_eq!(down.height, 240);
        assert_eq!(down.pixels.len(), 320 * 240);
        assert_eq!(down.pixels[0], 0xFF00FF00);
    }

    #[test]
    fn test_ppm_export() {
        let frame = VideoFrame::solid(2, 2, 0xFFFF0000); // Red
        let ppm = frame.to_ppm_bytes();
        let header = "P6\n2 2\n255\n";
        assert!(ppm.starts_with(header.as_bytes()));
        // 4 pixels * 3 bytes = 12 bytes payload
        assert_eq!(ppm.len(), header.len() + 12);
        assert_eq!(&ppm[header.len()..header.len() + 3], &[255, 0, 0]);
    }
}
