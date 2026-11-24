//! crates/iris-ui/src/lib.rs
//!
//! Iris UI: 2D software rasterization, viewfinder scaling, and GNOME Snapshot-inspired UI.

pub mod draw;
pub mod font;
pub mod geom;
pub mod views;

pub use draw::{OwnedBufferSurface, Surface};
pub use font::{draw_char, draw_text, draw_text_font, draw_text_small, Font};
pub use geom::{Color, Point, Rect};
pub use views::IrisRenderer;

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::CameraHub;

    #[test]
    fn test_iris_ui_headless_rendering() {
        let mut hub = CameraHub::new();
        let mut surface = OwnedBufferSurface::new(800, 600);

        // 1. Initial Photo Mode rendering
        IrisRenderer::render(&mut surface, Rect::new(0, 0, 800, 600), &hub, 1000);
        assert_ne!(surface.get_pixel(400, 300).to_u32(), 0);

        // 2. Video Recording Mode rendering
        hub.toggle_mode();
        hub.trigger_shutter(1005);
        assert!(hub.capture.is_recording());
        IrisRenderer::render(&mut surface, Rect::new(0, 0, 800, 600), &hub, 1008);

        // 3. Countdown Overlay rendering
        hub.toggle_mode();
        hub.config.timer = iris_core::config::TimerDuration::Delay3s;
        hub.trigger_shutter(1010);
        IrisRenderer::render(&mut surface, Rect::new(0, 0, 800, 600), &hub, 1011);

        // 4. Settings modal rendering
        hub.toggle_settings();
        IrisRenderer::render(&mut surface, Rect::new(0, 0, 800, 600), &hub, 1012);
    }
}
