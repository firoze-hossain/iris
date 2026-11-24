//! crates/iris-ui/src/font.rs
//!
//! Typography engine and procedural camera iconography for Iris Camera UI.

use crate::draw::Surface;
use crate::geom::{Color, Rect};

#[derive(Clone, Copy, Debug)]
pub struct Font {
    pub char_width: i32,
    pub char_height: i32,
}

impl Font {
    pub const fn regular() -> Self {
        Font { char_width: 8, char_height: 14 }
    }

    pub const fn small() -> Self {
        Font { char_width: 6, char_height: 10 }
    }

    pub const fn large() -> Self {
        Font { char_width: 14, char_height: 24 }
    }

    pub fn measure_str(&self, text: &str) -> Rect {
        let w = (text.chars().count() as i32) * self.char_width;
        Rect::new(0, 0, w, self.char_height)
    }
}

pub fn draw_text<S: Surface + ?Sized>(s: &mut S, x: i32, y: i32, text: &str, color: Color) {
    draw_text_font(s, x, y, text, color, &Font::regular());
}

pub fn draw_text_small<S: Surface + ?Sized>(s: &mut S, x: i32, y: i32, text: &str, color: Color) {
    draw_text_font(s, x, y, text, color, &Font::small());
}

pub fn draw_text_font<S: Surface + ?Sized>(
    s: &mut S,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
    font: &Font,
) {
    let mut cx = x;
    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        draw_char(s, cx, y, ch, color, font);
        cx += font.char_width;
    }
}

pub fn draw_char<S: Surface + ?Sized>(s: &mut S, x: i32, y: i32, ch: char, color: Color, font: &Font) {
    if ch == ' ' {
        return;
    }

    // Procedural icons
    match ch {
        '📷' => {
            // Still camera icon
            let w = font.char_width.max(12);
            let h = font.char_height.max(10);
            let cy = y + (font.char_height - h) / 2;
            s.draw_rounded_rect(Rect::new(x + 1, cy + 2, w - 2, h - 3), 2, color);
            // Lens circle
            s.draw_circle(x + w / 2, cy + 2 + (h - 3) / 2, (h - 5) / 2, color);
            // Top flash bump
            s.fill_rect(Rect::new(x + w / 3, cy, w / 3, 2), color);
            return;
        }
        '📹' => {
            // Video camcorder icon
            let w = font.char_width.max(12);
            let h = font.char_height.max(10);
            let cy = y + (font.char_height - h) / 2;
            // Body
            s.draw_rounded_rect(Rect::new(x + 1, cy + 1, (w * 2) / 3, h - 2), 2, color);
            // Triangular lens cone
            let cone_x = x + (w * 2) / 3 + 1;
            let mid_y = cy + h / 2;
            s.draw_line(cone_x, mid_y - 2, cone_x + 3, mid_y - 4, color);
            s.draw_line(cone_x, mid_y + 2, cone_x + 3, mid_y + 4, color);
            s.draw_line(cone_x + 3, mid_y - 4, cone_x + 3, mid_y + 4, color);
            return;
        }
        '⏱' => {
            // Timer stopwatch icon
            let r = (font.char_height.min(font.char_width) - 4) / 2;
            let cx = x + font.char_width / 2;
            let cy = y + font.char_height / 2 + 1;
            s.draw_circle(cx, cy, r, color);
            // Top button
            s.draw_line(cx - 1, cy - r - 2, cx + 1, cy - r - 2, color);
            s.draw_line(cx, cy - r - 2, cx, cy - r, color);
            // Hand
            s.draw_line(cx, cy, cx + r / 2, cy - r / 2, color);
            return;
        }
        '≡' => {
            // Settings hamburger lines
            let w = font.char_width.max(10);
            let y_mid = y + font.char_height / 2;
            s.draw_line(x + 1, y_mid - 4, x + w - 2, y_mid - 4, color);
            s.draw_line(x + 1, y_mid, x + w - 2, y_mid, color);
            s.draw_line(x + 1, y_mid + 4, x + w - 2, y_mid + 4, color);
            return;
        }
        '🔄' => {
            // Camera flip/switch icon
            let r = (font.char_height - 4) / 2;
            let cx = x + font.char_width / 2;
            let cy = y + font.char_height / 2;
            s.draw_circle(cx, cy, r, color);
            s.draw_line(cx + r, cy, cx + r - 3, cy - 3, color);
            s.draw_line(cx - r, cy, cx - r + 3, cy + 3, color);
            return;
        }
        '●' => {
            // Recording red circle
            let r = (font.char_height - 6) / 2;
            s.fill_circle(x + font.char_width / 2, y + font.char_height / 2, r, color);
            return;
        }
        '■' => {
            // Recording stop square
            let s_size = font.char_height - 6;
            let sy = y + 3;
            s.fill_rect(Rect::new(x + 1, sy, s_size, s_size), color);
            return;
        }
        _ => {}
    }

    let glyph = get_glyph_bitmap(ch);
    for row in 0..8 {
        let bits = glyph[row];
        for col in 0..8 {
            if (bits & (1 << (7 - col))) != 0 {
                let px = x + ((col as i32) * font.char_width) / 8;
                let py = y + ((row as i32) * font.char_height) / 8;
                s.put_pixel(px, py, color);
                if font.char_width >= 8 && font.char_height >= 14 {
                    s.put_pixel(px + 1, py, color);
                }
            }
        }
    }
}

