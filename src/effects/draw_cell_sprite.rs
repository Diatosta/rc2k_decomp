//! `effects::draw_cell_sprite` (0x4CB796): a ring entry's sprite: the place (st(0) x,
//! st(1) y, st(2) z) turned by `g_viewMatrix` into `flt_8EB8D5` .. (the depth also whole
//! in `dword_9356C0`); unless the depth is at or below `dword_8EB961` (a signed compare of
//! the bits): the four vertices' specular `scene::fog_at` of the depth, colour (ecx's top
//! byte | the entry's +0x90), texture places from cell eax of `g_marksCellPlaces`, depth
//! times `flt_8EB96D`; half the side (edx times `flt_624088` / depth, and that times the
//! cell's `g_marksCornerSlopes`) turned by `g_sines` / `g_cosines`[ebx] into
//! `dword_8EB8BD` ..; the corners about the projected place; `effects::clip_quad`, and unless
//! all is outside `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  st(0) .. st(2) the place (popped), eax the cell, ebx the angle, ecx the alpha (top
//!      byte), edx the size, esi the entry
//! Out: nothing (the original keeps every register but ebp, which no caller reads after it)
//!
//! Callers: the ring's kinds 1, 5, 6, 0xD, 0x13, 0x15, 0x16 (`sub_4CB64D`) and 0x4D0D80.
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the game's variables. Refuses,
//! before anything, a cell past 0x40, an angle of 0x1000 or more, a null entry, and the
//! fog's and the clip's refusals.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::ring_store::{RING_WORK, RingWork};
use crate::effects::surface_store::SURFACE;
use crate::errors::{BadInput, Refused};
use crate::scene::draw_store::DRAW;
use crate::scene::fog_at::fog_at;

use crate::d3d::vertices_store::VERTICES;

const CELLS: u32 = 0x41;
const ANGLES: u32 = 0x1000;

/// The original's 0x4CB796 .. 0x4CB81A: the place turned by the view.
fn turn(p: &[[u8; 10]; 3]) {
    turn_x87(p, &RING_WORK, &DRAW_WORK);
}

/// The original's 0x4CB84C .. 0x4CBA37 (its integer moves done before but those it does
/// between the x87 steps): the square, turned, about the projected place. Gives what the
/// original loads into edi at +0x17B: `flt_8EB8C1` between its two stores.
fn square(k: u32, angle: u32) -> u32 {
    square_x87(k as usize, angle as usize, &DRAW_WORK, &CORNER_SLOPES)
}

/// The Rust version (see the header): `cell` (eax), `angle` (ebx), `alpha` (ecx, its top
/// byte), `size` (edx), `entry` (esi). No caller reads a register it leaves.
pub fn draw_cell_sprite(
    cell: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    entry: u32,
    place: [[u8; 10]; 3],
) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    if angle >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    if entry == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    // The fog's and the clip's refusals, before anything.
    fog_at(0)?;
    if SETTINGS.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    turn(&place);
    if DRAW_WORK.turned[2].bits() as i32 <= DRAW_WORK.near.bits() as i32 {
        return Ok(());
    }
    let fog = fog_at(RING_WORK.whole.get())?;
    let colour = (alpha & 0xFF00_0000) | crate::effects::ring_entries::get(entry, 0x90)?;
    for n in 0..4 {
        VERTICES.vertices[n].specular.set(fog);
        VERTICES.vertices[n].colour.set(colour);
    }
    DRAW_WORK.sprite_half[0].set_bits(size);
    let at = cell << 4;
    let place = |n: usize| SURFACE.places.get_u32(at as usize + 4 * n);
    let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
    for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
        .into_iter()
        .enumerate()
    {
        VERTICES.vertices[n].u.set_bits(u);
        VERTICES.vertices[n].v.set_bits(v);
    }
    square(at, angle);
    if clip_quad()? {
        return Ok(());
    }
    draw_fan();
    DRAW.polygon_vertex_count.set_bits(4u32);
    Ok(())
}

use crate::effects::blocks::{CORNER_SLOPES, CornerSlopes};
use crate::effects::frame_state::{DRAW_WORK, DrawWork};
use crate::effects::plain::{ld_cell, st_cell};
use crate::effects::setup::SETTINGS;
use crate::math::x87_float as bh;
use crate::race::matrices_store::MATRICES;
use crate::scene::engine_tables_store::{cosine, sine};
use crate::session::Float;

