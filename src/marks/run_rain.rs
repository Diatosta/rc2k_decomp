//! `marks::run_rain` (0x4C16F9): the rain's frame. `dword_8BC50D` the settings' +0x2764
//! (the drops; none: `dword_8BC405` 0 and back), held to 0x100; the settings' +0x28 ..
//! +0x48 into `dword_8BC441` .., +0x1C .. +0x24 into `dword_8BC435` ..; `marks::place_view`,
//! `scene::depth_buffer_off`, render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1,
//! `scene::clear_texture`; the four vertices' rhw `dword_8C5F91` (through `dword_62400C`),
//! colours 0xFFFFFF, 0, 0x28000000, 0x28FFFFFF, the rest 0, `g_polygonVertexCount` 4;
//! `marks::drift_offsets` when `dword_7652A0` is set; the view's entry (the settings' +8,
//! 12 bytes at `flt_8BC489` and `dword_8BC4C5`): when the settings' +0xC or
//! `dword_7652A0` is set its move since the last frame scaled; the move (`dword_8BC47D`
//! ..) held to `flt_8C5FA5` long when longer than `flt_8C5FA1`; the view's place kept
//! (y plus `dword_8BC505`); `dword_8BC505` the clock's low 20 bits times `flt_8C5FB5`
//! when drifting. Then each drop k from the last: its noise place (`g_marksNoiseA` .. [k]
//! plus the view and the drift, wrapped to 0x40000 and centred) as whole numbers to
//! `marks::draw_streak`, and when the settings' +0x10 is set its turned place kept
//! (`dword_8BD111` .. [k], `fistp`). `dword_8BC405` 1.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D1569). The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables. Refuses null settings and a view entry past the five (the
//! settings' +8), before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::{x87, x87_run};
use crate::scene::vector3::STATUS_RVA;

const DROPS_RVA: Slot = Slot::at(0x8BC50D); // dword_8BC50D
const DRAWN_RVA: Slot = Slot::at(0x8BC405); // dword_8BC405
const MOST_DROPS: i32 = 0x100;
const COPIES_A: Slot = Slot::at(0x8BC441); // dword_8BC441 .. : the settings' +0x28 ..
const COPIES_B: Slot = Slot::at(0x8BC435); // dword_8BC435 .. : the settings' +0x1C ..
const RHW_RVA: Slot = Slot::at(0x8C5F91); // dword_8C5F91
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const DRIFTING_RVA: Slot = Slot::at(0x7652A0); // dword_7652A0
const ENTRIES: u32 = 5;
const ENTRY_RVA: Slot = Slot::at(0x8BC489); // flt_8BC489: 5 of 12 bytes
const LAST_RVA: Slot = Slot::at(0x8BC4C5); // dword_8BC4C5: 5 of 12 bytes
const MOVE_RVA: Slot = Slot::at(0x8BC47D); // dword_8BC47D .. dword_8BC485
const VIEW_RVA: Slot = Slot::at(0x8BDD7D); // dword_8BDD7D .. dword_8BDD85
const DRIFT_RVA: Slot = Slot::at(0x8BC505); // dword_8BC505
const DRIFT_K_RVA: Slot = Slot::at(0x8C5FB5); // flt_8C5FB5
const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const NOISE_RVA: [Slot; 3] = [Slot::at(0x8BC511), Slot::at(0x8BC911), Slot::at(0x8BCD11)];
const SPREAD_RVA: Slot = Slot::at(0x8BC429); // dword_8BC429
const TURNED_RVA: Slot = Slot::at(0x8BDE3D); // flt_8BDE3D .. flt_8BDE45
const KEPT_RVA: [Slot; 3] = [Slot::at(0x8BD111), Slot::at(0x8BD511), Slot::at(0x8BD911)];
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];
const COLOURS: [u32; 4] = [0xFF_FFFF, 0, 0x2800_0000, 0x28FF_FFFF];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The original's 0x4C18E7 .. 0x4C1935: the view's move since last frame, scaled, into its entry.
unsafe fn box_moved(e: u32) {
    unsafe { box_moved_x87(crate::marks::store::base() as usize, e as usize) }
}

