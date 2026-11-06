//! `effects::draw_subcell_sprite` (0x4CBF47): a ring entry's square sprite from a subcell:
//! the place (st(0) x, st(1) y, st(2) z) turned by `g_viewMatrix` into `flt_8EB8D5` ..
//! (the depth also whole in `dword_9356C0`); unless the depth is at or below
//! `dword_8EB961` (a signed compare of the bits): the four vertices' specular
//! `scene::fog_at` of the depth, colour the entry's +0x90 | 0xFF000000, texture places from
//! subcell eax of `g_marksSubcellPlaces`, depth times `flt_8EB96D`; half the side (edx's
//! bits as a float, times `flt_624088` / depth) in `dword_8EB8B5` and `dword_8EB8B9`; the
//! corners about the projected place; `effects::clip_quad`, and unless all is outside
//! `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  st(0) .. st(2) the place (popped), eax the subcell, edx the size (a float), esi the
//!      entry
//! Out: nothing (the original keeps every register but ecx, which no caller reads after it)
//!
//! Callers: the ring's kinds 0x10, 0x11 (`sub_4CB64D`) and code at 0x4C55EF .. 0x4C5AB4.
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the game's variables. Refuses,
//! before anything, a subcell past the table's 0x410, a null entry, and the fog's and the
//! clip's refusals.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::ring_store::RingWork;
use crate::errors::{BadInput, Refused};
use crate::race::owner_frame::Frame;
use crate::scene::fog_at::fog_at;

use crate::d3d::vertices_store::VERTICES;
const SUBCELLS: u32 = 0x410;

/// The original's 0x4CBF47 .. 0x4CBFCB: the place turned by the view.
fn turn(frame: &Frame, ring: &Ring, p: &[[u8; 10]; 3]) {
    turn_x87(
        frame,
        p,
        &ring.work,
        &crate::effects::owner::effects().frame.draw_work,
    );
}

/// The original's 0x4CC009 .. 0x4CC140 (its integer moves done before): the square about
/// the projected place.
fn square() {
    square_x87(&crate::effects::owner::effects().frame.draw_work);
}

/// The Rust version (see the header): `subcell` (eax), `size` (edx, a float's bits),
/// `entry` (esi). No caller reads a register it leaves.
pub fn draw_subcell_sprite(
    frame: &Frame,
    masks: &Masks,
    ring: &Ring,
    subcell: u32,
    size: u32,
    entry: u32,
    place: [[u8; 10]; 3],
) -> Result<(), Refused> {
    if subcell >= SUBCELLS {
        return Err(BadInput("a mark subcell past g_marksSubcellPlaces").into());
    }
    if entry == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    fog_at(0)?;
    if crate::effects::owner::effects().setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    turn(frame, ring, &place);
    if crate::effects::owner::effects().frame.draw_work.turned[2].bits() as i32
        <= crate::effects::owner::effects().frame.draw_work.near.bits() as i32
    {
        return Ok(());
    }
    let fog = fog_at(ring.work.whole.get())?;
    let colour = crate::effects::ring_entries::get(ring, entry, 0x90)? | 0xFF00_0000;
    for n in 0..4 {
        VERTICES.vertices[n].specular.set(fog);
        VERTICES.vertices[n].colour.set(colour);
    }
    crate::effects::owner::effects().frame.draw_work.half[0].set_bits(size);
    let at = subcell << 4;
    let place = |n: usize| masks.surface.word(0x400 + at as usize + 4 * n);
    let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
    for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
        .into_iter()
        .enumerate()
    {
        VERTICES.vertices[n].u.set_bits(u);
        VERTICES.vertices[n].v.set_bits(v);
    }
    square();
    if clip_quad()? {
        return Ok(());
    }
    draw_fan();
    crate::scene::owner::scene()
        .drawing
        .scene
        .polygon_vertex_count
        .set_bits(4u32);
    Ok(())
}

use crate::effects::frame_state::DrawWork;
use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;
use crate::math::x87_float::ld;