/// `turn`'s x87 block as plain Rust on the x87 model (37 instructions), same order and memory places.
fn turn_x87(p: &[[u8; 10]; 3], w: &RingWork, dw: &DrawWork) {
    let t0 = bh::from_f80(p[2]);
    let t1 = bh::from_f80(p[1]);
    let t2 = bh::from_f80(p[0]);
    let t3 = crate::math::x87_float::ld(MATRICES.view_matrix[0].bits());
    let t4 = t3 * t2;
    let t5 = crate::math::x87_float::ld(MATRICES.view_matrix[4].bits());
    let t6 = t5 * t1;
    let t7 = crate::math::x87_float::ld(MATRICES.view_matrix[8].bits());
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + crate::math::x87_float::ld(MATRICES.view_matrix[12].bits());
    let t12 = crate::math::x87_float::ld(MATRICES.view_matrix[1].bits());
    let t13 = t12 * t2;
    let t14 = crate::math::x87_float::ld(MATRICES.view_matrix[5].bits());
    let t15 = t14 * t1;
    let t16 = crate::math::x87_float::ld(MATRICES.view_matrix[9].bits());
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + crate::math::x87_float::ld(MATRICES.view_matrix[13].bits());
    let t21 = crate::math::x87_float::ld(MATRICES.view_matrix[2].bits());
    let t22 = t2 * t21;
    let t23 = crate::math::x87_float::ld(MATRICES.view_matrix[6].bits());
    let t24 = t1 * t23;
    let t25 = crate::math::x87_float::ld(MATRICES.view_matrix[10].bits());
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + crate::math::x87_float::ld(MATRICES.view_matrix[14].bits());
    dw.turned[0].set_bits(bh::st(t11));
    dw.turned[1].set_bits(bh::st(t20));
    w.whole.set_bits(bh::to_int(t29) as u32);
    dw.turned[2].set_bits(bh::st(t29));
}

/// The float at byte `k` of the slope records, whatever field of a record it falls on.
fn slope_at(cs: &CornerSlopes, k: usize) -> &Float {
    let record = &cs.slopes[k / 0x24];
    match (k % 0x24) / 4 {
        0 => &record.slope,
        n => &record.values[n - 1],
    }
}