fn get_glyph_bitmap(ch: char) -> [u8; 8] {
    match ch {
        '0' => [0x3C, 0x46, 0x4A, 0x52, 0x62, 0x42, 0x3C, 0x00],
        '1' => [0x18, 0x28, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00],
        '2' => [0x3C, 0x42, 0x02, 0x0C, 0x30, 0x40, 0x7E, 0x00],
        '3' => [0x3C, 0x42, 0x02, 0x1C, 0x02, 0x42, 0x3C, 0x00],
        '4' => [0x04, 0x0C, 0x14, 0x24, 0x7E, 0x04, 0x04, 0x00],
        '5' => [0x7E, 0x40, 0x7C, 0x02, 0x02, 0x42, 0x3C, 0x00],
        '6' => [0x1C, 0x20, 0x40, 0x7C, 0x42, 0x42, 0x3C, 0x00],
        '7' => [0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x20, 0x00],
        '8' => [0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x3C, 0x00],
        '9' => [0x3C, 0x42, 0x42, 0x3E, 0x02, 0x04, 0x38, 0x00],
        ':' => [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30],
        '+' => [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '/' => [0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x00],
        '(' => [0x08, 0x10, 0x20, 0x20, 0x20, 0x10, 0x08, 0x00],
        ')' => [0x20, 0x10, 0x08, 0x08, 0x08, 0x10, 0x20, 0x00],
        '[' => [0x1C, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1C, 0x00],
        ']' => [0x38, 0x08, 0x08, 0x08, 0x08, 0x08, 0x38, 0x00],
        '<' => [0x08, 0x10, 0x20, 0x40, 0x20, 0x10, 0x08, 0x00],
        '>' => [0x20, 0x10, 0x08, 0x04, 0x08, 0x10, 0x20, 0x00],
        '=' => [0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x00],
        '%' => [0x62, 0x64, 0x08, 0x10, 0x20, 0x4C, 0x8C, 0x00],
        '!' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00],
        '?' => [0x3C, 0x42, 0x04, 0x08, 0x10, 0x00, 0x10, 0x00],
        'A' | 'a' => [0x18, 0x24, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
        'B' | 'b' => [0x7C, 0x42, 0x42, 0x7C, 0x42, 0x42, 0x7C, 0x00],
        'C' | 'c' => [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00],
        'D' | 'd' => [0x78, 0x44, 0x42, 0x42, 0x42, 0x44, 0x78, 0x00],
        'E' | 'e' => [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x7E, 0x00],
        'F' | 'f' => [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x40, 0x00],
        'G' | 'g' => [0x3C, 0x42, 0x40, 0x4E, 0x42, 0x42, 0x3C, 0x00],
        'H' | 'h' => [0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
        'I' | 'i' => [0x3E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00],
        'J' | 'j' => [0x1E, 0x04, 0x04, 0x04, 0x44, 0x44, 0x38, 0x00],
        'K' | 'k' => [0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00],
        'L' | 'l' => [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00],
        'M' | 'm' => [0x42, 0x66, 0x5A, 0x42, 0x42, 0x42, 0x42, 0x00],
        'N' | 'n' => [0x42, 0x62, 0x52, 0x4A, 0x46, 0x42, 0x42, 0x00],
        'O' | 'o' => [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'P' | 'p' => [0x7C, 0x42, 0x42, 0x7C, 0x40, 0x40, 0x40, 0x00],
        'Q' | 'q' => [0x3C, 0x42, 0x42, 0x42, 0x4A, 0x44, 0x3A, 0x00],
        'R' | 'r' => [0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x00],
        'S' | 's' => [0x3C, 0x42, 0x40, 0x3C, 0x02, 0x42, 0x3C, 0x00],
        'T' | 't' => [0x7E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00],
        'U' | 'u' => [0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'V' | 'v' => [0x42, 0x42, 0x42, 0x24, 0x24, 0x18, 0x18, 0x00],
        'W' | 'w' => [0x42, 0x42, 0x42, 0x42, 0x5A, 0x66, 0x42, 0x00],
        'X' | 'x' => [0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x42, 0x00],
        'Y' | 'y' => [0x42, 0x42, 0x24, 0x18, 0x18, 0x18, 0x18, 0x00],
        'Z' | 'z' => [0x7E, 0x04, 0x08, 0x10, 0x20, 0x40, 0x7E, 0x00],
        _ => [0x3C, 0x42, 0x04, 0x08, 0x10, 0x00, 0x10, 0x00], // '?'
    }
}
