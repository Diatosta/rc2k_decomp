//! A text in the style of the race's own notices (the car's warnings, "get ready"): the notice
//! font with its outline, no box, enlarged with the screen as the warnings are. Used for the texts
//! the Rust side adds over the race frame (the replay viewer's button names, the players-left
//! messages). The text is drawn at the font's size into a picture of its own whose unwritten
//! pixels stay see-through, then copied to the screen scaled by the screen's height over 600,
//! blended with `strength` over what is there.

use crate::errors::Refused;
use crate::owners::Owners;
use rc2k_core::strings::text::{MODE_AT_X, MODE_MEASURE};

/// Pixels kept round the letters in the picture, for the outline.
const MARGIN: i32 = 4;
/// The colour no glyph has: a picture pixel of it is not written.
const SEE_THROUGH: u32 = 0xFF00FF;

/// The text drawn into a picture of its own, glyph by glyph at the font's own size.
struct Capture {
    inner: crate::text::draw_text::GameCanvas,
    gfx: &'static crate::gfx::owner::Gfx,
    pixels: Vec<u8>,
    width: i32,
    height: i32,
}

impl crate::text::draw_text::Canvas for Capture {
    fn fonts(&self) -> &'static crate::gfx::owner::FontStores {
        self.inner.fonts()
    }
    fn keeps_font_changes(&self) -> bool {
        self.inner.keeps_font_changes()
    }
    fn kerning(&self) -> crate::text::glyph_kerning::State {
        self.inner.kerning()
    }
    fn set_kerning(&mut self, kerning: crate::text::glyph_kerning::State) {
        self.inner.set_kerning(kerning);
    }
    fn clip_right(&self) -> i32 {
        self.width - 1
    }
    fn blit_glyph(&mut self, call: crate::gfx::blit_glyph::BlitCall) -> Result<(), Refused> {
        use crate::gfx::blit_glyph::{Scratch, blit_glyph, glyph_record};
        let mut surface = crate::gfx::surface::surface(self.gfx);
        surface.pitch = self.width * surface.bytes_per_pixel;
        let clip = crate::gfx::surface::Clip {
            left: 0,
            right: self.width - 1,
            top: 0,
            bottom: self.height - 1,
        };
        let record = glyph_record(&self.gfx.fonts, &surface, call.table, call.glyph)?;
        let mut scratch = Scratch {
            left_skip: 0,
            visible_w: 0,
            top_skip: 0,
            rows_left: 0,
        };
        let pixels = &mut self.pixels;
        blit_glyph(
            &self.gfx.fonts,
            &record,
            call.x,
            call.y,
            call.palette,
            &surface,
            &clip,
            &mut scratch,
            &mut |offset, bytes| {
                let at = offset as usize;
                if let Some(to) = pixels.get_mut(at..at + bytes.len()) {
                    to.copy_from_slice(bytes);
                }
            },
        )
    }
}

