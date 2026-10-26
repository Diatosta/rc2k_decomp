//! `marks::hexagons_setup` (0x4BBE62): the hexagons' frame set up: their two axes
//! (`flt_814C1A`, `flt_814C26`) times `flt_814BC2` into `unk_814C6E` / `unk_814C7A` and those
//! times `flt_82E286` .. into `unk_814C86` / `unk_814C92`; `scene::depth_buffer_on`,
//! `scene::set_texture` (`dword_5D5A24`), render states 0x11 = 1, 0x12 = 1, 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1, 3 = 1; six vertices of `g_polygonVertices` with rhw 0x100
//! (through `dword_62400C`), colour -1, specular 0xFF000000, texture places 0 /
//! `dword_82E2A2` (u 0, a, a, 0; v 0, 0, a, a); `g_polygonVertexCount` 6; `flt_814C02` 1 over
//! `flt_6241C8`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BC0B1` (0x4BC0BF). The x87 run is plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables.

use crate::marks::hexagon_state::HEXAGON_WORK;
use crate::marks::hexagon_vars::HEX;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::scene::draw_store::DRAW;
use crate::scene::texture_places_store::TEXTURE_PLACES;

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

/// The original's 0x4BBE62 .. 0x4BBF1A: the axes scaled.
fn axes() {
    axes_x87(crate::marks::hexagon_vars::HexagonVars::base())
}

/// The Rust version (see the header).
pub fn hexagons_setup() {
    use crate::scene::set_render_state::set_render_state;
    {
        axes();
        crate::scene::depth_buffer_on::depth_buffer(true);
        // `dword_5D5A24`, entry 1999 of the places' slots.
        crate::scene::set_texture::set_texture(TEXTURE_PLACES.place_slot[1999].get());
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        for n in 0..6 {
            DRAW.scratch.set_bits(0x100u32);
            VERTICES.vertices[n].rhw.set_bits(x87::int_to_float(0x100));
        }
        for n in 0..6 {
            VERTICES.vertices[n].colour.set(0xFFFF_FFFFu32);
            VERTICES.vertices[n].specular.set(0xFF00_0000u32);
        }
        let side: u32 = HEX.side.get();
        for (n, (u, v)) in [(0, 0), (side, 0), (side, side), (0, side)]
            .into_iter()
            .enumerate()
        {
            VERTICES.vertices[n].u.set_bits(u);
            VERTICES.vertices[n].v.set_bits(v);
        }
        DRAW.polygon_vertex_count.set_bits(6u32);
        HEXAGON_WORK
            .inverse
            .set_bits(x87::reciprocal(DRAW.farthest.bits()));
    }
}

use crate::marks::plain::ld;

/// `axes`'s x87 block as plain Rust on the x87 model (36 instructions), same order and memory places.
fn axes_x87(hx: usize) {
    let t0 = bh::ld(HEXAGON_WORK.axes[0].bits());
    let t1 = t0 * bh::ld(HEXAGON_WORK.radii[0].bits());
    HEXAGON_WORK.setup_axes[0].set_bits(bh::st(t1));
    let t2 = bh::ld(HEXAGON_WORK.axes[1].bits());
    let t3 = t2 * bh::ld(HEXAGON_WORK.radii[0].bits());
    HEXAGON_WORK.setup_axes[1].set_bits(bh::st(t3));
    let t4 = bh::ld(HEXAGON_WORK.axes[2].bits());
    let t5 = t4 * bh::ld(HEXAGON_WORK.radii[0].bits());
    HEXAGON_WORK.setup_axes[2].set_bits(bh::st(t5));
    let t6 = bh::ld(HEXAGON_WORK.axes[3].bits());
    let t7 = t6 * bh::ld(HEXAGON_WORK.radii[0].bits());
    HEXAGON_WORK.setup_axes[3].set_bits(bh::st(t7));
    let t8 = bh::ld(HEXAGON_WORK.axes[4].bits());
    let t9 = t8 * bh::ld(HEXAGON_WORK.radii[0].bits());
    HEXAGON_WORK.setup_axes[4].set_bits(bh::st(t9));
    let t10 = bh::ld(HEXAGON_WORK.axes[5].bits());
    let t11 = t10 * bh::ld(HEXAGON_WORK.radii[0].bits());
    HEXAGON_WORK.setup_axes[5].set_bits(bh::st(t11));
    let t12 = bh::ld(HEXAGON_WORK.setup_axes[0].bits());
    let t13 = t12 * ld(hx, 0xC9346);
    HEXAGON_WORK.setup_axes[6].set_bits(bh::st(t13));
    let t14 = bh::ld(HEXAGON_WORK.setup_axes[1].bits());
    let t15 = t14 * ld(hx, 0xC934A);
    HEXAGON_WORK.setup_axes[7].set_bits(bh::st(t15));
    let t16 = bh::ld(HEXAGON_WORK.setup_axes[2].bits());
    let t17 = t16 * ld(hx, 0xC934E);
    HEXAGON_WORK.setup_axes[8].set_bits(bh::st(t17));
    let t18 = bh::ld(HEXAGON_WORK.setup_axes[3].bits());
    let t19 = t18 * ld(hx, 0xC9352);
    HEXAGON_WORK.setup_axes[9].set_bits(bh::st(t19));
    let t20 = bh::ld(HEXAGON_WORK.setup_axes[4].bits());
    let t21 = t20 * ld(hx, 0xC9356);
    HEXAGON_WORK.setup_axes[10].set_bits(bh::st(t21));
    let t22 = bh::ld(HEXAGON_WORK.setup_axes[5].bits());
    let t23 = t22 * ld(hx, 0xC935A);
    HEXAGON_WORK.setup_axes[11].set_bits(bh::st(t23));
}

#[cfg(test)]
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
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};
    use crate::marks::hexagon_state::{HEXAGON_WORK, HexagonWork};
    use crate::marks::typed_sync::{overlay, with_overlays};

    #[test]
    fn axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF832,
                hi: 0xC935E,
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
                with_overlays(b[0], &[overlay(&HEXAGON_WORK, HexagonWork::START)], || {
                    super::axes_x87(b[0])
                })
            },
        );
    }
}
