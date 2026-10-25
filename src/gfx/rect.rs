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
    a.set(v as u32)
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
unsafe fn carry_out(plan: &rc2k_core::gfx::raster::Plan, colour: u32, palette: u32) {
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
    let base = surface().base as i32;
    for &offset in &plan.pixels {
        unsafe { put(base.wrapping_add(offset) as u32, plan.bits, colour, palette) };
    }
}

/// The pixel at `p` for colour `colour` by the surface's depth.
unsafe fn put(p: u32, bits: u32, colour: u32, palette: u32) {
    unsafe {
        match bits {
            16 => (p as *mut u16)
                .write_unaligned(((palette + 4 * colour) as *const u32).read_unaligned() as u16),
            32 => (p as *mut u32)
                .write_unaligned(((palette + 4 * colour) as *const u32).read_unaligned()),
            _ => (p as *mut u8).write(colour as u8),
        }
    }
}

fn rect_at(rect: u32) -> [i32; 4] {
    std::array::from_fn(|n| unsafe { ((rect + 4 * n as u32) as *const i32).read_unaligned() })
}

/// `gfx::fill_rect` (0x406869): rectangle ebx (x, y, width, height) filled, clipped. As the
/// original: on a 16-bit surface `palette[eax]`; on a 32-bit one nothing is drawn (the 32-bit
/// test skips the fill: `notes/bugs.md`). Refuses (before anything) other depths: on 8 bits
/// the original fills with the byte and then once more with dwords past the rectangle's
/// rows.
pub unsafe fn fill_rect(rect: u32, colour: u32, palette: u32) -> Result<(), Refused> {
    let plan = rc2k_core::gfx::raster::fill(rect_at(rect), &core_clip(), &core_surface())?;
    unsafe { carry_out(&plan, colour, palette) };
    Ok(())
}

/// `gfx::draw_hline` (0x406710): `length` pixels from (x, y) rightwards, clipped (the left
/// clip as the original: what is cut is -x, right only for a left edge of 0).
pub unsafe fn draw_hline(x: i32, y: i32, length: i32, colour: u32, palette: u32) {
    let plan = rc2k_core::gfx::raster::hline(x, y, length, &core_clip(), &core_surface());
    unsafe { carry_out(&plan, colour, palette) };
}

/// `gfx::frame_rect` (0x4066DF): rectangle ebx's outline moved by (ecx, edx): the top and
/// bottom rows (`draw_hline`), then the left and right columns between them (`draw_vline`).
pub unsafe fn frame_rect(rect: u32, dx: i32, dy: i32, colour: u32, palette: u32) {
    for plan in rc2k_core::gfx::raster::frame(rect_at(rect), dx, dy, &core_clip(), &core_surface())
    {
        unsafe { carry_out(&plan, colour, palette) };
    }
}
