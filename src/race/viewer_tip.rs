//! The name of the highlighted button, shown under the viewer's render button for a second or so
//! when the highlight moves (and when the viewer opens on its play button), then faded out over
//! a quarter of a second. It is drawn on the 2D layer over the finished frame, in the font of the
//! viewer's name row on a dark box, only while the viewer is stopped and not while a video is
//! rendered. The choice of the text and the timing are `rc2k_core::race::viewer_tip`.

use crate::errors::Refused;
use crate::gfx::typefaces::fonts;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::race::render_row;
use rc2k_core::race::viewer_tip::{
    BUTTONS, HUD_BUTTONS, Highlight, SPEED_BUTTON, button_text, hud_text, speed_text, strength,
};
use rc2k_core::strings::own::id;
use std::sync::Mutex;
use std::time::Instant;

/// What had the highlight when the name was last started, and when.
struct Shown {
    highlight: Highlight,
    since: Instant,
}

static SHOWN: Mutex<Option<Shown>> = Mutex::new(None);

/// The viewer opens: the first highlight shown names itself.
pub fn reset() {
    *SHOWN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// Where the highlight is now.
fn highlight(race: &Race, choice: u32) -> Highlight {
    let editing = race.viewer_buffers().camera_edit.get();
    if editing != 0 {
        Highlight::Editing(editing)
    } else if render_row::selected() {
        Highlight::Render
    } else {
        Highlight::Button(choice.min(BUTTONS - 1))
    }
}

/// The text that names `highlight`, in the language in use ("{}" filled in).
fn text_of(o: Owners<'static>, highlight: Highlight) -> String {
    let race = o.race;
    let block = race.replay.replay_view().replay_block.get() as usize;
    let camera_byte =
        |byte: usize| crate::race::viewer::camera_byte_at(race, byte.wrapping_add(8 * block)).ok();
    let (text, with) = match highlight {
        Highlight::Render => (id::VIEWER_TIP_RENDER, None),
        Highlight::Editing(camera) => (id::VIEWER_TIP_EDITING, Some(camera)),
        Highlight::Button(button) => {
            let hud = HUD_BUTTONS.iter().position(|&b| b == button);
            match (
                hud,
                camera_byte(if button == SPEED_BUTTON {
                    4
                } else {
                    2 + hud.unwrap_or(0)
                }),
            ) {
                (Some(car), Some(byte)) => (hud_text(byte != 0), Some(car as u32 + 1)),
                (None, Some(byte)) if button == SPEED_BUTTON => (speed_text(byte), None),
                _ => (button_text(button).unwrap_or(id::BLANK), None),
            }
        }
    };
    let template = rc2k_core::files::ansi::ansi_text(
        &crate::strings::append_string_by_id::own_text(o.strings, text),
    );
    template.replace("{}", &with.map(|n| n.to_string()).unwrap_or_default())
}

/// The box and the name drawn over the frame, if the name is still to show.
pub fn draw(o: Owners<'static>) -> Result<(), Refused> {
    let highlight = highlight(o.race, o.menus.list.choice.get());
    let elapsed = {
        let mut shown = SHOWN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        match shown.as_mut() {
            Some(s) if s.highlight == highlight => {}
            _ => {
                *shown = Some(Shown {
                    highlight,
                    since: now,
                });
            }
        }
        shown
            .as_ref()
            .map_or(0, |s| s.since.elapsed().as_millis() as u32)
    };
    let Some(strength) = strength(elapsed) else {
        return Ok(());
    };
    let text = text_of(o, highlight);
    draw_label(o, &rc2k_core::files::ansi::ansi_bytes(&text), strength);
    Ok(())
}

/// The label's layout, in the panel's 800 x 600 units, which `scale` takes to the screen as it
/// takes the panel's own items and texts: the box lies centred just under the render button
/// (the button ends at layout y 227), so it hides nothing of the panel.
const BOX_TOP: i32 = 235;
const BOX_HEIGHT: i32 = 20;
/// The letters' height in the font, where they start in the box, and the padding at each side.
const LETTERS_TOP: i32 = 3;
const PADDING: i32 = 10;
const BOX_COLOUR: u32 = 0x0C1238;

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

fn draw_label(o: Owners<'static>, text: &[u8], strength: u8) {
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return;
    }
    put_label(o, text, strength);
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
}

