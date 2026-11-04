//! `effects::draw_flare_part` (0x4C4FFB): one part of the lens flare drawn as a screen
//! square. Its centre (st(0) x, st(1) y) rounded into `g_polygonVertices`' fifth vertex
//! (+0x80, +0x84); the four vertices' colour (ecx << 24) | esi; their texture places from
//! corner eax of `g_marksCorners` (its words times `flt_8EB969`: 0/2, 4/6, 0xC/0xE, 8/0xA);
//! the side: edx (`dword_8EB8B5`) by edx times the corner's height over its width (the
//! words' differences, 16-bit, made positive: `dword_8EB8B9`), times `flt_9206DE` /
//! `flt_9206E2` into `flt_8EB941` / `flt_8EB945`; the corners (0, side) less those, turned
//! by `g_sines` / `g_cosines`[ebx >> 20] about the centre; depth `dword_8EB961` (as a
//! float) times `flt_8EB96D`; `dword_8F3981` plus 1; `effects::clip_quad`, and unless all is
//! outside `scene::draw_fan`.
//!
//! In:  the corner, the angle << 20, the alpha, the size, the colour (the original's
//!      eax, ebx, ecx, edx, esi), the centre (st(0), st(1), popped)
//! Out: nothing (its one caller keeps its registers round it: `pusha` .. `popa`)
//!
//! Caller: `effects::draw_lens_flare` (0x4C4FEA). Refuses a corner past `g_marksCorners`
//! (0x100 or more) and null marks settings (`clip_quad`'s refusal), before anything.

use crate::effects::blocks::MARK_CORNERS;
use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::frame_state::{DRAW_WORK, FLARE_SCALES, VIEW_VARS};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::matrix3::rounded;

const CORNERS: u32 = 0x100;
use crate::d3d::vertices_store::VERTICES;
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0, 2), (4, 6), (0xC, 0xE), (8, 0xA)];

/// A 16-bit difference made positive (`sub bp, ..; jns; neg bp`), the high half 0.
fn abs16(a: u32, b: u32) -> u32 {
    let d = (a as u16).wrapping_sub(b as u16);
    (if (d as i16) < 0 { d.wrapping_neg() } else { d }) as u32
}

/// The Rust version (see the header): `corner` (the original's eax), `angle` (ebx, << 20),
/// `alpha` (ecx), `size` (edx), `colour` (esi), `centre` (st(0) x, st(1) y).
pub fn draw_flare_part(
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
    {
        if crate::effects::setup::SETTINGS.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for n in 0..2 {
            DRAW_WORK.sides[n].set_bits(x87::loaded(FLARE_SCALES.scale[n].bits()));
        }
        let colour = (alpha << 24) | colour;
        for n in 0..4 {
            VERTICES.vertices[n].colour.set(colour);
        }
        let entry = &MARK_CORNERS.corners[corner as usize];
        let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
        let uv_k: u32 = DRAW_WORK.uv_k.bits();
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = &VERTICES.vertices[n];
            vertex.u.set_bits(x87::int_times(word(u) as i32, uv_k));
            vertex.v.set_bits(x87::int_times(word(v) as i32, uv_k));
        }
        let [cx, cy] = centre.map(rounded);
        VERTICES.vertices[4].x.set_bits(cx);
        VERTICES.vertices[4].y.set_bits(cy);
        let across = abs16(word(4), word(0));
        let along = abs16(word(0xA), word(2));
        let ratio = x87::int_over_int(along as i32, across as i32);
        let width = x87::int_to_float(size as i32);
        let height = x87::int_times(size as i32, ratio);
        DRAW_WORK.half[0].set_bits(width);
        DRAW_WORK.half[1].set_bits(height);
        let turn = angle >> 20;
        let sw = x87::mul(width, DRAW_WORK.sides[0].bits());
        let sh = x87::mul(height, DRAW_WORK.sides[1].bits());
        DRAW_WORK.sides[0].set_bits(sw);
        DRAW_WORK.sides[1].set_bits(sh);
        let corners = [
            (x87::sub(0, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(height, sh)),
            (x87::sub(0, sw), x87::sub(height, sh)),
        ];
        let sin: u32 = crate::scene::engine_tables_store::sine(turn as usize).bits();
        let cos: u32 = crate::scene::engine_tables_store::cosine(turn as usize).bits();
        let depth = x87::mul(DRAW_WORK.near.bits(), DRAW_WORK.z_k.bits());
        for (n, (x, y)) in corners.into_iter().enumerate() {
            let vertex = &VERTICES.vertices[n];
            let [tx, ty] = x87::turn_about(x, y, sin, cos, cx, cy);
            vertex.x.set_bits(tx);
            vertex.y.set_bits(ty);
            vertex.z.set_bits(depth);
        }
        VIEW_VARS
            .cleared
            .set(VIEW_VARS.cleared.get().wrapping_add(1));
        if clip_quad()? {
            return Ok(());
        }
        draw_fan();
    }
    Ok(())
}
