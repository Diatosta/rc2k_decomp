//! `effects::windscreen_drops_setup`: the windscreen drops' frame set up: their two axes scaled
//! into two more, and those by the setup's scales into two more; the render states, the drops'
//! texture, six polygon vertices (rhw 0x100, colour -1, specular 0xFF000000, texture places 0 / a)
//! and the drops' depth scale (1 over the farthest depth).
//!
//! The x87 run is plain Rust on the x87 model (`math::x87_float`).

use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::math::x87;
use crate::math::x87_float::{self as bh, stf};

const STATES: [(u32, u32); 8] = [
    (1, 0x11),
    (1, 0x12),
    (4, 0x15),
    (5, 0x13),
    (6, 0x14),
    (0, 0x1C),
    (1, 0x1B),
    (1, 3),
];

pub fn windscreen_drops_setup(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
) {
    let part = scene.scratch.game_scratch();
    use crate::scene::set_render_state::set_render_state;
    {
        axes_x87(drops);
        crate::scene::depth_buffer_on::depth_buffer(part, true);
        // The drops' texture: entry 1999 of the places' slots.
        let drops_texture = scene.textures.place_slots.borrow().place_slot[1999];
        crate::scene::set_texture::set_texture(part, scene, drops_texture);
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        for item in buffer.vertices.iter().take(6) {
            item.rhw.set(x87::from_int(0x100));
        }
        for item in buffer.vertices.iter().take(6) {
            item.colour.set(0xFFFF_FFFFu32);
            item.specular.set(0xFF00_0000u32);
        }
        let side = drops.hex.borrow().side;
        for (n, (u, v)) in [(0.0, 0.0), (side, 0.0), (side, side), (0.0, side)]
            .into_iter()
            .enumerate()
        {
            buffer.vertices[n].u.set(u);
            buffer.vertices[n].v.set(v);
        }
        scene
            .scratch
            .front_end_polygon()
            .polygon_vertex_count
            .set(6);
        drops.work.borrow_mut().inverse =
            x87::reciprocal(&scene.scratch.front_end_polygon().window.farthest);
    }
}

/// The axes scaled, as plain Rust on the x87 model, in the same order.
fn axes_x87(drops: &WindscreenDrops) {
    let hex = drops.hex.borrow();
    let mut work = drops.work.borrow_mut();
    let t = bh::ld(work.axes[0]) * bh::ld(work.radii[0]);
    work.setup_axes[0] = stf(t);
    let t = bh::ld(work.axes[1]) * bh::ld(work.radii[0]);
    work.setup_axes[1] = stf(t);
    let t = bh::ld(work.axes[2]) * bh::ld(work.radii[0]);
    work.setup_axes[2] = stf(t);
    let t = bh::ld(work.axes[3]) * bh::ld(work.radii[0]);
    work.setup_axes[3] = stf(t);
    let t = bh::ld(work.axes[4]) * bh::ld(work.radii[0]);
    work.setup_axes[4] = stf(t);
    let t = bh::ld(work.axes[5]) * bh::ld(work.radii[0]);
    work.setup_axes[5] = stf(t);
    let t = bh::ld(work.setup_axes[0]) * bh::ld(hex.axes_k[0]);
    work.setup_axes[6] = stf(t);
    let t = bh::ld(work.setup_axes[1]) * bh::ld(hex.axes_k[1]);
    work.setup_axes[7] = stf(t);
    let t = bh::ld(work.setup_axes[2]) * bh::ld(hex.axes_k[2]);
    work.setup_axes[8] = stf(t);
    let t = bh::ld(work.setup_axes[3]) * bh::ld(hex.axes_k[3]);
    work.setup_axes[9] = stf(t);
    let t = bh::ld(work.setup_axes[4]) * bh::ld(hex.axes_k[4]);
    work.setup_axes[10] = stf(t);
    let t = bh::ld(work.setup_axes[5]) * bh::ld(hex.axes_k[5]);
    work.setup_axes[11] = stf(t);
}

#[cfg(test)]
mod tests;