/// The original's 0x4C1965 .. 0x4C1996: the move's length into `flt_8BDD5D`, compared with `flt_8C5FA1` (`fstsw` into `g_fpuStatus`).
unsafe fn move_length() {
    unsafe {
        move_length_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
        )
    }
}

/// The original's 0x4C19AC .. 0x4C19F2: the move scaled to `flt_8C5FA5` over its length, times its three factors.
unsafe fn move_held() {
    unsafe { move_held_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4C1A0A .. 0x4C1A1C: the view's y plus the drift (`fistp`) kept in its entry.
unsafe fn keep_y(e: u32) {
    unsafe { keep_y_x87(crate::marks::store::base() as usize, e as usize) }
}

/// The Rust version (see the header).
pub unsafe fn run_rain() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if get(s + 8) >= ENTRIES {
            return Err(BadInput("a rain view entry past the five").into());
        }
        let mut drops = get(s + 0x2764) as i32;
        game_write(DROPS_RVA, drops);
        if drops == 0 {
            game_write(DRAWN_RVA, 0u32);
            return Ok(());
        }
        if drops > MOST_DROPS {
            drops = MOST_DROPS;
            game_write(DROPS_RVA, drops);
        }
        for n in 0..9 {
            game_write(COPIES_A + 4 * n, get(s + 0x28 + 4 * n as u32));
        }
        for n in 0..3 {
            game_write(COPIES_B + 4 * n, get(s + 0x1C + 4 * n as u32));
        }
        crate::marks::place_view::place_view()?;
        crate::scene::depth_buffer_on::depth_buffer(false);
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        // `clear_texture`: what it returns went only to eax, which nothing here reads.
        crate::scene::clear_texture::clear_texture();
        let rhw: u32 = game_read(RHW_RVA);
        for (n, colour) in COLOURS.into_iter().enumerate() {
            let v = VERTICES_RVA + 0x20 * n;
            game_write(SCRATCH_RVA, rhw);
            game_write(v + 0xC, x87::int_to_float(rhw as i32));
            game_write(v + 0x10, colour);
            for o in [0x14, 0x18, 0x1C] {
                game_write(v + o, 0u32);
            }
        }
        game_write(COUNT_RVA, 4u32);
        if game_read::<u32>(DRIFTING_RVA) != 0 {
            crate::marks::drift_offsets::drift_offsets();
        }
        let s: u32 = game_read(SETTINGS_RVA);
        let e = get(s + 8) * 12;
        if get(s + 0xC) != 0 || game_read::<u32>(DRIFTING_RVA) != 0 {
            box_moved(e);
        }
        for n in 0..3 {
            game_write(
                MOVE_RVA + 4 * n,
                game_read::<u32>(ENTRY_RVA + e as usize + 4 * n),
            );
        }
        move_length();
        let status = game_read::<u32>(STATUS_RVA) & 0x700;
        game_write(STATUS_RVA, status);
        if status != 0x100 {
            move_held();
        }
        game_write(LAST_RVA + e as usize, game_read::<u32>(VIEW_RVA));
        keep_y(e);
        game_write(LAST_RVA + e as usize + 8, game_read::<u32>(VIEW_RVA + 8));
        if game_read::<u32>(DRIFTING_RVA) != 0 {
            let clock = game_read::<u32>(CLOCK_RVA) & 0xF_FFFF;
            game_write(
                DRIFT_RVA,
                x87::int_times_to_int(clock as i32, game_read(DRIFT_K_RVA)),
            );
        }
        for k in (0..drops.max(0) as usize).rev() {
            let drift: u32 = game_read(DRIFT_RVA);
            let spread: u32 = game_read(SPREAD_RVA);
            let view = |n: usize| game_read::<u32>(VIEW_RVA + 4 * n);
            let noise = |n: usize| game_read::<u32>(NOISE_RVA[n] + 4 * k);
            let x = noise(0).wrapping_add(view(0)).wrapping_add(spread);
            let y = noise(1).wrapping_add(drift).wrapping_add(view(1));
            let z = noise(2).wrapping_add(view(2)).wrapping_add(spread);
            let [x, y, z] = [x, y, z].map(|v| (v & 0x3_FFFF).wrapping_sub(0x1_FFFF));
            let place = [
                x87_run::int_80(z as i32, false),
                x87_run::int_80(y as i32, false),
                x87_run::int_80(x as i32, false),
            ];
            crate::marks::draw_streak::draw_streak(place)?;
            let s: u32 = game_read(SETTINGS_RVA);
            if get(s + 0x10) != 0 {
                for (n, kept) in KEPT_RVA.into_iter().enumerate() {
                    let v: u32 = game_read(TURNED_RVA + 4 * n);
                    game_write(kept + 4 * k, x87::to_int(v));
                }
            }
        }
        game_write(DRAWN_RVA, 1u32);
    }
    Ok(())
}

