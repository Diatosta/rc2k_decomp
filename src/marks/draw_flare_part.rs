//! `marks::draw_flare_part` (0x4C4FFB): one part of the lens flare drawn as a screen
//! square. Its centre (st(0) x, st(1) y) rounded into `g_polygonVertices`' fifth vertex
//! (+0x80, +0x84); the four vertices' colour (ecx << 24) | esi; their texture places from
//! corner eax of `g_marksCorners` (its words times `flt_8EB969`: 0/2, 4/6, 0xC/0xE, 8/0xA);
//! the side: edx (`dword_8EB8B5`) by edx times the corner's height over its width (the
//! words' differences, 16-bit, made positive: `dword_8EB8B9`), times `flt_9206DE` /
//! `flt_9206E2` into `flt_8EB941` / `flt_8EB945`; the corners (0, side) less those, turned
//! by `g_sines` / `g_cosines`[ebx >> 20] about the centre; depth `dword_8EB961` (as a
//! float) times `flt_8EB96D`; `dword_8F3981` plus 1; `marks::clip_quad`, and unless all is
//! outside `scene::draw_fan`.
//!
//! In:  the corner, the angle << 20, the alpha, the size, the colour (the original's
//!      eax, ebx, ecx, edx, esi), the centre (st(0), st(1), popped)
//! Out: nothing (its one caller keeps its registers round it: `pusha` .. `popa`)
//!
//! Caller: `marks::draw_lens_flare` (0x4C4FEA). The clip is its Rust version (the copy's
//! call goes to its hook); the draw through `part` (1 `draw_fan`), in the game its Rust
//! version. Refuses a corner past `g_marksCorners` (0x100 or more) and null marks
//! settings (`clip_quad`'s refusal), before anything. Thunk by `two_in_hook!`. Shadow mode
//! runs the original alone (it draws). Self-test: a copy whose draw goes to a stand-in,
//! against the Rust version, centres, corners, sizes and angles by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::math::x87;
use crate::scene::matrix3::rounded;

const SCALE: [usize; 2] = [rva(0x9206DE), rva(0x9206E2)]; // flt_9206DE, flt_9206E2
const SIDE_RVA: [usize; 2] = [rva(0x8EB941), rva(0x8EB945)]; // flt_8EB941, flt_8EB945
const SIZE_RVA: [usize; 2] = [rva(0x8EB8B5), rva(0x8EB8B9)]; // dword_8EB8B5, dword_8EB8B9
const CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners: 0x100 of 0x80 bytes
const CORNERS: u32 = 0x100;
const UV_K_RVA: usize = rva(0x8EB969); // flt_8EB969
const NEAR_RVA: usize = rva(0x8EB961); // dword_8EB961
const Z_K_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const DRAWN_RVA: usize = rva(0x8F3981); // dword_8F3981
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4], a fifth's x, y
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0, 2), (4, 6), (0xC, 0xE), (8, 0xA)];

/// A 16-bit difference made positive (`sub bp, ..; jns; neg bp`), the high half 0.
fn abs16(a: u32, b: u32) -> u32 {
    let d = (a as u16).wrapping_sub(b as u16);
    (if (d as i16) < 0 { d.wrapping_neg() } else { d }) as u32
}

/// The Rust version (see the header): `corner` (the original's eax), `angle` (ebx, << 20),
/// `alpha` (ecx), `size` (edx), `colour` (esi), `centre` (st(0) x, st(1) y).
pub unsafe fn draw_flare_part(
    corner: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    colour: u32,
    centre: [[u8; 10]; 2],
) -> Result<(), Refused> {
    if corner >= CORNERS {
        return Err(BadInput("a flare corner past g_marksCorners").into());
    }
    unsafe {
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for n in 0..2 {
            game_write(SIDE_RVA[n], x87::loaded(game_read(SCALE[n])));
        }
        let colour = (alpha << 24) | colour;
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
        }
        let entry = game_addr(CORNERS_RVA) as u32 + corner * 0x80;
        let word = |off: u32| ((entry + off) as *const u16).read_unaligned() as u32;
        let uv_k: u32 = game_read(UV_K_RVA);
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
            game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
        }
        let [cx, cy] = centre.map(rounded);
        game_write(VERTICES_RVA + 0x80, cx);
        game_write(VERTICES_RVA + 0x84, cy);
        let across = abs16(word(4), word(0));
        let along = abs16(word(0xA), word(2));
        let ratio = x87::int_over_int(along as i32, across as i32);
        let width = x87::int_to_float(size as i32);
        let height = x87::int_times(size as i32, ratio);
        game_write(SIZE_RVA[0], width);
        game_write(SIZE_RVA[1], height);
        let turn = angle >> 20;
        let sw = x87::mul(width, game_read(SIDE_RVA[0]));
        let sh = x87::mul(height, game_read(SIDE_RVA[1]));
        game_write(SIDE_RVA[0], sw);
        game_write(SIDE_RVA[1], sh);
        let corners = [
            (x87::sub(0, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(height, sh)),
            (x87::sub(0, sw), x87::sub(height, sh)),
        ];
        let sin: u32 = game_read(SINES_RVA + 4 * turn as usize);
        let cos: u32 = game_read(COSINES_RVA + 4 * turn as usize);
        let depth = x87::mul(game_read(NEAR_RVA), game_read(Z_K_RVA));
        for (n, (x, y)) in corners.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            let [tx, ty] = x87::turn_about(x, y, sin, cos, cx, cy);
            game_write(vertex, tx);
            game_write(vertex + 4, ty);
            game_write(vertex + 8, depth);
        }
        game_write(DRAWN_RVA, game_read::<u32>(DRAWN_RVA).wrapping_add(1));
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
    }
    Ok(())
}
