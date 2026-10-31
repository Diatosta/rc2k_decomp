//! Solid rectangles and lines straight on the surface (`sub_406869`, `sub_4066DF`,
//! `sub_406710`, `sub_4067AF`; stream A, part F1, 2026-10-18): the message box's fill and
//! frame. Colour eax: the byte itself on an 8-bit surface, else `palette[eax]` (esi a table
//! of 256 dwords). Clipped to the clip rectangle; the work in `g_blitLeftSkip`,
//! `g_blitVisibleWidth`, `g_blitTopSkip`, `g_blitRowsLeft`.

use crate::errors::Refused;
use crate::gfx::blit::BLIT;
use crate::gfx::surface::{clip, surface};
use crate::session::Dword;

// Converted: `rc2k_core::gfx::raster`; here the clip and surface are read from the game, and
// the work variables and pixels the plan holds written back.

fn set(a: &Dword, v: i32) {
    a.set(v as u32);
}

fn core_clip() -> rc2k_core::gfx::raster::Clip {
    let c = clip();
    rc2k_core::gfx::raster::Clip {
        left: c.left,
        right: c.right,
        top: c.top,
        bottom: c.bottom,
    }
}

fn core_surface() -> rc2k_core::gfx::raster::Surface {
    let s = surface();
    rc2k_core::gfx::raster::Surface {
        bits: s.bits_per_pixel,
        bytes_per_pixel: s.bytes_per_pixel,
        pitch: s.pitch,
    }
}

/// The plan's work variables, then its pixels in colour `colour`.
fn carry_out(plan: &rc2k_core::gfx::raster::Plan, colour: u32, palette: u32) {
    let w = plan.work;
    for (at, value) in [
        (&BLIT.left_skip, w.left_skip),
        (&BLIT.visible_width, w.width),
        (&BLIT.top_skip, w.top_skip),
        (&BLIT.rows_left, w.rows),
    ] {
        if let Some(v) = value {
            set(at, v);
        }
    }
    let Some(target) = surface().target else {
        crate::fatal::raise::end("gfx::rect", crate::errors::BadInput("no surface"))
    };
    for &offset in &plan.pixels {
        let bytes = pixel(plan.bits, colour, palette);
        if let Err(e) = crate::gfx::surface::write(target, offset as u32, &bytes) {
            crate::fatal::raise::end("gfx::rect", e)
        }
    }
}

/// The pixel for colour `colour` by the surface's depth: `palette[colour]` as a word or a
/// dword, else the byte itself.
fn pixel(bits: u32, colour: u32, palette: u32) -> Vec<u8> {
    // The palette is a place of `gfx::glyph_store` (the fonts') or a game address.
    let entry =
        || match crate::gfx::glyph_store::read_bytes(palette, 4 * colour, 4, "a palette entry") {
            Ok(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            Err(e) => crate::fatal::raise::end("a palette entry", e),
        };
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
pub fn rect_in<const N: usize>(bytes: &crate::session::Bytes<N>) -> [i32; 4] {
    std::array::from_fn(|n| bytes.get_u32(4 * n) as i32)
}

/// `gfx::fill_rect` (0x406869): rectangle ebx (x, y, width, height) filled, clipped. As the
/// original: on a 16-bit surface `palette[eax]`; on a 32-bit one nothing is drawn (the 32-bit
/// test skips the fill: `notes/bugs.md`). Refuses (before anything) other depths: on 8 bits
/// the original fills with the byte and then once more with dwords past the rectangle's
/// rows.
pub fn fill_rect(rect: [i32; 4], colour: u32, palette: u32) -> Result<(), Refused> {
    let plan = rc2k_core::gfx::raster::fill(rect, &core_clip(), &core_surface())?;
    carry_out(&plan, colour, palette);
    Ok(())
}

/// `gfx::draw_hline` (0x406710): `length` pixels from (x, y) rightwards, clipped (the left
/// clip as the original: what is cut is -x, right only for a left edge of 0).
pub fn draw_hline(x: i32, y: i32, length: i32, colour: u32, palette: u32) {
    let plan = rc2k_core::gfx::raster::hline(x, y, length, &core_clip(), &core_surface());
    carry_out(&plan, colour, palette);
}

/// `gfx::frame_rect` (0x4066DF): rectangle ebx's outline moved by (ecx, edx): the top and
/// bottom rows (`draw_hline`), then the left and right columns between them (`draw_vline`).
pub fn frame_rect(rect: [i32; 4], dx: i32, dy: i32, colour: u32, palette: u32) {
    for plan in rc2k_core::gfx::raster::frame(rect, dx, dy, &core_clip(), &core_surface()) {
        carry_out(&plan, colour, palette);
    }
}
