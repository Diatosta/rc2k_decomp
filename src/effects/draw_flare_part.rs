//! `effects::draw_flare_part` (0x4C4FFB): one part of the lens flare drawn as a screen square.
//! Its centre (x, y) rounded into the polygon vertex buffer's fifth vertex (+0x80, +0x84); the
//! four vertices' colour (alpha << 24) | colour; their texture places from the mark corner
//! (its words times `uv_k`: 0/2, 4/6, 0xC/0xE, 8/0xA); the side: the size (`half`) by the size
//! times the corner's height over its width (the words' differences, 16-bit, made positive),
//! times two constants into `sides`; the corners (0, side) less those, turned by the sine and
//! cosine tables at (angle >> 20) about the centre; depth `near` (as a float) times `z_k`;
//! `cleared` plus 1; `effects::clip_quad`, and unless all is outside `scene::draw_fan`.
//!
//! Caller: `effects::draw_lens_flare`. Refuses a corner past the mark corners table
//! (0x100 or more) and null marks settings (`clip_quad`'s refusal), before anything.

use crate::effects::clip_quad::{clip_quad, draw_fan};
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

/// `corner` is the mark corner's number, `angle` the turn (<< 20), `alpha` and `colour` the
/// vertices' colour, `size` the side, `centre` the square's centre (st(0) x, st(1) y).
#[allow(
    clippy::too_many_arguments,
    reason = "the original's registers, the effects' groups and the owners"
)]
pub fn draw_flare_part(
    corner: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    colour: u32,
    centre: [[u8; 10]; 2],
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if corner >= CORNERS {
        return Err(BadInput("a flare corner past g_marksCorners").into());
    }
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for n in 0..2 {
            effects.frame.draw_work.sides[n].set(x87::loaded(&effects.frame.flare_scales.scale[n]));
        }
        let colour = (alpha << 24) | colour;
        for n in 0..4 {
            VERTICES.vertices[n].colour.set(colour);
        }
        let entry = &effects.blocks.corners.corners[corner as usize];
        let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
        let uv_k = effects.frame.draw_work.uv_k.get();
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = &VERTICES.vertices[n];
            vertex.u.set(x87::int_times(word(u) as i32, uv_k));
            vertex.v.set(x87::int_times(word(v) as i32, uv_k));
        }
        let [cx, cy] = centre.map(rounded);
        VERTICES.vertices[4].x.set_bits(cx);
        VERTICES.vertices[4].y.set_bits(cy);
        let across = abs16(word(4), word(0));
        let along = abs16(word(0xA), word(2));
        let ratio = x87::int_ratio(along as i32, across as i32);
        let width = x87::from_int(size as i32);
        let height = x87::int_times(size as i32, ratio);
        effects.frame.draw_work.half[0].set(width);
        effects.frame.draw_work.half[1].set(height);
        let turn = angle >> 20;
        let sw = x87::mul(width, &effects.frame.draw_work.sides[0]);
        let sh = x87::mul(height, &effects.frame.draw_work.sides[1]);
        effects.frame.draw_work.sides[0].set(sw);
        effects.frame.draw_work.sides[1].set(sh);
        let corners = [
            (x87::sub(0, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(height, sh)),
            (x87::sub(0, sw), x87::sub(height, sh)),
        ];
        let sin: u32 = crate::scene::engine_tables_store::sine(turn as usize).bits();
        let cos: u32 = crate::scene::engine_tables_store::cosine(turn as usize).bits();
        let depth = x87::mul(&effects.frame.draw_work.near, &effects.frame.draw_work.z_k);
        for (n, (x, y)) in corners.into_iter().enumerate() {
            let vertex = &VERTICES.vertices[n];
            let [tx, ty] = x87::turn_about(x, y, sin, cos, cx, cy);
            vertex.x.set(tx);
            vertex.y.set(ty);
            vertex.z.set(depth);
        }
        effects
            .frame
            .view_vars
            .cleared
            .set(effects.frame.view_vars.cleared.get().wrapping_add(1));
        if clip_quad(effects)? {
            return Ok(());
        }
        draw_fan(scene);
    }
    Ok(())
}