use crate::marks::plain::{ld, ldi, st, st16, sti};
use crate::math::x87_float as bh;

/// `box_moved`'s x87 block as plain Rust on the x87 model (13 instructions), same order and memory places.
unsafe fn box_moved_x87(mk: usize, e: usize) {
    unsafe {
        let t0 = ldi(mk, 0x158E3D);
        let t1 = t0 - ldi(mk, e.wrapping_add(0x157585));
        let t2 = t1 * ld(mk, 0x161055);
        st(mk, e.wrapping_add(0x157549), t2);
        let t3 = ldi(mk, 0x158E41);
        let t4 = t3 + ldi(mk, 0x1575C5);
        let t5 = t4 - ldi(mk, e.wrapping_add(0x157589));
        let t6 = t5 * ld(mk, 0x161059);
        st(mk, e.wrapping_add(0x15754D), t6);
        let t7 = ldi(mk, 0x158E45);
        let t8 = t7 - ldi(mk, e.wrapping_add(0x15758D));
        let t9 = t8 * ld(mk, 0x16105D);
        st(mk, e.wrapping_add(0x157551), t9);
    }
}

/// `move_length`'s x87 block as plain Rust on the x87 model (12 instructions), same order and memory places.
unsafe fn move_length_x87(rc: usize, mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x15753D);
        let t1 = t0 * t0;
        let t2 = ld(mk, 0x157541);
        let t3 = t2 * t2;
        let t4 = ld(mk, 0x157545);
        let t5 = t4 * t4;
        let t6 = t3 + t5;
        let t7 = t1 + t6;
        let t8 = t7.sqrt();
        st(mk, 0x158E1D, t8);
        st16(rc, 0x31C814, bh::compare(t8, ld(mk, 0x161061)));
    }
}

/// `move_held`'s x87 block as plain Rust on the x87 model (13 instructions), same order and memory places.
unsafe fn move_held_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x158E1D);
        let t1 = ld(mk, 0x161065) / t0;
        let t2 = t1 * ld(mk, 0x15753D);
        let t3 = t2 * ld(mk, 0x161069);
        st(mk, 0x15753D, t3);
        let t4 = t1 * ld(mk, 0x157541);
        let t5 = t4 * ld(mk, 0x16106D);
        st(mk, 0x157541, t5);
        let t6 = t1 * ld(mk, 0x157545);
        let t7 = t6 * ld(mk, 0x161071);
        st(mk, 0x157545, t7);
    }
}

