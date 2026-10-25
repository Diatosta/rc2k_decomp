//! `marks::draw_cell_sprite` (0x4CB796): a ring entry's sprite: the place (st(0) x,
//! st(1) y, st(2) z) turned by `g_viewMatrix` into `flt_8EB8D5` .. (the depth also whole
//! in `dword_9356C0`); unless the depth is at or below `dword_8EB961` (a signed compare of
//! the bits): the four vertices' specular `scene::fog_at` of the depth, colour (ecx's top
//! byte | the entry's +0x90), texture places from cell eax of `g_marksCellPlaces`, depth
//! times `flt_8EB96D`; half the side (edx times `flt_624088` / depth, and that times the
//! cell's `g_marksCornerSlopes`) turned by `g_sines` / `g_cosines`[ebx] into
//! `dword_8EB8BD` ..; the corners about the projected place; `marks::clip_quad`, and unless
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

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::scene::fog_at::fog_at;

const DEPTH_RVA: Slot = Slot::at(0x8EB8DD); // flt_8EB8DD
const WHOLE_RVA: Slot = Slot::at(0x9356C0); // dword_9356C0
const NEAR_RVA: Slot = Slot::at(0x8EB961); // dword_8EB961
const HALF_RVA: Slot = Slot::at(0x8EB8BD); // dword_8EB8BD
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const CELLS_RVA: Slot = Slot::at(0x935766); // g_marksCellPlaces: 0x41 of 16 bytes
const CELLS: u32 = 0x41;
const ANGLES: u32 = 0x1000;

/// The original's 0x4CB796 .. 0x4CB81A: the place turned by the view.
unsafe fn turn(p: &[[u8; 10]; 3]) {
    unsafe {
        turn_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
            p.as_ptr() as usize,
        )
    }
}

/// The original's 0x4CB84C .. 0x4CBA37 (its integer moves done before but those it does
/// between the x87 steps): the square, turned, about the projected place. Gives what the
/// original loads into edi at +0x17B: `flt_8EB8C1` between its two stores.
unsafe fn square(k: u32, s: u32) -> u32 {
    let edi: u32;
    unsafe {
        edi = square_x87(
            crate::race::store::asm_base() as usize,
            crate::race::store::buffers_asm_base() as usize,
            crate::marks::store::base() as usize,
            k as usize,
            s as usize,
        )
    }
    edi
}

/// The Rust version (see the header): `cell` (eax), `angle` (ebx), `alpha` (ecx, its top
/// byte), `size` (edx), `entry` (esi). No caller reads a register it leaves.
pub unsafe fn draw_cell_sprite(
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
    unsafe {
        // The fog's and the clip's refusals, before anything.
        fog_at(0)?;
        if crate::marks::setup::SETTINGS.pointer.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        turn(&place);
        if game_read::<i32>(DEPTH_RVA) <= game_read::<i32>(NEAR_RVA) {
            return Ok(());
        }
        let fog = fog_at(game_read(WHOLE_RVA))?;
        let colour = (alpha & 0xFF00_0000) | ((entry + 0x90) as *const u32).read_unaligned();
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x14, fog);
            game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
        }
        game_write(HALF_RVA, size);
        let at = cell << 4;
        let place = |n: usize| game_read::<u32>(CELLS_RVA + at as usize + 4 * n);
        let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
        for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        square(at, 4 * angle);
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
        game_write(VERTEX_COUNT_RVA, 4u32);
    }
    Ok(())
}

use crate::marks::plain::{ld, ld80, ldi, rd, st, sti, wr};
use crate::math::x87_float as bh;

/// `turn`'s x87 block as plain Rust on the x87 model (37 instructions), same order and memory places.
unsafe fn turn_x87(rc: usize, mk: usize, p: usize) {
    unsafe {
        let t0 = bh::from_f80(ld80(p, 0x14));
        let t1 = bh::from_f80(ld80(p, 0xA));
        let t2 = bh::from_f80(ld80(p, 0));
        let t3 = ld(rc, 0x1EA2CC);
        let t4 = t3 * t2;
        let t5 = ld(rc, 0x1EA2DC);
        let t6 = t5 * t1;
        let t7 = ld(rc, 0x1EA2EC);
        let t8 = t7 * t0;
        let t9 = t6 + t8;
        let t10 = t4 + t9;
        let t11 = t10 + ld(rc, 0x1EA2FC);
        let t12 = ld(rc, 0x1EA2D0);
        let t13 = t12 * t2;
        let t14 = ld(rc, 0x1EA2E0);
        let t15 = t14 * t1;
        let t16 = ld(rc, 0x1EA2F0);
        let t17 = t16 * t0;
        let t18 = t15 + t17;
        let t19 = t13 + t18;
        let t20 = t19 + ld(rc, 0x1EA300);
        let t21 = ld(rc, 0x1EA2D4);
        let t22 = t2 * t21;
        let t23 = ld(rc, 0x1EA2E4);
        let t24 = t1 * t23;
        let t25 = ld(rc, 0x1EA2F4);
        let t26 = t0 * t25;
        let t27 = t26 + t24;
        let t28 = t22 + t27;
        let t29 = t28 + ld(rc, 0x1EA304);
        st(mk, 0x186995, t11);
        st(mk, 0x186999, t20);
        sti(mk, 0x1D0780, t29);
        st(mk, 0x18699D, t29);
    }
}