/// `turn`'s x87 block as plain Rust on the x87 model (37 instructions), same order and memory places.
fn turn_x87(frame: &Frame, p: &[[u8; 10]; 3], w: &RingWork, dw: &DrawWork) {
    let t0 = bh::from_f80(p[2]);
    let t1 = bh::from_f80(p[1]);
    let t2 = bh::from_f80(p[0]);
    let t3 = ld(frame.matrices.view_matrix[0].bits());
    let t4 = t3 * t2;
    let t5 = ld(frame.matrices.view_matrix[4].bits());
    let t6 = t5 * t1;
    let t7 = ld(frame.matrices.view_matrix[8].bits());
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + ld(frame.matrices.view_matrix[12].bits());
    let t12 = ld(frame.matrices.view_matrix[1].bits());
    let t13 = t12 * t2;
    let t14 = ld(frame.matrices.view_matrix[5].bits());
    let t15 = t14 * t1;
    let t16 = ld(frame.matrices.view_matrix[9].bits());
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + ld(frame.matrices.view_matrix[13].bits());
    let t21 = ld(frame.matrices.view_matrix[2].bits());
    let t22 = t2 * t21;
    let t23 = ld(frame.matrices.view_matrix[6].bits());
    let t24 = t1 * t23;
    let t25 = ld(frame.matrices.view_matrix[10].bits());
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + ld(frame.matrices.view_matrix[14].bits());
    dw.turned[0].set_bits(bh::st(t11));
    dw.turned[1].set_bits(bh::st(t20));
    w.whole.set_bits(bh::to_int(t29) as u32);
    dw.turned[2].set_bits(bh::st(t29));
}

