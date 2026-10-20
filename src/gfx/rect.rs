//! Solid rectangles and lines straight on the surface (`sub_406869`, `sub_4066DF`,
//! `sub_406710`, `sub_4067AF`; stream A, part F1, 2026-10-18): the message box's fill and
//! frame. Colour eax: the byte itself on an 8-bit surface, else `palette[eax]` (esi a table
//! of 256 dwords). Clipped to the clip rectangle; the work in `g_blitLeftSkip`,
//! `g_blitVisibleWidth`, `g_blitTopSkip`, `g_blitRowsLeft`.

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::gfx::surface::{clip, surface};

const LEFT_SKIP: usize = rva(0x4F504C); // g_blitLeftSkip
const WIDTH: usize = rva(0x4F5050); // g_blitVisibleWidth
const TOP_SKIP: usize = rva(0x4F5054); // g_blitTopSkip
const ROWS: usize = rva(0x4F5058); // g_blitRowsLeft

// Converted: `rc2k_core::gfx::raster`; here the clip and surface are read from the game, and
// the work variables and pixels the plan holds written back.

fn set(a: usize, v: i32) {
    unsafe { game_write(a, v as u32) }
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
        (LEFT_SKIP, w.left_skip),
        (WIDTH, w.width),
        (TOP_SKIP, w.top_skip),
        (ROWS, w.rows),
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

/// `gfx::draw_vline` (0x4067AF): `length` pixels from (x, y) downwards, clipped (the top clip
/// as the original: what is cut is -y).
pub unsafe fn draw_vline(x: i32, y: i32, length: i32, colour: u32, palette: u32) {
    let plan = rc2k_core::gfx::raster::vline(x, y, length, &core_clip(), &core_surface());
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

/// A test surface (64 x 32, a gap of 8 bytes a row) at 16 or 32 bits, a clip rectangle and a
/// palette, all on the game's variables; everything put back by `end`.
pub struct RectStage {
    pub pixels: Vec<u8>,
    pub palette: Vec<u32>,
    pub kept: Vec<crate::service::harness::Region>,
    pub saved: Vec<u8>,
}

impl RectStage {
    pub fn new() -> Self {
        use crate::gfx::surface::*;
        use crate::service::harness::{at, capture};
        let kept = vec![
            at(BITS_PER_PIXEL_RVA),
            at(BYTES_PER_PIXEL_RVA),
            at(PITCH_RVA),
            at(BASE_RVA),
            at(CLIP_LEFT_RVA),
            at(CLIP_RIGHT_RVA),
            at(CLIP_TOP_RVA),
            at(CLIP_BOTTOM_RVA),
            at(LEFT_SKIP),
            at(WIDTH),
            at(TOP_SKIP),
            at(ROWS),
        ];
        let saved = capture(&kept);
        Self {
            pixels: vec![0; (64 * 4 + 8) * 32],
            palette: (0..256u32)
                .map(|k| k.wrapping_mul(0x0101_0307) ^ 0x8000_0000)
                .collect(),
            kept,
            saved,
        }
    }

    /// The surface at `bits`, cleared to 0xAA, the clip (left, right, top, bottom).
    pub fn prepare(&self, bits: u32, c: [i32; 4]) {
        use crate::gfx::surface::*;
        unsafe {
            std::ptr::write_bytes(self.pixels.as_ptr() as *mut u8, 0xAA, self.pixels.len());
            game_write(BITS_PER_PIXEL_RVA, bits);
            game_write(BYTES_PER_PIXEL_RVA, bits / 8);
            game_write(PITCH_RVA, 64 * (bits / 8) + 8);
            game_write(BASE_RVA, self.pixels.as_ptr() as u32);
            game_write(CLIP_LEFT_RVA, c[0] as u32);
            game_write(CLIP_RIGHT_RVA, c[1] as u32);
            game_write(CLIP_TOP_RVA, c[2] as u32);
            game_write(CLIP_BOTTOM_RVA, c[3] as u32);
            for a in [LEFT_SKIP, WIDTH, TOP_SKIP, ROWS] {
                game_write(a, 0x5555_5555u32);
            }
        }
    }

    /// What the functions write.
    pub fn regions(&self) -> Vec<crate::service::harness::Region> {
        use crate::service::harness::at;
        vec![
            (self.pixels.as_ptr() as u32, self.pixels.len()),
            at(LEFT_SKIP),
            at(WIDTH),
            at(TOP_SKIP),
            at(ROWS),
        ]
    }

    pub fn end(self) {
        crate::service::harness::restore(&self.kept, &self.saved);
    }
}

impl Default for RectStage {
    fn default() -> Self {
        Self::new()
    }
}

/// The test cases: (bits, clip, x, y, width, height).
pub const CASES: [(u32, [i32; 4], i32, i32, i32, i32); 8] = [
    (16, [0, 63, 0, 31], 3, 2, 10, 5),
    (16, [4, 40, 3, 20], 0, 0, 20, 10),
    (32, [0, 63, 0, 31], 50, 25, 30, 20),
    (32, [8, 63, 5, 31], -5, -3, 20, 10),
    (16, [0, 63, 0, 31], 70, 2, 5, 5),
    (32, [0, 63, 0, 31], 10, 10, 0, 4),
    (16, [10, 30, 10, 20], 5, 5, 40, 30),
    (32, [0, 63, 0, 31], 0, 31, 64, 1),
];
