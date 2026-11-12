//! Solid rectangles and lines straight on the surface (`gfx::fill_rect`, `gfx::frame_rect`,
//! `gfx::draw_hline`): the message box's fill and frame. The colour is the byte itself on
//! an 8-bit surface, else `palette[colour]` (a table of 256 dwords). Clipped to the clip rectangle; the work in `g_blitLeftSkip`,
//! `g_blitVisibleWidth`, `g_blitTopSkip`, `g_blitRowsLeft`.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::gfx::surface::{clip, surface};

// Converted: `rc2k_core::gfx::raster`; here the clip and surface are read from the game, and
// the work variables and pixels the plan holds written back.

fn set(a: &crate::session::PlainDword, v: i32) {
    a.set(v as u32);
}

fn core_clip(gfx: &Gfx) -> rc2k_core::gfx::raster::Clip {
    let c = clip(gfx);
    rc2k_core::gfx::raster::Clip {
        left: c.left,
        right: c.right,
        top: c.top,
        bottom: c.bottom,
    }
}

fn core_surface(gfx: &Gfx) -> rc2k_core::gfx::raster::Surface {
    let s = surface(gfx);
    rc2k_core::gfx::raster::Surface {
        bits: s.bits_per_pixel,
        bytes_per_pixel: s.bytes_per_pixel,
        pitch: s.pitch,
    }
}

/// The palette a colour is looked up in: 256 dwords.
#[derive(Clone, Copy)]
pub enum Palette<'a> {
    /// A font's palette as the game's data keeps it: a place of `gfx::glyph_store`.
    Held(u32),
    /// A palette of Rust's game state: a store, bounded by it.
    Of(&'a dyn crate::pointers::Store),
}

impl Palette<'_> {
    /// Entry `colour`.
    fn entry(self, colour: u32) -> u32 {
        let what = "a palette entry";
        let bytes = match self {
            Palette::Held(place) => crate::gfx::glyph_store::read_bytes(place, 4 * colour, 4, what),
            Palette::Of(store) => {
                let mut entry = vec![0; 4];
                store.load(4 * colour as usize, &mut entry).map(|()| entry)
            }
        };
        match bytes {
            Ok(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            Err(e) => crate::fatal::raise::end(what, e),
        }
    }
}

/// The plan's work variables, then its pixels in colour `colour`.
fn carry_out(plan: &rc2k_core::gfx::raster::Plan, colour: u32, palette: Palette<'_>, gfx: &Gfx) {
    let drawing = &gfx.drawing;
    let w = plan.work;
    for (at, value) in [
        (&drawing.blit.left_skip, w.left_skip),
        (&drawing.blit.visible_width, w.width),
        (&drawing.blit.top_skip, w.top_skip),
        (&drawing.blit.rows_left, w.rows),
    ] {
        if let Some(v) = value {
            set(at, v);
        }
    }
    let Some(target) = surface(gfx).target else {
        crate::fatal::raise::end("gfx::rect", crate::errors::BadInput("no surface"))
    };
    for &offset in &plan.pixels {
        let bytes = pixel(plan.bits, colour, palette);
        if let Err(e) = crate::gfx::surface::write(target, offset as u32, &bytes, gfx) {
            crate::fatal::raise::end("gfx::rect", e)
        }
    }
}

/// The pixel for colour `colour` by the surface's depth: `palette[colour]` as a word or a
/// dword, else the byte itself.
fn pixel(bits: u32, colour: u32, palette: Palette<'_>) -> Vec<u8> {
    let entry = || palette.entry(colour);
    match bits {
        16 => (entry() as u16).to_le_bytes().to_vec(),
        32 => entry().to_le_bytes().to_vec(),
        _ => vec![colour as u8],
    }
}

/// A rectangle (x, y, width, height) kept as four dwords.
pub fn rect_of(cells: &[crate::session::Dword; 4]) -> [i32; 4] {
    cells.each_ref().map(|c| c.get() as i32)
}

/// A rectangle (x, y, width, height) kept in the first 16 bytes of a buffer.
pub fn rect_in(bytes: &impl crate::session::ByteBuffer) -> [i32; 4] {
    std::array::from_fn(|n| bytes.get_u32_at(4 * n) as i32)
}

/// `gfx::fill_rect` (0x406869): the rectangle (x, y, width, height) filled, clipped. As the
/// original: on a 16-bit surface `palette[colour]`; on a 32-bit one nothing is drawn (the 32-bit
/// test skips the fill: `notes/bugs.md`). Refuses (before anything) other depths: on 8 bits
/// the original fills with the byte and then once more with dwords past the rectangle's
/// rows.
pub fn fill_rect(
    rect: [i32; 4],
    colour: u32,
    palette: Palette<'_>,
    gfx: &Gfx,
) -> Result<(), Refused> {
    let plan = rc2k_core::gfx::raster::fill(rect, &core_clip(gfx), &core_surface(gfx))?;
    carry_out(&plan, colour, palette, gfx);
    Ok(())
}

/// `gfx::draw_hline` (0x406710): `length` pixels from (x, y) rightwards, clipped (the left
/// clip as the original: what is cut is -x, right only for a left edge of 0).
pub fn draw_hline(x: i32, y: i32, length: i32, colour: u32, palette: Palette<'_>, gfx: &Gfx) {
    let plan = rc2k_core::gfx::raster::hline(x, y, length, &core_clip(gfx), &core_surface(gfx));
    carry_out(&plan, colour, palette, gfx);
}

/// `gfx::frame_rect` (0x4066DF): the rectangle's outline moved by (dx, dy): the top and
/// bottom rows (`draw_hline`), then the left and right columns between them (`draw_vline`).
pub fn frame_rect(rect: [i32; 4], dx: i32, dy: i32, colour: u32, palette: Palette<'_>, gfx: &Gfx) {
    for plan in rc2k_core::gfx::raster::frame(rect, dx, dy, &core_clip(gfx), &core_surface(gfx)) {
        carry_out(&plan, colour, palette, gfx);
    }
}