/// `square`'s x87 block as plain Rust on the x87 model (41 instructions), same order and memory places.
fn square_x87(dw: &DrawWork) {
    let t0 = ld(crate::scene::owner::scene().drawing.scene.view_scale.bits());
    let t1 = t0 / bh::ld(dw.turned[2].bits());
    let t2 = bh::ld(dw.turned[2].bits());
    let t3 = t2 * bh::ld(dw.z_k.bits());
    st_cell(&VERTICES.vertices[0].z, t3);
    st_cell(&VERTICES.vertices[1].z, t3);
    st_cell(&VERTICES.vertices[2].z, t3);
    st_cell(&VERTICES.vertices[3].z, t3);
    let t4 = bh::ld(dw.half[0].bits());
    let t5 = t4 * t1;
    dw.half[0].set_bits(bh::st(t5));
    dw.half[1].set_bits(bh::st(t5));
    let t6 = t1 * bh::ld(dw.turned[0].bits());
    let t7 = t1 * bh::ld(dw.turned[1].bits());
    let t8 = -t7;
    let t9 = t8 + ld(crate::scene::owner::scene().drawing.scene.view_centre[1].bits());
    let t10 = t9 - bh::ld(dw.half[1].bits());
    st_cell(&VERTICES.vertices[0].y, t10);
    let t11 = t9 - bh::ld(dw.half[1].bits());
    st_cell(&VERTICES.vertices[1].y, t11);
    let t12 = t9 + bh::ld(dw.half[1].bits());
    st_cell(&VERTICES.vertices[2].y, t12);
    let t13 = t9 + bh::ld(dw.half[1].bits());
    st_cell(&VERTICES.vertices[3].y, t13);
    let t14 = t6 + ld(crate::scene::owner::scene().drawing.scene.view_centre[0].bits());
    let t15 = t14 - bh::ld(dw.half[0].bits());
    st_cell(&VERTICES.vertices[0].x, t15);
    let t16 = t14 + bh::ld(dw.half[0].bits());
    st_cell(&VERTICES.vertices[1].x, t16);
    let t17 = t14 + bh::ld(dw.half[0].bits());
    st_cell(&VERTICES.vertices[2].x, t17);
    let t18 = t14 - bh::ld(dw.half[0].bits());
    st_cell(&VERTICES.vertices[3].x, t18);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn turn_x87(rc: usize, mk: usize, p: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x186999]",
            "fist dword ptr [{mk} + 0x1D0780]",
            "fstp dword ptr [{mk} + 0x18699D]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn square_x87(rc: usize, bf: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x18699D]",
            "fld dword ptr [{mk} + 0x18699D]",
            "fmul dword ptr [{mk} + 0x186A2D]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fst dword ptr [{bf} + 0x5605C8]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
            "fld dword ptr [{mk} + 0x186975]",
            ".byte 0xD8, 0xC9",
            "fst dword ptr [{mk} + 0x186975]",
            "fstp dword ptr [{mk} + 0x186979]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x186995]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x186999]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186979]",
            "fstp dword ptr [{bf} + 0x5605A4]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186979]",
            "fstp dword ptr [{bf} + 0x5605C4]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{mk} + 0x186979]",
            "fstp dword ptr [{bf} + 0x5605E4]",
            "fadd dword ptr [{mk} + 0x186979]",
            "fstp dword ptr [{bf} + 0x560604]",
            "fadd dword ptr [{rc} + 0x22409C]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186975]",
            "fstp dword ptr [{bf} + 0x5605A0]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{mk} + 0x186975]",
            "fstp dword ptr [{bf} + 0x5605C0]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{mk} + 0x186975]",
            "fstp dword ptr [{bf} + 0x5605E0]",
            "fsub dword ptr [{mk} + 0x186975]",
            "fstp dword ptr [{bf} + 0x560600]",
            rc = in(reg) rc,
            bf = in(reg) bf,
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
    use crate::d3d::vertices_store::VERTICES;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential, with_copy};
    use crate::effects::frame_state::DrawWork;
    use crate::effects::ring_store::RingWork;
    use crate::session::Float;

    /// The vertex float at byte `off` of the vertices (x, y or z of a vertex).
    fn vertex_cell(off: usize) -> &'static Float {
        let v = &VERTICES.vertices[off / 32];
        match off % 32 {
            0x0 => &v.x,
            0x4 => &v.y,
            0x8 => &v.z,
            _ => panic!("not a tested vertex field"),
        }
    }

    /// Runs `f` with the vertex floats at `offsets` holding the test buffer's (at `at`)
    /// bytes; what `f` leaves goes back to the buffer, the vertices as they were.
    fn staged<R>(at: usize, offsets: &[usize], f: impl FnOnce() -> R) -> R {
        let host = |off: usize| crate::pointers::host_at(at + off).cast::<u32>();
        let saved: Vec<u32> = offsets.iter().map(|&o| vertex_cell(o).bits()).collect();
        for &o in offsets {
            vertex_cell(o).set_bits(unsafe { host(o).read_unaligned() });
        }
        let out = f();
        for (&o, &old) in offsets.iter().zip(&saved) {
            unsafe { host(o).write_unaligned(vertex_cell(o).bits()) };
            vertex_cell(o).set_bits(old);
        }
        out
    }

    #[test]
    fn turn_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x186931,
                    hi: 0x1D0826,
                    reads: &[],
                    writes: &[
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x18699D, F32),
                        (0x1D0780, I32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x1E,
                    reads: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::turn_x87(b[0], b[1], b[2]),
            |b| {
                with_copy(b[1], RingWork::START, |w| {
                    with_copy::<DrawWork, _>(b[1], DrawWork::START, |dw| {
                        let place: [[u8; 10]; 3] = std::array::from_fn(|n| unsafe {
                            crate::pointers::host_at(b[2] + 10 * n)
                                .cast::<[u8; 10]>()
                                .read_unaligned()
                        });
                        super::turn_x87(&crate::race::owner::race().frame, &place, w, dw);
                    });
                });
            },
        );
    }

    #[test]
    fn square_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x5605A0,
                    hi: 0x56060C,
                    reads: &[],
                    writes: &[
                        (0x5605A0, F32),
                        (0x5605A4, F32),
                        (0x5605A8, F32),
                        (0x5605C0, F32),
                        (0x5605C4, F32),
                        (0x5605C8, F32),
                        (0x5605E0, F32),
                        (0x5605E4, F32),
                        (0x5605E8, F32),
                        (0x560600, F32),
                        (0x560604, F32),
                        (0x560608, F32),
                    ],
                },
                Op::Base {
                    lo: 0x186931,
                    hi: 0x186A31,
                    reads: &[
                        (0x186975, F32),
                        (0x186979, F32),
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x18699D, F32),
                        (0x186A2D, F32),
                    ],
                    writes: &[(0x186975, F32), (0x186979, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::square_x87(b[0], b[1], b[2]),
            |b| {
                with_copy::<DrawWork, _>(b[2], DrawWork::START, |dw| {
                    staged(
                        b[1].wrapping_add(0x5605A0),
                        &[
                            0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                        ],
                        || super::square_x87(dw),
                    );
                });
            },
        );
    }
}
