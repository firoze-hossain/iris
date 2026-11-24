//! crates/iris-ui/src/draw.rs
//!
//! Software rasterizer for Iris Camera UI: viewport letterboxing, circular shutter buttons,
//! flash animation blending, and countdown overlays.

use iris_core::config::CameraMode;
use iris_core::frame::VideoFrame;
use crate::geom::{Color, Rect};

pub trait Surface {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
    fn get_pixel(&self, x: i32, y: i32) -> Color;

    fn fill_rect(&mut self, r: Rect, color: Color) {
        let x0 = r.x.max(0);
        let y0 = r.y.max(0);
        let x1 = (r.x + r.w).min(self.width());
        let y1 = (r.y + r.h).min(self.height());
        for y in y0..y1 {
            for x in x0..x1 {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn draw_rect(&mut self, r: Rect, color: Color) {
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        self.draw_line(r.x, r.y, r.x + r.w - 1, r.y, color);
        self.draw_line(r.x, r.y + r.h - 1, r.x + r.w - 1, r.y + r.h - 1, color);
        self.draw_line(r.x, r.y, r.x, r.y + r.h - 1, color);
        self.draw_line(r.x + r.w - 1, r.y, r.x + r.w - 1, r.y + r.h - 1, color);
    }

    fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.put_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let mut x = r;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.put_pixel(cx + x, cy + y, color);
            self.put_pixel(cx + y, cy + x, color);
            self.put_pixel(cx - y, cy + x, color);
            self.put_pixel(cx - x, cy + y, color);
            self.put_pixel(cx - x, cy - y, color);
            self.put_pixel(cx - y, cy - x, color);
            self.put_pixel(cx + y, cy - x, color);
            self.put_pixel(cx + x, cy - y, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let r2 = r * r;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r2 {
                    self.put_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }

    fn fill_rounded_rect(&mut self, r: Rect, radius: i32, color: Color) {
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        let rad = radius.min(r.w / 2).min(r.h / 2);
        if rad <= 0 {
            self.fill_rect(r, color);
            return;
        }

        self.fill_rect(Rect::new(r.x + rad, r.y, r.w - 2 * rad, r.h), color);
        self.fill_rect(Rect::new(r.x, r.y + rad, rad, r.h - 2 * rad), color);
        self.fill_rect(Rect::new(r.x + r.w - rad, r.y + rad, rad, r.h - 2 * rad), color);

        self.fill_circle(r.x + rad, r.y + rad, rad, color);
        self.fill_circle(r.x + r.w - rad - 1, r.y + rad, rad, color);
        self.fill_circle(r.x + rad, r.y + r.h - rad - 1, rad, color);
        self.fill_circle(r.x + r.w - rad - 1, r.y + r.h - rad - 1, rad, color);
    }

    fn draw_rounded_rect(&mut self, r: Rect, radius: i32, color: Color) {
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        let rad = radius.min(r.w / 2).min(r.h / 2);
        if rad <= 0 {
            self.draw_rect(r, color);
            return;
        }

        self.draw_line(r.x + rad, r.y, r.x + r.w - rad - 1, r.y, color);
        self.draw_line(r.x + rad, r.y + r.h - 1, r.x + r.w - rad - 1, r.y + r.h - 1, color);
        self.draw_line(r.x, r.y + rad, r.x, r.y + r.h - rad - 1, color);
        self.draw_line(r.x + r.w - 1, r.y + rad, r.x + r.w - 1, r.y + r.h - rad - 1, color);

        let mut x = rad;
        let mut y = 0;
        let mut err = 0;

        let tl = (r.x + rad, r.y + rad);
        let tr = (r.x + r.w - rad - 1, r.y + rad);
        let bl = (r.x + rad, r.y + r.h - rad - 1);
        let br = (r.x + r.w - rad - 1, r.y + r.h - rad - 1);

        while x >= y {
            self.put_pixel(tl.0 - x, tl.1 - y, color);
            self.put_pixel(tl.0 - y, tl.1 - x, color);
            self.put_pixel(tr.0 + x, tr.1 - y, color);
            self.put_pixel(tr.0 + y, tr.1 - x, color);
            self.put_pixel(bl.0 - x, bl.1 + y, color);
            self.put_pixel(bl.0 - y, bl.1 + x, color);
            self.put_pixel(br.0 + x, br.1 + y, color);
            self.put_pixel(br.0 + y, br.1 + x, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// Renders video frame scaled into destination rect maintaining aspect ratio
    fn draw_video_frame(&mut self, frame: &VideoFrame, dest: Rect) {
        if dest.w <= 0 || dest.h <= 0 || frame.width == 0 || frame.height == 0 {
            return;
        }

        // Calculate aspect-ratio preserving fit (letterbox)
        let frame_aspect = frame.width as f32 / frame.height as f32;
        let dest_aspect = dest.w as f32 / dest.h as f32;

        let (fit_w, fit_h) = if dest_aspect > frame_aspect {
            // Limited by height
            let h = dest.h;
            let w = (h as f32 * frame_aspect) as i32;
            (w, h)
        } else {
            // Limited by width
            let w = dest.w;
            let h = (w as f32 / frame_aspect) as i32;
            (w, h)
        };

        let off_x = dest.x + (dest.w - fit_w) / 2;
        let off_y = dest.y + (dest.h - fit_h) / 2;

        // Fill background letterbox borders with dark black
        self.fill_rect(dest, Color::VIEWPORT_BG);

        // Blit scaled frame pixels
        let x_ratio = ((frame.width as u64) << 16) / fit_w as u64;
        let y_ratio = ((frame.height as u64) << 16) / fit_h as u64;

        for dy in 0..fit_h {
            let sy = ((dy as u64 * y_ratio) >> 16) as usize;
            let src_row = sy * frame.width as usize;

            for dx in 0..fit_w {
                let sx = ((dx as u64 * x_ratio) >> 16) as usize;
                let s_idx = src_row + sx;
                if s_idx < frame.pixels.len() {
                    let col = Color::from_u32(frame.pixels[s_idx]);
                    self.put_pixel(off_x + dx, off_y + dy, col);
                }
            }
        }
    }

    /// Renders the iconic Ubuntu Camera shutter / record button:
    /// - White outer ring (radius 34)
    /// - In Photo Mode: Solid white inner circle (radius 26)
    /// - In Video Idle Mode: Solid red inner circle (radius 26)
    /// - In Video Recording Mode: Red rounded square (18x18)
    fn draw_shutter_button(&mut self, cx: i32, cy: i32, mode: CameraMode, is_recording: bool) {
        // Outer white ring
        self.draw_circle(cx, cy, 34, Color::SHUTTER_RING);
        self.draw_circle(cx, cy, 33, Color::SHUTTER_RING);

        if is_recording {
            // Stop button: red rounded square inside white ring
            let sq_size = 22;
            let sq_rect = Rect::new(cx - sq_size / 2, cy - sq_size / 2, sq_size, sq_size);
            self.fill_rounded_rect(sq_rect, 4, Color::SHUTTER_RECORDING_STOP);
        } else {
            match mode {
                CameraMode::Photo => {
                    self.fill_circle(cx, cy, 27, Color::SHUTTER_PHOTO_INNER);
                }
                CameraMode::Video => {
                    self.fill_circle(cx, cy, 27, Color::SHUTTER_VIDEO_INNER);
                }
            }
        }
    }
}

/// Owned pixel buffer for headless tests and offscreen rendering.
#[derive(Clone)]
pub struct OwnedBufferSurface {
    pub pixels: Vec<u32>,
    pub width: i32,
    pub height: i32,
}

impl OwnedBufferSurface {
    pub fn new(w: i32, h: i32) -> Self {
        OwnedBufferSurface {
            pixels: vec![Color::VIEWPORT_BG.to_u32(); (w * h) as usize],
            width: w,
            height: h,
        }
    }
}

impl Surface for OwnedBufferSurface {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.pixels.len() {
            if color.a == 255 {
                self.pixels[idx] = color.to_u32();
            } else if color.a > 0 {
                let bg = Color::from_u32(self.pixels[idx]);
                self.pixels[idx] = color.blend_over(bg).to_u32();
            }
        }
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Color::rgba(0, 0, 0, 0);
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.pixels.len() {
            Color::from_u32(self.pixels[idx])
        } else {
            Color::rgba(0, 0, 0, 0)
        }
    }
}
