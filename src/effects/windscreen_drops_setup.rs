//! `effects::windscreen_drops_setup` (0x4BBE62): the windscreen drops' frame set up: their two
//! axes times a scale into two more, and those times the setup's scales into two more;
//! `scene::depth_buffer_on`, `scene::set_texture` (the drops' texture), render states 0x11 =
//! 1, 0x12 = 1, 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1, 3 = 1; six polygon vertices
//! with rhw 0x100, colour -1, specular 0xFF000000, texture places 0 / a (u 0, a, a, 0; v 0, 0,
//! a, a); the polygon's vertex count 6; the drops' depth scale 1 over the farthest depth.
//!
//! Caller: `effects::windscreen_drops_frame` The x87 run is plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables.

use crate::effects::cell_x87::CellX87;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::math::x87;

use crate::d3d::vertices_store::VERTICES;
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

pub fn windscreen_drops_setup(drops: &WindscreenDrops, scene: &crate::scene::owner::Scene) {
    use crate::scene::set_render_state::set_render_state;
    {
        axes(drops);
        crate::scene::depth_buffer_on::depth_buffer(true);
        // The drops' texture: entry 1999 of the places' slots.
        crate::scene::set_texture::set_texture(scene, scene.textures.places.place_slot[1999].get());
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        for n in 0..6 {
            VERTICES.vertices[n].rhw.set(x87::from_int(0x100));
        }
        for n in 0..6 {
            VERTICES.vertices[n].colour.set(0xFFFF_FFFFu32);
            VERTICES.vertices[n].specular.set(0xFF00_0000u32);
        }
        let side: u32 = drops.hex.side.get();
        for (n, (u, v)) in [(0, 0), (side, 0), (side, side), (0, side)]
            .into_iter()
            .enumerate()
        {
            VERTICES.vertices[n].u.set_bits(u);
            VERTICES.vertices[n].v.set_bits(v);
        }
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set_bits(6u32);
        drops.work.inverse.set(x87::reciprocal(
            &crate::scene::polygon_work::front_end().window.farthest,
        ));
    }
}

use crate::effects::plain::ld_cell;

/// `axes`'s x87 block as plain Rust on the x87 model (36 instructions), same order and memory places.
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

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn axes_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0xAFCDA]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD2E]",
            "fld dword ptr [{mk} + 0xAFCDE]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD32]",
            "fld dword ptr [{mk} + 0xAFCE2]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD36]",
            "fld dword ptr [{mk} + 0xAFCE6]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD3A]",
            "fld dword ptr [{mk} + 0xAFCEA]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD3E]",
            "fld dword ptr [{mk} + 0xAFCEE]",
            "fmul dword ptr [{mk} + 0xAFC82]",
            "fstp dword ptr [{mk} + 0xAFD42]",
            "fld dword ptr [{mk} + 0xAFD2E]",
            "fmul dword ptr [{mk} + 0xC9346]",
            "fstp dword ptr [{mk} + 0xAFD46]",
            "fld dword ptr [{mk} + 0xAFD32]",
            "fmul dword ptr [{mk} + 0xC934A]",
            "fstp dword ptr [{mk} + 0xAFD4A]",
            "fld dword ptr [{mk} + 0xAFD36]",
            "fmul dword ptr [{mk} + 0xC934E]",
            "fstp dword ptr [{mk} + 0xAFD4E]",
            "fld dword ptr [{mk} + 0xAFD3A]",
            "fmul dword ptr [{mk} + 0xC9352]",
            "fstp dword ptr [{mk} + 0xAFD52]",
            "fld dword ptr [{mk} + 0xAFD3E]",
            "fmul dword ptr [{mk} + 0xC9356]",
            "fstp dword ptr [{mk} + 0xAFD56]",
            "fld dword ptr [{mk} + 0xAFD42]",
            "fmul dword ptr [{mk} + 0xC935A]",
            "fstp dword ptr [{mk} + 0xAFD5A]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_state::WindscreenDropWork;
    use crate::effects::windscreen_drop_vars::WindscreenDropVars;

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
        differential(
            &SPEC,
            |b| reference::axes_x87(b[0]),
            |b| {
                with_overlays(
                    b[0],
                    &[
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.work,
                            WindscreenDropWork::START,
                        ),
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || super::axes_x87(&crate::effects::owner::effects().windscreen_drops),
                );
            },
        );
    }
}