/// The label made at the font's size, then copied to the screen scaled by the panel's factor.
fn put_label(o: Owners<'static>, text: &[u8], strength: u8) {
    use crate::gfx::surface::{Target, surface};
    use crate::text::draw_text::{GameCanvas, draw_text_at};
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
    let real = surface(o.gfx);
    let Some(Target::Locked(handle)) = real.target else {
        return;
    };
    let bpp = real.bytes_per_pixel;
    if !(bpp == 2 || bpp == 4) || real.pitch <= 0 {
        return;
    }
    let font = fonts::PANEL_A;
    let mut canvas = GameCanvas::new(o.gfx);
    let (cw, ch) = (
        crate::gfx::button_hints::width_of(font, text, &mut canvas) + 2 * PADDING,
        BOX_HEIGHT,
    );
    // The picture: the box's colour, the letters over it.
    let box_pixel = crate::input::osk_draw::pixel(BOX_COLOUR, real.bits_per_pixel);
    let mut pixels = Vec::with_capacity((cw * ch * bpp) as usize);
    for _ in 0..cw * ch {
        pixels.extend_from_slice(&box_pixel.to_le_bytes()[..bpp as usize]);
    }
    let mut capture = Capture {
        inner: canvas,
        gfx: o.gfx,
        pixels,
        width: cw,
        height: ch,
    };
    let mut bytes = text.to_vec();
    bytes.push(0);
    let _ = draw_text_at(font, &bytes[..], PADDING, LETTERS_TOP, &mut capture);
    // To the screen, `scale` times as large, centred; where it is a frame's own pixel the
    // label's blends into it, over the GPU's unwritten canvas it carries its alpha instead.
    let scale = f64::from(h) / 600.0;
    let (dw, dh) = (
        (f64::from(cw) * scale).round() as i32,
        (f64::from(ch) * scale).round() as i32,
    );
    let (left, top) = ((w - dw) / 2, (f64::from(BOX_TOP) * scale).round() as i32);
    let place = crate::gfx::locked_pixels::Place::start(handle);
    for dy in 0..dh {
        let y = top + dy;
        if !(0..h).contains(&y) {
            continue;
        }
        let from = left.max(0);
        let to = (left + dw).min(w);
        if to <= from {
            continue;
        }
        let at = (y * real.pitch + from * bpp) as u32;
        let len = ((to - from) * bpp) as usize;
        let before = if strength < 255 && bpp == 4 {
            place.offset(at).read_bytes(len, "the label's row").ok()
        } else {
            None
        };
        let mut row = Vec::with_capacity(len);
        for x in from..to {
            let src = |sx: f64, sy: f64| {
                let (sx, sy) = (
                    ((sx.max(0.0)) as i32).min(cw - 1),
                    ((sy.max(0.0)) as i32).min(ch - 1),
                );
                let p = ((sy * cw + sx) * bpp) as usize;
                let mut px = [0u8; 4];
                px[..bpp as usize].copy_from_slice(&capture.pixels[p..p + bpp as usize]);
                px
            };
            let (fx, fy) = (f64::from(x - left) / scale, f64::from(dy) / scale);
            let mut px = src(fx, fy);
            if scale < 1.0 && bpp == 4 {
                // Smaller than the font's size: the four pixels the new one covers, averaged.
                let step = 0.5 / scale;
                let taps = [
                    src(fx, fy),
                    src(fx + step, fy),
                    src(fx, fy + step),
                    src(fx + step, fy + step),
                ];
                for c in 0..3 {
                    px[c] = (taps.iter().map(|t| u32::from(t[c])).sum::<u32>() / 4) as u8;
                }
            }
            if let Some(before) = &before {
                let k = 4 * (x - from) as usize;
                let under =
                    u32::from_le_bytes([before[k], before[k + 1], before[k + 2], before[k + 3]]);
                let own = u32::from_le_bytes([px[0], px[1], px[2], 0]);
                px = rc2k_core::race::viewer_tip::over(own, under, strength).to_le_bytes();
            }
            row.extend_from_slice(&px[..bpp as usize]);
        }
        place.must_write_bytes_at(at, &row, "the viewer's label");
    }
}