/// `square`'s x87 block as plain Rust on the x87 model (77 instructions), same order and memory places.
unsafe fn square_x87(rc: usize, bf: usize, mk: usize, k: usize, s: usize) -> u32 {
    unsafe {
        let t0 = ld(rc, 0x224088);
        let t1 = t0 / ld(mk, 0x18699D);
        let t2 = ld(mk, 0x18699D);
        let t3 = t2 * ld(mk, 0x186A2D);
        st(bf, 0x5605A8, t3);
        st(bf, 0x5605C8, t3);
        st(bf, 0x5605E8, t3);
        st(bf, 0x560608, t3);
        let t4 = ldi(mk, 0x18697D);
        let t5 = t4 * t1;
        st(mk, 0x18697D, t5);
        let t6 = t5 * ld(mk, k.wrapping_add(0x1BB7F4));
        st(mk, 0x186981, t6);
        let t7 = rd(mk, 0x18697D);
        let t8 = t7 ^ 0x80000000;
        wr(mk, 0x186989, t8);
        let t9 = rd(mk, 0x186981);
        wr(mk, 0x186985, t9);
        let t10 = ld(mk, 0x18697D);
        let t11 = ld(mk, 0x186981);
        let t12 = t11 * ld(bf, s.wrapping_add(0x624AE4));
        let t13 = t10 * ld(bf, s.wrapping_add(0x623AE4));
        let t14 = t12 + t13;
        st(mk, 0x18697D, t14);
        let t15 = -t11;
        let t16 = t15 * ld(bf, s.wrapping_add(0x623AE4));
        let t17 = t10 * ld(bf, s.wrapping_add(0x624AE4));
        let t18 = t16 + t17;
        st(mk, 0x186981, t18);
        let t19 = ld(mk, 0x186985);
        let t20 = ld(mk, 0x186989);
        let t21 = t20 * ld(bf, s.wrapping_add(0x624AE4));
        let t22 = t19 * ld(bf, s.wrapping_add(0x623AE4));
        let t23 = t21 + t22;
        st(mk, 0x186985, t23);
        let t24 = -t20;
        let t25 = t24 * ld(bf, s.wrapping_add(0x623AE4));
        let t26 = t19 * ld(bf, s.wrapping_add(0x624AE4));
        let t27 = t25 + t26;
        st(mk, 0x186989, t27);
        let t28 = t1 * ld(mk, 0x186995);
        let t29 = t1 * ld(mk, 0x186999);
        let t30 = -t29;
        let t31 = t30 + ld(rc, 0x2240A0);
        let t32 = t31 - ld(mk, 0x186981);
        st(bf, 0x5605A4, t32);
        let t33 = t31 - ld(mk, 0x186989);
        st(bf, 0x5605C4, t33);
        let t34 = t31 + ld(mk, 0x186981);
        st(bf, 0x5605E4, t34);
        let t35 = t31 + ld(mk, 0x186989);
        st(bf, 0x560604, t35);
        let t36 = t28 + ld(rc, 0x22409C);
        let t37 = t36 - ld(mk, 0x18697D);
        st(bf, 0x5605A0, t37);
        let t38 = t36 - ld(mk, 0x186985);
        st(bf, 0x5605C0, t38);
        let t39 = t36 + ld(mk, 0x18697D);
        st(bf, 0x5605E0, t39);
        let t40 = t36 + ld(mk, 0x186985);
        st(bf, 0x560600, t40);
        t9
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn turn_x87(rc: usize, mk: usize, p: usize) {
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
            )
        }
    }

    pub(super) unsafe fn square_x87(rc: usize, bf: usize, mk: usize, k: usize, s: usize) -> u32 {
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
            )
        }
        result
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

    #[test]
    fn turn_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1EA2CC,
                    hi: 0x1EA308,
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
                    lo: 0x186995,
                    hi: 0x1D0784,
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
            |b| unsafe { reference::turn_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::turn_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn square_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2240A4,
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
                    lo: 0x18697D,
                    hi: 0x1BBBF8,
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
            |b| unsafe { reference::square_x87(b[0], b[1], b[2], b[3], b[4]) },
            |b| unsafe { super::square_x87(b[0], b[1], b[2], b[3], b[4]) },
        );
    }
}
