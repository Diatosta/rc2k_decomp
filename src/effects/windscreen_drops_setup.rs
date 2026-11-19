//! `effects::windscreen_drops_setup`: the windscreen drops' frame set up: their two axes scaled
//! into two more, and those by the setup's scales into two more; the render states, the drops'
//! texture, six polygon vertices (rhw 0x100, colour -1, specular 0xFF000000, texture places 0 / a)
//! and the drops' depth scale (1 over the farthest depth).
//!
//! The x87 run is plain Rust on the x87 model (`math::x87_float`).

use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::math::cell_x87::CellX87;
use crate::math::x87;

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

/// The axes scaled.
fn axes(drops: &WindscreenDrops) {
    axes_x87(drops);
}

pub fn windscreen_drops_setup(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
) {
    let part = scene.scratch.game_scratch();
    use crate::scene::set_render_state::set_render_state;
    {
        axes(drops);
        crate::scene::depth_buffer_on::depth_buffer(part, true);
        // The drops' texture: entry 1999 of the places' slots.
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.places.get().place_slot[1999].get(),
        );
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
        let side = drops.hex.side.get();
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
        drops.work.inverse.set(x87::reciprocal(
            &scene.scratch.front_end_polygon().window.farthest,
        ));
    }
}

use crate::effects::plain::ld_cell;

/// `axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn axes_x87(drops: &WindscreenDrops) {
    let t0 = drops.work.axes[0].ld();
    let t1 = t0 * drops.work.radii[0].ld();
    drops.work.setup_axes[0].st(t1);
    let t2 = drops.work.axes[1].ld();
    let t3 = t2 * drops.work.radii[0].ld();
    drops.work.setup_axes[1].st(t3);
    let t4 = drops.work.axes[2].ld();
    let t5 = t4 * drops.work.radii[0].ld();
    drops.work.setup_axes[2].st(t5);
    let t6 = drops.work.axes[3].ld();
    let t7 = t6 * drops.work.radii[0].ld();
    drops.work.setup_axes[3].st(t7);
    let t8 = drops.work.axes[4].ld();
    let t9 = t8 * drops.work.radii[0].ld();
    drops.work.setup_axes[4].st(t9);
    let t10 = drops.work.axes[5].ld();
    let t11 = t10 * drops.work.radii[0].ld();
    drops.work.setup_axes[5].st(t11);
    let t12 = drops.work.setup_axes[0].ld();
    let t13 = t12 * ld_cell(&drops.hex.axes_k[0]);
    drops.work.setup_axes[6].st(t13);
    let t14 = drops.work.setup_axes[1].ld();
    let t15 = t14 * ld_cell(&drops.hex.axes_k[1]);
    drops.work.setup_axes[7].st(t15);
    let t16 = drops.work.setup_axes[2].ld();
    let t17 = t16 * ld_cell(&drops.hex.axes_k[2]);
    drops.work.setup_axes[8].st(t17);
    let t18 = drops.work.setup_axes[3].ld();
    let t19 = t18 * ld_cell(&drops.hex.axes_k[3]);
    drops.work.setup_axes[9].st(t19);
    let t20 = drops.work.setup_axes[4].ld();
    let t21 = t20 * ld_cell(&drops.hex.axes_k[4]);
    drops.work.setup_axes[10].st(t21);
    let t22 = drops.work.setup_axes[5].ld();
    let t23 = t22 * ld_cell(&drops.hex.axes_k[5]);
    drops.work.setup_axes[11].st(t23);
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::typed_sync::{overlay_laid, with_overlays};

    #[test]
    fn axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF832,
                hi: 0xC9402,
                reads: &[
                    (0xAFC82, F32),
                    (0xAFCDA, F32),
                    (0xAFCDE, F32),
                    (0xAFCE2, F32),
                    (0xAFCE6, F32),
                    (0xAFCEA, F32),
                    (0xAFCEE, F32),
                    (0xAFD2E, F32),
                    (0xAFD32, F32),
                    (0xAFD36, F32),
                    (0xAFD3A, F32),
                    (0xAFD3E, F32),
                    (0xAFD42, F32),
                    (0xC9346, F32),
                    (0xC934A, F32),
                    (0xC934E, F32),
                    (0xC9352, F32),
                    (0xC9356, F32),
                    (0xC935A, F32),
                ],
                writes: &[
                    (0xAFD2E, F32),
                    (0xAFD32, F32),
                    (0xAFD36, F32),
                    (0xAFD3A, F32),
                    (0xAFD3E, F32),
                    (0xAFD42, F32),
                    (0xAFD46, F32),
                    (0xAFD4A, F32),
                    (0xAFD4E, F32),
                    (0xAFD52, F32),
                    (0xAFD56, F32),
                    (0xAFD5A, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes("effects.windscreen_drops_setup.axes_x87", &SPEC, |b| {
            with_overlays(
                b[0],
                &[
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.work),
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
                ],
                || super::axes_x87(&crate::test_support::own::effects().windscreen_drops),
            );
        });
    }
}
