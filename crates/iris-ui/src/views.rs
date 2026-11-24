//! crates/iris-ui/src/views.rs
//!
//! View layout, components, and event hit-testing for Iris Camera:
//! - Viewfinder frame presentation with letterboxing and composition guides
//! - Top control bar: Camera device switcher, resolution badge, recording status
//! - Bottom floating bar: Countdown timer, iconic circular shutter button, Photo/Video mode pill
//! - Settings modal dialog for camera selection, resolutions, grid, and selfie mirror mode.

use iris_core::capture::VideoRecordingSession;
use iris_core::config::{CameraMode, TimerDuration};
use iris_core::hub::CameraHub;
use crate::draw::Surface;
use crate::font::{draw_char, draw_text, draw_text_font, draw_text_small, Font};
use crate::geom::{Color, Rect};

pub struct IrisRenderer;

impl IrisRenderer {
    /// Renders the complete Iris Camera UI onto the surface.
    pub fn render<S: Surface + ?Sized>(s: &mut S, bounds: Rect, hub: &CameraHub, current_epoch: u64) {
        // 1. Draw Viewfinder (Live Camera Feed)
        let bottom_bar_height = 80;
        let viewport_rect = Rect::new(bounds.x, bounds.y, bounds.w, bounds.h - bottom_bar_height);

        if let Some(frame) = hub.current_frame.as_ref() {
            s.draw_video_frame(frame, viewport_rect);
        } else {
            s.fill_rect(viewport_rect, Color::VIEWPORT_BG);
            draw_text(
                s,
                viewport_rect.center_x() - 80,
                viewport_rect.center_y() - 10,
                "No Camera Feed",
                Color::TEXT_MUTED,
            );
        }

        // 2. Draw Top Bar Overlay (Translucent gradient / pills)
        let top_bar_rect = Rect::new(bounds.x, bounds.y, bounds.w, 48);
        s.fill_rect(top_bar_rect, Color::rgba(10, 12, 16, 160));

        // Settings button '≡' on left
        let settings_btn = Rect::new(bounds.x + 14, bounds.y + 10, 32, 28);
        s.fill_rounded_rect(settings_btn, 6, Color::rgba(255, 255, 255, 20));
        s.draw_rounded_rect(settings_btn, 6, Color::OVERLAY_BORDER);
        draw_char(s, settings_btn.x + 10, settings_btn.y + 7, '≡', Color::TEXT_WHITE, &Font::regular());

        // Camera Switcher Pill in top center-left
        if let Some(active_dev) = hub.registry.active_device() {
            let dev_label = format!("{} 🔄", active_dev.name);
            let pill_w = (dev_label.len() as i32) * 8 + 18;
            let cam_pill = Rect::new(bounds.x + 56, bounds.y + 10, pill_w, 28);
            s.fill_rounded_rect(cam_pill, 6, Color::rgba(255, 255, 255, 20));
            s.draw_rounded_rect(cam_pill, 6, Color::OVERLAY_BORDER);
            draw_text_small(s, cam_pill.x + 10, cam_pill.y + 9, &dev_label, Color::TEXT_PRIMARY);
        }

        // Resolution Switcher Badge on top center-right
        let res_label = hub.config.resolution.short_name();
        let res_pill = Rect::new(bounds.x + bounds.w - 180, bounds.y + 10, 68, 28);
        s.fill_rounded_rect(res_pill, 6, Color::rgba(255, 255, 255, 20));
        s.draw_rounded_rect(res_pill, 6, Color::OVERLAY_BORDER);
        draw_text_small(s, res_pill.x + 12, res_pill.y + 9, res_label, Color::ACCENT_BLUE);

        // Recording Status Badge on top far right (pulsing red pill if recording)
        if hub.capture.is_recording() {
            let duration_sec = hub.capture.active_recording.as_ref()
                .map(|r| r.elapsed_seconds(current_epoch))
                .unwrap_or(0);
            let dur_str = VideoRecordingSession::format_duration(duration_sec);
            let rec_text = format!("● REC {}", dur_str);
            let rec_pill = Rect::new(bounds.x + bounds.w - 104, bounds.y + 10, 94, 28);
            s.fill_rounded_rect(rec_pill, 6, Color::REC_BADGE_BG);
            draw_text_small(s, rec_pill.x + 8, rec_pill.y + 9, &rec_text, Color::TEXT_WHITE);
        }

        // 3. Countdown Overlay (Centered in Viewfinder)
        if let Some(ms_left) = hub.capture.countdown_remaining_ms {
            let secs = (ms_left + 999) / 1000;
            let cx = viewport_rect.center_x();
            let cy = viewport_rect.center_y();

            // Background circle with glow
            s.fill_circle(cx, cy, 50, Color::rgba(0, 0, 0, 160));
            s.draw_circle(cx, cy, 50, Color::ACCENT_ORANGE);
            s.draw_circle(cx, cy, 51, Color::ACCENT_ORANGE);

            let num_str = format!("{}", secs);
            draw_text_font(s, cx - 10, cy - 14, &num_str, Color::TEXT_WHITE, &Font::large());
        }

        // 4. Screen Flash Animation (during photo snapshot)
        if hub.capture.is_flashing() {
            let flash_alpha = (hub.capture.flash_remaining_frames as u8) * 60;
            s.fill_rect(viewport_rect, Color::rgba(255, 255, 255, flash_alpha));
        }

        // 5. Draw Bottom Floating Control Bar (Ubuntu Camera style)
        let bottom_rect = Rect::new(bounds.x, bounds.y + bounds.h - bottom_bar_height, bounds.w, bottom_bar_height);
        s.fill_rect(bottom_rect, Color::OVERLAY_BAR_BG);
        s.draw_line(bottom_rect.x, bottom_rect.y, bottom_rect.x + bottom_rect.w, bottom_rect.y, Color::OVERLAY_BORDER);

        let center_x = bottom_rect.center_x();
        let center_y = bottom_rect.center_y();

        // 5a. Countdown Timer Pill on Left: [ ⏱ Off / ⏱ 3s / ⏱ 5s / ⏱ 10s ]
        let timer_str = match hub.config.timer {
            TimerDuration::Off => "⏱ Off",
            TimerDuration::Delay3s => "⏱ 3s",
            TimerDuration::Delay5s => "⏱ 5s",
            TimerDuration::Delay10s => "⏱ 10s",
        };
        let timer_pill = Rect::new(center_x - 170, center_y - 18, 64, 36);
        let timer_active = hub.config.timer != TimerDuration::Off;
        s.fill_rounded_rect(
            timer_pill,
            8,
            if timer_active { Color::PILL_ACTIVE_BG } else { Color::PILL_INACTIVE_BG },
        );
        s.draw_rounded_rect(timer_pill, 8, Color::OVERLAY_BORDER);
        draw_text_small(s, timer_pill.x + 10, timer_pill.y + 13, timer_str, if timer_active { Color::ACCENT_ORANGE } else { Color::TEXT_SECONDARY });

        // 5b. Iconic Shutter / Record Button in Center
        s.draw_shutter_button(
            center_x,
            center_y,
            hub.config.mode,
            hub.capture.is_recording(),
        );

        // 5c. Photo / Video Mode Switcher Pill on Right: [ 📷 | 📹 ]
        let mode_pill = Rect::new(center_x + 106, center_y - 18, 80, 36);
        s.fill_rounded_rect(mode_pill, 18, Color::rgba(0, 0, 0, 100));
        s.draw_rounded_rect(mode_pill, 18, Color::OVERLAY_BORDER);

        let is_photo = hub.config.mode == CameraMode::Photo;
        // Photo toggle half
        if is_photo {
            s.fill_rounded_rect(Rect::new(mode_pill.x + 2, mode_pill.y + 2, 38, 32), 16, Color::PILL_ACTIVE_BG);
        }
        draw_char(s, mode_pill.x + 14, mode_pill.y + 11, '📷', if is_photo { Color::TEXT_WHITE } else { Color::TEXT_MUTED }, &Font::regular());

        // Video toggle half
        if !is_photo {
            s.fill_rounded_rect(Rect::new(mode_pill.x + 40, mode_pill.y + 2, 38, 32), 16, Color::PILL_ACTIVE_BG);
        }
        draw_char(s, mode_pill.x + 52, mode_pill.y + 11, '📹', if !is_photo { Color::TEXT_WHITE } else { Color::TEXT_MUTED }, &Font::regular());

        // 5d. Gallery Thumbnail on Far Right
        let gallery_thumb = Rect::new(bounds.x + bounds.w - 54, center_y - 18, 36, 36);
        s.fill_rounded_rect(gallery_thumb, 8, Color::rgba(255, 255, 255, 25));
        s.draw_rounded_rect(gallery_thumb, 8, Color::OVERLAY_BORDER);
        if let Some(latest) = hub.gallery.latest_item() {
            let label = match latest.kind {
                iris_core::gallery::MediaKind::Photo => "IMG",
                iris_core::gallery::MediaKind::Video => "VID",
            };
            draw_text_small(s, gallery_thumb.x + 6, gallery_thumb.y + 13, label, Color::ACCENT_BLUE);
        }

        // 6. Settings Modal Dialog (if opened)
        if hub.show_settings_modal {
            Self::render_settings_modal(s, bounds);
        }
    }

