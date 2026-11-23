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

/// How much of the letters' fill the picture holds round `(x, y)`: the fill of its four nearest
/// pixels weighted by their share (`Picture::fill`), 0 to 1.
fn tap_fill(picture: &Picture, x: f32, y: f32) -> f32 {
    let (x, y) = (x - 0.5, y - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    [
        (0, 0, (1.0 - fx) * (1.0 - fy)),
        (1, 0, fx * (1.0 - fy)),
        (0, 1, (1.0 - fx) * fy),
        (1, 1, fx * fy),
    ]
    .into_iter()
    .map(|(dx, dy, w)| w * picture.fill(x0 as i32 + dx, y0 as i32 + dy))
    .sum()
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

    /// How much of the letters' fill the pixel is, 0 to 1: its brightness where it is written
    /// (the font's fill is white, its outline black, the edge between them grey), 0 elsewhere.
    fn fill(&self, x: i32, y: i32) -> f32 {
        let Some(v) = self.raw(x, y) else {
            return 0.0;
        };
        let [r, g, b] = if self.bytes == 4 {
            [v >> 16 & 0xFF, v >> 8 & 0xFF, v & 0xFF]
        } else {
            // 5:6:5, each channel taken out to 8 bits.
            let c = |value: u32, bits: u32| (value << (8 - bits)) | (value >> (2 * bits - 8));
            [c(v >> 11 & 0x1F, 5), c(v >> 5 & 0x3F, 6), c(v & 0x1F, 5)]
        };
        (r + g + b) as f32 / 765.0
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
    let [area_left, area_top, area_w, area_h] = area;
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
    let lie = Lie {
        left: left as f32,
        top: top as f32,
        across: scale,
        down: scale,
    };
    paint(o, &picture, lie, [left, top, dw, dh], strength, None);
}

/// A text's letters without their outline as one shape of the colour `colour` (0xRRGGBB): where
/// the font's fill is the colour is, its edges as soft as the fill's brightness says. The shape
/// lies so that the centre of mass of the fill is at the screen's `centre_x` and the first row
/// with any fill in it at `top`, one of the font's pixels `across` screen pixels wide and `down`
/// high (the screen's pixels, as floats: a layout's picture is not set on whole pixels). Meant for
/// the digits the replay viewer puts on its pictures (`rc2k_core::race::viewer_digits`), which
/// the atlas' own digits are the shape of. The surface must be locked.
pub fn draw_shape(
    o: Owners<'static>,
    text: &[u8],
    colour: u32,
    [centre_x, top]: [f32; 2],
    [across, down]: [f32; 2],
) {
    let Some(picture) = capture_picture(o, text) else {
        return;
    };
    let Some(ink) =
        rc2k_core::race::viewer_digits::ink_of(picture.width, picture.height, |x, y| {
            picture.fill(x, y)
        })
    else {
        return;
    };
    let lie = Lie {
        left: centre_x - ink.centre_x * across,
        top: top - ink.top as f32 * down,
        across,
        down,
    };
    // The screen pixels the picture reaches, one more each side for its soft edge.
    let rect = [
        lie.left.floor() as i32 - 1,
        lie.top.floor() as i32 - 1,
        (picture.width as f32 * across).ceil() as i32 + 3,
        (picture.height as f32 * down).ceil() as i32 + 3,
    ];
    paint(o, &picture, lie, rect, 255, Some(colour));
}

/// Where a picture lies on the screen: its top left corner, and how many screen pixels one of its
/// pixels is wide and high.
#[derive(Clone, Copy)]
struct Lie {
    left: f32,
    top: f32,
    across: f32,
    down: f32,
}

/// `picture` blended over the screen's pixels in `rect` (left, top, width, height) as it lies
/// (`lie`), at `strength` (0 ..= 255): the colours of its own pixels, or, with a `shape` colour
/// (0xRRGGBB), the shape of its letters' fill (`Picture::fill`) in that colour. A picture pixel's
/// share of the screen pixel decides the blend, so edges are soft; the surface must be locked, and
/// the 16-bit screens take the nearest pixel whole.
fn paint(
    o: Owners<'static>,
    picture: &Picture,
    lie: Lie,
    [rect_left, rect_top, rect_w, rect_h]: [i32; 4],
    strength: u8,
    shape: Option<u32>,
) {
    use crate::gfx::surface::{Target, surface};
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
    let real = surface(o.gfx);
    let Some(Target::Locked(handle)) = real.target else {
        return;
    };
    let bpp = real.bytes_per_pixel;
    if !(bpp == 2 || bpp == 4) || real.pitch <= 0 || strength == 0 {
        return;
    }
    let place = crate::gfx::locked_pixels::Place::start(handle);
    // Smaller than the font's size: four taps cover what a screen pixel does.
    let shrunk = lie.across < 1.0 || lie.down < 1.0;
    let (spread_x, spread_y) = if shrunk {
        (0.25 / lie.across, 0.25 / lie.down)
    } else {
        (0.0, 0.0)
    };
    let taps: &[(f32, f32)] = if shrunk {
        &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
    } else {
        &[(0.0, 0.0)]
    };
    for y in rect_top..rect_top + rect_h {
        let (from, to) = (rect_left.max(0), (rect_left + rect_w).min(w));
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
            let sx = ((x as f32 + 0.5) - lie.left) / lie.across;
            let sy = ((y as f32 + 0.5) - lie.top) / lie.down;
            if bpp == 2 {
                // A 16-bit screen has no blend: the nearest written pixel, whole (a shape: where the
                // fill is more than half).
                let at = (sx.floor() as i32, sy.floor() as i32);
                let value = match shape {
                    Some(rgb) => (picture.fill(at.0, at.1) >= 0.5)
                        .then(|| crate::input::osk_draw::pixel(rgb, real.bits_per_pixel)),
                    None => picture.raw(at.0, at.1),
                };
                if let Some(v) = value {
                    row[k..k + 2].copy_from_slice(&(v as u16).to_le_bytes());
                }
                continue;
            }
            let (mut sum, mut share) = ([0f32; 3], 0f32);
            for &(ox, oy) in taps {
                let (px, py) = (sx + ox * spread_x, sy + oy * spread_y);
                if shape.is_some() {
                    share += tap_fill(picture, px, py);
                    continue;
                }
                let (s, c) = tap(picture, px, py);
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
            let own = shape.unwrap_or_else(|| {
                let c = |i: usize| ((sum[i] / (share * taps.len() as f32)).round() as u32).min(255);
                c(0) << 16 | c(1) << 8 | c(2)
            });
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