/// What the picture holds round `(x, y)` (in its pixels, the centre of pixel 0 at 0.5), as the
/// sum of its written pixels' colours weighted by their share and the share written.
fn tap(picture: &Picture, x: f32, y: f32) -> ([f32; 3], f32) {
    let (x, y) = (x - 0.5, y - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let mut sum = [0f32; 3];
    let mut share = 0f32;
    for (dx, dy, w) in [
        (0, 0, (1.0 - fx) * (1.0 - fy)),
        (1, 0, fx * (1.0 - fy)),
        (0, 1, (1.0 - fx) * fy),
        (1, 1, fx * fy),
    ] {
        if let Some(rgb) = picture.written(x0 as i32 + dx, y0 as i32 + dy) {
            for (s, c) in sum.iter_mut().zip(rgb) {
                *s += w * c as f32;
            }
            share += w;
        }
    }
    (sum, share)
}

/// The finished picture and how to read its pixels.
struct Picture {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
    bytes: usize,
    see_through: u32,
}

impl Picture {
    /// The pixel at (x, y) as its raw value, none outside or not written.
    fn raw(&self, x: i32, y: i32) -> Option<u32> {
        if !(0..self.width).contains(&x) || !(0..self.height).contains(&y) {
            return None;
        }
        let at = (y * self.width + x) as usize * self.bytes;
        let mut four = [0u8; 4];
        four[..self.bytes].copy_from_slice(&self.pixels[at..at + self.bytes]);
        let value = u32::from_le_bytes(four);
        (value != self.see_through).then_some(value)
    }

    /// The pixel's three channels (32-bit pixels), none outside or not written.
    fn written(&self, x: i32, y: i32) -> Option<[u32; 3]> {
        self.raw(x, y)
            .map(|v| [v >> 16 & 0xFF, v >> 8 & 0xFF, v & 0xFF])
    }
}

/// The text drawn into a picture of its own at the font's size; none when the screen is not of
/// 32 or 16 bits or the text has no width.
fn capture_picture(o: Owners<'static>, text: &[u8]) -> Option<Picture> {
    let real = crate::gfx::surface::surface(o.gfx);
    let bpp = real.bytes_per_pixel;
    if !(bpp == 2 || bpp == 4) {
        return None;
    }
    let mut bytes = text.to_vec();
    bytes.push(0);
    // The race's notice font, drawn from x as given.
    let mut font = o.gfx.fonts.font_block.game_fonts[2].get();
    font.mode = MODE_AT_X;
    let mut canvas = crate::text::draw_text::GameCanvas::new(o.gfx);
    let mut measure = font;
    measure.mode = MODE_MEASURE;
    let text_width =
        crate::text::measure_text::measure_text(&mut measure, &bytes[..], 0, &mut canvas)
            .map_or(0, |m| m.width as i32);
    let (cw, ch) = (text_width + 2 * MARGIN, font.line_height + 2 * MARGIN);
    if text_width <= 0 || font.line_height <= 0 {
        return None;
    }
    let see_through = crate::input::osk_draw::pixel(SEE_THROUGH, real.bits_per_pixel);
    let mut pixels = Vec::with_capacity((cw * ch * bpp) as usize);
    for _ in 0..cw * ch {
        pixels.extend_from_slice(&see_through.to_le_bytes()[..bpp as usize]);
    }
    let mut capture = Capture {
        inner: canvas,
        gfx: o.gfx,
        pixels,
        width: cw,
        height: ch,
    };
    if crate::text::draw_text::draw_text(&mut font, &bytes[..], MARGIN, MARGIN, &mut capture)
        .is_err()
    {
        return None;
    }
    Some(Picture {
        pixels: capture.pixels,
        width: cw,
        height: ch,
        bytes: bpp as usize,
        see_through,
    })
}

/// Draws `text` (ANSI bytes, no NUL) with its letters' top at `top` (in 800 x 600 layout units),
/// centred across the screen, at `strength` (0 ..= 255). The surface must be locked.
pub fn draw(o: Owners<'static>, text: &[u8], top: i32, strength: u8) {
    let screen = o.system.session.screen.get();
    let area = [0, 0, screen.width.get() as i32, screen.height.get() as i32];
    draw_in(o, text, area, top, strength);
}

/// `draw` within `area` (left, top, width, height in screen pixels, a split-screen part): the text
/// is centred across the area, enlarged by the area's height over 600 (less where the area is
/// narrower than the text), and its letters' top lies `top` layout units down the area.
pub fn draw_in(o: Owners<'static>, text: &[u8], area: [i32; 4], top: i32, strength: u8) {
    use crate::gfx::surface::{Target, surface};
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
    let [area_left, area_top, area_w, area_h] = area;
    let real = surface(o.gfx);
    let Some(Target::Locked(handle)) = real.target else {
        return;
    };
    let bpp = real.bytes_per_pixel;
    if !(bpp == 2 || bpp == 4) || real.pitch <= 0 || strength == 0 {
        return;
    }
    let Some(picture) = capture_picture(o, text) else {
        return;
    };
    let (cw, ch) = (picture.width, picture.height);
    // To the screen, `scale` times as large (less where the screen is narrower than the text).
    let scale = (f64::from(area_h) / 600.0).min(f64::from(area_w) / f64::from(cw)) as f32;
    let (dw, dh) = (
        (cw as f32 * scale).round() as i32,
        (ch as f32 * scale).round() as i32,
    );
    let left = area_left + (area_w - dw) / 2;
    let top = area_top + ((top - MARGIN) as f32 * scale).round() as i32;
    let place = crate::gfx::locked_pixels::Place::start(handle);
    // Smaller than the font's size: four taps cover what a screen pixel does.
    let spread = if scale < 1.0 { 0.25 / scale } else { 0.0 };
    let taps: &[(f32, f32)] = if scale < 1.0 {
        &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
    } else {
        &[(0.0, 0.0)]
    };
    for dy in 0..dh {
        let y = top + dy;
        let (from, to) = (left.max(0), (left + dw).min(w));
        if !(0..h).contains(&y) || to <= from {
            continue;
        }
        let at = (y * real.pitch + from * bpp) as u32;
        let len = ((to - from) * bpp) as usize;
        let Ok(mut row) = place.offset(at).read_bytes(len, "a notice's row") else {
            continue;
        };
        for x in from..to {
            let k = (x - from) as usize * bpp as usize;
            let (sx, sy) = ((x - left) as f32 + 0.5, dy as f32 + 0.5);
            let (sx, sy) = (sx / scale, sy / scale);
            if bpp == 2 {
                // A 16-bit screen has no blend: the nearest written pixel, whole.
                if let Some(v) = picture.raw(sx as i32, sy as i32) {
                    row[k..k + 2].copy_from_slice(&(v as u16).to_le_bytes());
                }
                continue;
            }
            let (mut sum, mut share) = ([0f32; 3], 0f32);
            for &(ox, oy) in taps {
                let (s, c) = tap(&picture, sx + ox * spread, sy + oy * spread);
                for (a, b) in sum.iter_mut().zip(s) {
                    *a += b;
                }
                share += c;
            }
            share /= taps.len() as f32;
            let alpha = (share * f32::from(strength)).round().clamp(0.0, 255.0) as u8;
            if alpha == 0 {
                continue;
            }
            let own = {
                let c = |i: usize| ((sum[i] / (share * taps.len() as f32)).round() as u32).min(255);
                c(0) << 16 | c(1) << 8 | c(2)
            };
            let under = u32::from_le_bytes([row[k], row[k + 1], row[k + 2], row[k + 3]]);
            let px = rc2k_core::race::viewer_tip::over(own, under, alpha);
            row[k..k + 4].copy_from_slice(&px.to_le_bytes());
        }
        place.must_write_bytes_at(at, &row, "a notice's row");
    }
}

#[cfg(phone)]
/// A run of equal pixels in a row of a text picture.
pub struct Run {
    pub x: i32,
    pub y: i32,
    pub len: i32,
    pub rgb: [u8; 3],
}

/// The text in the notice font as runs of pixels (with the picture's width and height), for a
/// renderer that draws them as rectangles at any scale. None where the screen is not 32 bits.
#[cfg(phone)]
pub fn runs(o: Owners<'static>, text: &[u8]) -> Option<(Vec<Run>, i32, i32)> {
    if crate::gfx::surface::surface(o.gfx).bytes_per_pixel != 4 {
        return None;
    }
    let picture = capture_picture(o, text)?;
    let mut runs: Vec<Run> = Vec::new();
    for y in 0..picture.height {
        let mut x = 0;
        while x < picture.width {
            let Some(rgb) = picture.written(x, y) else {
                x += 1;
                continue;
            };
            let mut end = x + 1;
            while picture.written(end, y) == Some(rgb) {
                end += 1;
            }
            runs.push(Run {
                x,
                y,
                len: end - x,
                rgb: [rgb[0] as u8, rgb[1] as u8, rgb[2] as u8],
            });
            x = end;
        }
    }
    Some((runs, picture.width, picture.height))
}