    /// Renders the comprehensive camera configuration modal dialog
    fn render_settings_modal<S: Surface + ?Sized>(s: &mut S, bounds: Rect) {
        let modal_w = 420;
        let modal_h = 340;
        let mx = bounds.center_x() - modal_w / 2;
        let my = bounds.center_y() - modal_h / 2;
        let modal_rect = Rect::new(mx, my, modal_w, modal_h);

        // Modal backdrop shadow & card
        s.fill_rounded_rect(modal_rect, 12, Color::rgb(26, 30, 38));
        s.draw_rounded_rect(modal_rect, 12, Color::rgba(255, 255, 255, 60));

        // Header
        draw_text(s, mx + 20, my + 16, "Camera Preferences", Color::TEXT_WHITE);
        s.draw_line(mx + 12, my + 42, mx + modal_w - 12, my + 42, Color::rgba(255, 255, 255, 20));

        // Settings rows
        let rows = [
            ("Resolution", "480p SD  720p HD  1080p FHD  4K UHD"),
            ("Composition Grid", "Off  3×3 (Thirds)  Golden  Crosshair"),
            ("Mirror Selfie Mode", "[✓] Enabled (Flip horizontally)"),
            ("Flash Screen Illumination", "[✓] Enabled (Snapshot flash)"),
            ("Shutter Audio Feedback", "[✓] Enabled"),
            ("Storage Photo Directory", "~/Pictures/Camera"),
            ("Storage Video Directory", "~/Videos/Camera"),
        ];

        for (i, (key, val)) in rows.iter().enumerate() {
            let ry = my + 54 + (i as i32) * 34;
            draw_text_small(s, mx + 20, ry, key, Color::TEXT_PRIMARY);
            draw_text_small(s, mx + 20, ry + 14, val, Color::ACCENT_BLUE);
        }

        // Close button
        let done_btn = Rect::new(mx + modal_w - 90, my + modal_h - 40, 70, 26);
        s.fill_rounded_rect(done_btn, 6, Color::rgb(0, 122, 255));
        draw_text(s, done_btn.x + 18, done_btn.y + 6, "Done", Color::TEXT_WHITE);
    }
}