/// `keep_y`'s x87 block as plain Rust on the x87 model (3 instructions), same order and memory places.
unsafe fn keep_y_x87(mk: usize, e: usize) {
    unsafe {
        let t0 = ldi(mk, 0x158E41);
        let t1 = t0 + ldi(mk, 0x1575C5);
        sti(mk, e.wrapping_add(0x157589), t1);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn box_moved_x87(mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{mk} + 0x158E3D]",
            "fisub dword ptr [{mk} + {e} + 0x157585]",
            "fmul dword ptr [{mk} + 0x161055]",
            "fstp dword ptr [{mk} + {e} + 0x157549]",
            "fild dword ptr [{mk} + 0x158E41]",
            "fiadd dword ptr [{mk} + 0x1575C5]",
            "fisub dword ptr [{mk} + {e} + 0x157589]",
            "fmul dword ptr [{mk} + 0x161059]",
            "fstp dword ptr [{mk} + {e} + 0x15754D]",
            "fild dword ptr [{mk} + 0x158E45]",
            "fisub dword ptr [{mk} + {e} + 0x15758D]",
            "fmul dword ptr [{mk} + 0x16105D]",
            "fstp dword ptr [{mk} + {e} + 0x157551]",
            mk = in(reg) mk,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn move_length_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x15753D]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x157541]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x157545]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fst dword ptr [{mk} + 0x158E1D]",
            "fcomp dword ptr [{mk} + 0x161061]",
            "fstsw word ptr [{rc} + 0x31C814]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn move_held_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x158E1D]",
            "fdivr dword ptr [{mk} + 0x161065]",
            ".byte 0xD9, 0xC0",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x15753D]",
            "fmul dword ptr [{mk} + 0x161069]",
            "fstp dword ptr [{mk} + 0x15753D]",
            "fmul dword ptr [{mk} + 0x157541]",
            "fmul dword ptr [{mk} + 0x16106D]",
            "fstp dword ptr [{mk} + 0x157541]",
            "fmul dword ptr [{mk} + 0x157545]",
            "fmul dword ptr [{mk} + 0x161071]",
            "fstp dword ptr [{mk} + 0x157545]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn keep_y_x87(mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{mk} + 0x158E41]",
            "fiadd dword ptr [{mk} + 0x1575C5]",
            "fistp dword ptr [{mk} + {e} + 0x157589]",
            mk = in(reg) mk,
            e = in(reg) e,
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

    #[test]
    fn box_moved_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x157549,
                    hi: 0x161061,
                    reads: &[
                        (0x157585, I32),
                        (0x157589, I32),
                        (0x15758D, I32),
                        (0x157591, I32),
                        (0x157595, I32),
                        (0x157599, I32),
                        (0x15759D, I32),
                        (0x1575A1, I32),
                        (0x1575A5, I32),
                        (0x1575C1, I32),
                        (0x1575C5, I32),
                        (0x1575C9, I32),
                        (0x1575FD, I32),
                        (0x157601, I32),
                        (0x157605, I32),
                        (0x158E3D, I32),
                        (0x158E41, I32),
                        (0x158E45, I32),
                        (0x161055, F32),
                        (0x161059, F32),
                        (0x16105D, F32),
                    ],
                    writes: &[
                        (0x157549, F32),
                        (0x15754D, F32),
                        (0x157551, F32),
                        (0x157555, F32),
                        (0x157559, F32),
                        (0x15755D, F32),
                        (0x157561, F32),
                        (0x157565, F32),
                        (0x157569, F32),
                        (0x157585, F32),
                        (0x157589, F32),
                        (0x15758D, F32),
                        (0x1575C1, F32),
                        (0x1575C5, F32),
                        (0x1575C9, F32),
                    ],
                },
                Op::Index(&[0, 12, 24, 60, 120]),
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::box_moved_x87(b[0], b[1]) },
            |b| unsafe { super::box_moved_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn move_length_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x31C814,
                    hi: 0x31C816,
                    reads: &[],
                    writes: &[(0x31C814, Status)],
                },
                Op::Base {
                    lo: 0x15753D,
                    hi: 0x161065,
                    reads: &[
                        (0x15753D, F32),
                        (0x157541, F32),
                        (0x157545, F32),
                        (0x161061, F32),
                    ],
                    writes: &[(0x158E1D, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::move_length_x87(b[0], b[1]) },
            |b| unsafe { super::move_length_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn move_held_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x15753D,
                hi: 0x161075,
                reads: &[
                    (0x15753D, F32),
                    (0x157541, F32),
                    (0x157545, F32),
                    (0x158E1D, F32),
                    (0x161065, F32),
                    (0x161069, F32),
                    (0x16106D, F32),
                    (0x161071, F32),
                ],
                writes: &[(0x15753D, F32), (0x157541, F32), (0x157545, F32)],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::move_held_x87(b[0]) },
            |b| unsafe { super::move_held_x87(b[0]) },
        );
    }

    #[test]
    fn keep_y_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x157589,
                    hi: 0x158E45,
                    reads: &[(0x1575C5, I32), (0x158E41, I32)],
                    writes: &[
                        (0x157589, I32),
                        (0x157595, I32),
                        (0x1575A1, I32),
                        (0x1575C5, I32),
                        (0x157601, I32),
                    ],
                },
                Op::Index(&[0, 12, 24, 60, 120]),
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::keep_y_x87(b[0], b[1]) },
            |b| unsafe { super::keep_y_x87(b[0], b[1]) },
        );
    }
}