/// `square`'s x87 block as plain Rust on the x87 model (77 instructions), same order and memory places.
/// `k` is the cell's offset in the cell table (16 bytes a cell), which the original also
/// adds to `g_marksCornerSlopes` (records of 0x24: it reads the float at that byte, whatever
/// field it falls on: section 16), so that read goes through `slope_at`. `angle` is the
/// sine table's cell (the original's `s` is four times it; the cosine is the cell 0x400 on).
fn square_x87(k: usize, angle: usize, dw: &DrawWork, cs: &CornerSlopes) -> u32 {
    let t0 = crate::math::x87_float::ld(DRAW.view_scale.bits());
    let t1 = t0 / bh::ld(dw.turned[2].bits());
    let t2 = bh::ld(dw.turned[2].bits());
    let t3 = t2 * bh::ld(dw.z_k.bits());
    st_cell(&VERTICES.vertices[0].z, t3);
    st_cell(&VERTICES.vertices[1].z, t3);
    st_cell(&VERTICES.vertices[2].z, t3);
    st_cell(&VERTICES.vertices[3].z, t3);
    let t4 = bh::int(dw.sprite_half[0].bits() as i32);
    let t5 = t4 * t1;
    dw.sprite_half[0].set_bits(bh::st(t5));
    let t6 = t5 * ld_cell(slope_at(cs, k));
    dw.sprite_half[1].set_bits(bh::st(t6));
    let t7 = dw.sprite_half[0].bits();
    let t8 = t7 ^ 0x80000000;
    dw.sprite_half[3].set_bits(t8);
    let t9 = dw.sprite_half[1].bits();
    dw.sprite_half[2].set_bits(t9);
    let t10 = bh::ld(dw.sprite_half[0].bits());
    let t11 = bh::ld(dw.sprite_half[1].bits());
    let t12 = t11 * ld_cell(cosine(angle));
    let t13 = t10 * ld_cell(sine(angle));
    let t14 = t12 + t13;
    dw.sprite_half[0].set_bits(bh::st(t14));
    let t15 = -t11;
    let t16 = t15 * ld_cell(sine(angle));
    let t17 = t10 * ld_cell(cosine(angle));
    let t18 = t16 + t17;
    dw.sprite_half[1].set_bits(bh::st(t18));
    let t19 = bh::ld(dw.sprite_half[2].bits());
    let t20 = bh::ld(dw.sprite_half[3].bits());
    let t21 = t20 * ld_cell(cosine(angle));
    let t22 = t19 * ld_cell(sine(angle));
    let t23 = t21 + t22;
    dw.sprite_half[2].set_bits(bh::st(t23));
    let t24 = -t20;
    let t25 = t24 * ld_cell(sine(angle));
    let t26 = t19 * ld_cell(cosine(angle));
    let t27 = t25 + t26;
    dw.sprite_half[3].set_bits(bh::st(t27));
    let t28 = t1 * bh::ld(dw.turned[0].bits());
    let t29 = t1 * bh::ld(dw.turned[1].bits());
    let t30 = -t29;
    let t31 = t30 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    let t32 = t31 - bh::ld(dw.sprite_half[1].bits());
    st_cell(&VERTICES.vertices[0].y, t32);
    let t33 = t31 - bh::ld(dw.sprite_half[3].bits());
    st_cell(&VERTICES.vertices[1].y, t33);
    let t34 = t31 + bh::ld(dw.sprite_half[1].bits());
    st_cell(&VERTICES.vertices[2].y, t34);
    let t35 = t31 + bh::ld(dw.sprite_half[3].bits());
    st_cell(&VERTICES.vertices[3].y, t35);
    let t36 = t28 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    let t37 = t36 - bh::ld(dw.sprite_half[0].bits());
    st_cell(&VERTICES.vertices[0].x, t37);
    let t38 = t36 - bh::ld(dw.sprite_half[2].bits());
    st_cell(&VERTICES.vertices[1].x, t38);
    let t39 = t36 + bh::ld(dw.sprite_half[0].bits());
    st_cell(&VERTICES.vertices[2].x, t39);
    let t40 = t36 + bh::ld(dw.sprite_half[2].bits());
    st_cell(&VERTICES.vertices[3].x, t40);
    t9
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

    pub(super) fn square_x87(rc: usize, bf: usize, mk: usize, k: usize, s: usize) -> u32 {
        let result: u32;
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
            "fild dword ptr [{mk} + 0x18697D]",
            ".byte 0xD8, 0xC9",
            "fst dword ptr [{mk} + 0x18697D]",
            "fmul dword ptr [{mk} + {k} + 0x1BB7F4]",
            "fstp dword ptr [{mk} + 0x186981]",
            "mov {t}, dword ptr [{mk} + 0x18697D]",
            "xor {t}, 0x80000000",
            "mov dword ptr [{mk} + 0x186989], {t}",
            "mov {t}, dword ptr [{mk} + 0x186981]",
            "mov dword ptr [{mk} + 0x186985], {t}",
            "fld dword ptr [{mk} + 0x18697D]",
            "fld dword ptr [{mk} + 0x186981]",
            ".byte 0xD9, 0xC1",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x18697D]",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x186981]",
            "fld dword ptr [{mk} + 0x186985]",
            "fld dword ptr [{mk} + 0x186989]",
            ".byte 0xD9, 0xC1",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x186985]",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x186989]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x186995]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x186999]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186981]",
            "fstp dword ptr [{bf} + 0x5605A4]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186989]",
            "fstp dword ptr [{bf} + 0x5605C4]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{mk} + 0x186981]",
            "fstp dword ptr [{bf} + 0x5605E4]",
            "fadd dword ptr [{mk} + 0x186989]",
            "fstp dword ptr [{bf} + 0x560604]",
            "fadd dword ptr [{rc} + 0x22409C]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x18697D]",
            "fstp dword ptr [{bf} + 0x5605A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186985]",
            "fstp dword ptr [{bf} + 0x5605C0]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{mk} + 0x18697D]",
            "fstp dword ptr [{bf} + 0x5605E0]",
            "fadd dword ptr [{mk} + 0x186985]",
            "fstp dword ptr [{bf} + 0x560600]",
            rc = in(reg) rc,
            bf = in(reg) bf,
            mk = in(reg) mk,
            k = in(reg) k,
            s = in(reg) s,
            t = out(reg) result,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
        result
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential, with_copy};
    use crate::effects::blocks::CornerSlopes;
    use crate::effects::frame_state::DrawWork;
    use crate::effects::ring_store::RingWork;
    use crate::scene::engine_tables_store::{cosine, sine};
    use crate::session::Float;

    /// The vertex float at byte `off` of the vertices (x, y or z of a vertex).
    fn vertex_cell(off: usize) -> &'static Float {
        let v = &crate::d3d::vertices_store::VERTICES.vertices[off / 32];
        match off % 32 {
            0x0 => &v.x,
            0x4 => &v.y,
            0x8 => &v.z,
            _ => panic!("not a tested vertex field"),
        }
    }

    /// Runs `f` with each cell holding the test buffer's dword at its address; what `f`
    /// leaves goes back to the buffer, the cells as they were.
    fn staged<R>(cells: &[(&Float, usize)], f: impl FnOnce() -> R) -> R {
        let host = |a: usize| crate::pointers::host_at(a).cast::<u32>();
        let saved: Vec<u32> = cells.iter().map(|(c, _)| c.bits()).collect();
        for &(c, a) in cells {
            c.set_bits(unsafe { host(a).read_unaligned() });
        }
        let out = f();
        for (&(c, a), &old) in cells.iter().zip(&saved) {
            unsafe { host(a).write_unaligned(c.bits()) };
            c.set_bits(old);
        }
        out
    }

    /// The three ten-byte values at `at` of the test buffer.
    fn values(at: usize) -> [[u8; 10]; 3] {
        std::array::from_fn(|n| {
            std::array::from_fn(|i| unsafe { crate::pointers::host_at(at + 10 * n + i).read() })
        })
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
                        super::turn_x87(&values(b[2]), w, dw);
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
                    hi: 0x628AE4,
                    reads: &[
                        (0x623AE4, F32),
                        (0x623AE8, F32),
                        (0x623B48, F32),
                        (0x624AE4, F32),
                        (0x624AE8, F32),
                        (0x624B48, F32),
                        (0x627AE0, F32),
                        (0x628AE0, F32),
                    ],
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
                    hi: 0x1BDBF4,
                    reads: &[
                        (0x18697D, F32),
                        (0x18697D, I32),
                        (0x18697D, U32),
                        (0x186981, F32),
                        (0x186981, U32),
                        (0x186985, F32),
                        (0x186989, F32),
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x18699D, F32),
                        (0x186A2D, F32),
                        (0x1BB7F4, F32),
                        (0x1BB804, F32),
                        (0x1BB824, F32),
                        (0x1BBBE4, F32),
                        (0x1BBBF4, F32),
                    ],
                    writes: &[
                        (0x18697D, F32),
                        (0x186981, F32),
                        (0x186985, F32),
                        (0x186985, U32),
                        (0x186989, F32),
                        (0x186989, U32),
                    ],
                },
                Op::Index(&[0, 16, 48, 1008, 1024]),
                Op::Index(&[0, 4, 100, 16380]),
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::square_x87(b[0], b[1], b[2], b[3], b[4]),
            |b| {
                with_copy::<DrawWork, _>(b[2], DrawWork::START, |dw| {
                    with_copy::<CornerSlopes, _>(b[2], CornerSlopes::START, |cs| {
                        let vertices: Vec<(&Float, usize)> = [
                            0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                        ]
                        .into_iter()
                        .map(|o| (vertex_cell(o), b[1].wrapping_add(0x5605A0 + o)))
                        .collect();
                        let tables = b[1].wrapping_add(0x623AE4);
                        let mut cells = vertices;
                        cells.push((sine(b[4] / 4), tables.wrapping_add(b[4])));
                        cells.push((cosine(b[4] / 4), tables.wrapping_add(b[4] + 0x1000)));
                        staged(&cells, || super::square_x87(b[3], b[4] / 4, dw, cs))
                    })
                })
            },
        );
    }
}
