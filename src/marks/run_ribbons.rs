//! `marks::run_ribbons` (0x4CB71A, with its chunk 0x4CF85E .. 0x4CFAAF): the marks' ring
//! walked for its streaks (kind 0x12) when the settings' +0x60 is set: each entry of
//! `g_marksRingCount` from `g_marksRingCursor` along +4; a streak's life (+0x84) less
//! `g_marksFrameTicks` (dies when that borrows, or when its y is below its ground +0x28:
//! `fcomp`); a new one (x 0: no place yet) has its place turned by the view into its last
//! (+0x1C ..) and is moved once; then moved (the motion's y gains `flt_93A572` a tick), its
//! last place into `dword_8EB8E1` .., its place turned into `flt_8EB8D5` .. (kept as the
//! last when the settings' +0xC is set or +0x10 is 1); the first vertex's specular
//! `scene::fog_at` of the last depth's bits; `marks::draw_ribbon` (cell 0x20, width 0x800,
//! the alpha the colour's index, the colour `dword_939DB2`[life times +0x78], or
//! `dword_939BB2`[..] when `dword_765238` is set). A dead entry is moved to just before
//! the cursor (or that moved on past it).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D1640 .., after `sub_4CB64D`). The x87 blocks are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the entries and the game's variables; the fog its Rust
//! version. Refuses, before anything (the walk simulated), null settings, a null entry or
//! link on the way, and a colour index out of its table.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::emitters::{RING_COUNT_RVA, RING_CURSOR_RVA};
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::fog_at::fog_at;

const TICKS_RVA: Slot = Slot::at(0x764FD0); // g_marksFrameTicks
const STREAK: u32 = 0x12;
const LAST_RVA: Slot = Slot::at(0x8EB8E1); // dword_8EB8E1 .. dword_8EB8E9
const PLACE_RVA: Slot = Slot::at(0x8EB8D5); // flt_8EB8D5 .. flt_8EB8DD
const SPECULAR_RVA: usize = rva(0x9605B4); // the first vertex's specular
const INDEX_RVA: Slot = Slot::at(0x9356C0); // dword_9356C0
const OTHER_RVA: Slot = Slot::at(0x765238); // dword_765238
const COLOURS: [(Slot, i32); 2] = [(Slot::at(0x939DB2), 257), (Slot::at(0x939BB2), 128)];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The original's 0x4CF890 .. 0x4CF94D: a new streak's place turned into its last (+0x1C ..), and a first move.
unsafe fn first_frame(e: u32) {
    unsafe {
        first_frame_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
            e as usize,
        )
    }
}

/// The original's 0x4CF94D .. 0x4CF98C: the place moved by the motion, the motion's y by `flt_93A572`.
unsafe fn move_(e: u32) {
    unsafe { move_x87(crate::marks::store::base() as usize, e as usize) }
}

/// The original's 0x4CF9A6 .. 0x4CFA2D: the place turned by the view into `flt_8EB8D5` ...
unsafe fn turn_place(e: u32) {
    unsafe {
        turn_place_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
            e as usize,
        )
    }
}

/// The colour table in use and its size.
unsafe fn colours() -> (Slot, i32) {
    unsafe { COLOURS[(game_read::<u32>(OTHER_RVA) != 0) as usize] }
}

/// Why it refuses, if it does: the walk as the original makes it, nothing written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Some("no marks settings");
        }
        let ticks: u32 = game_read(TICKS_RVA);
        let mut e: u32 = game_read(RING_CURSOR_RVA);
        let count = game_read::<u32>(RING_COUNT_RVA) as i32;
        if count > 0 && (e == 0 || get(e) == 0) {
            return Some("a null ring entry");
        }
        for _ in 0..count.max(0) {
            if e == 0 || get(e) == 0 || get(e + 4) == 0 {
                return Some("a null ring entry or link");
            }
            if get(s + 0x60) != 0 && get(e + 0xC) == STREAK {
                let life = get(e + 0x84);
                if life >= ticks && x87::compare_status(get(e + 0x14), get(e + 0x28)) != 0x100 {
                    if fog_at(0).is_err() {
                        return Some("the fog table is not there");
                    }
                    let i = x87::int_times_to_int(life.wrapping_sub(ticks) as i32, get(e + 0x78));
                    if !(0..colours().1).contains(&i) {
                        return Some("a streak's colour index past its table");
                    }
                }
            }
            e = get(e + 4);
        }
        None
    }
}

/// The streak step at e (see the header): whether the entry dies.
unsafe fn streak(e: u32) -> Result<bool, Refused> {
    unsafe {
        let t: u32 = game_read(TICKS_RVA);
        let life = get(e + 0x84);
        put(e + 0x84, life.wrapping_sub(t));
        if life < t {
            return Ok(true);
        }
        if x87::compare_status(get(e + 0x14), get(e + 0x28)) == 0x100 {
            return Ok(true);
        }
        if get(e + 0x10) == 0 {
            first_frame(e);
        }
        move_on(e);
        game_write(LAST_RVA, get(e + 0x1C));
        game_write(LAST_RVA + 4, get(e + 0x20));
        game_write(LAST_RVA + 8, get(e + 0x24));
        turn_place(e);
        let s: u32 = game_read(SETTINGS_RVA);
        if get(s + 0xC) != 0 || get(s + 0x10) == 1 {
            put(e + 0x1C, game_read(PLACE_RVA));
            put(e + 0x20, game_read(PLACE_RVA + 4));
            put(e + 0x24, game_read(PLACE_RVA + 8));
        }
        game_write(SPECULAR_RVA, fog_at(get(e + 0x24))?);
        let i = x87::int_times_to_int(get(e + 0x84) as i32, get(e + 0x78));
        game_write(INDEX_RVA, i);
        let colour = game_read::<u32>(colours().0 + 4 * i as usize);
        crate::marks::draw_ribbon::draw_ribbon(0x20, i as u32, 0x800, colour)?;
    }
    Ok(false)
}

unsafe fn move_on(e: u32) {
    unsafe { move_(e) }
}

/// The Rust version (see the header).
pub unsafe fn run_ribbons() -> Result<(), Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    unsafe {
        let mut e: u32 = game_read(RING_CURSOR_RVA);
        let mut counter = game_read::<u32>(RING_COUNT_RVA) as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let next = get(e + 4);
            let settings: u32 = game_read(SETTINGS_RVA);
            if get(settings + 0x60) != 0 && get(e + 0xC) == STREAK && streak(e)? {
                game_write(
                    RING_COUNT_RVA,
                    game_read::<u32>(RING_COUNT_RVA).wrapping_sub(1),
                );
                if e == game_read::<u32>(RING_CURSOR_RVA) {
                    game_write(RING_CURSOR_RVA, get(e + 4));
                } else {
                    let (before, after) = (get(e), get(e + 4));
                    put(before + 4, after);
                    put(after, before);
                    let cursor: u32 = game_read(RING_CURSOR_RVA);
                    let last = get(cursor);
                    put(cursor, e);
                    put(last + 4, e);
                    put(e, last);
                    put(e + 4, cursor);
                }
            }
            e = next;
        }
    }
    Ok(())
}

use crate::marks::plain::{ld, ldi, st};

/// `first_frame`'s x87 block as plain Rust on the x87 model (52 instructions), same order and memory places.
unsafe fn first_frame_x87(rc: usize, mk: usize, e: usize) {
    unsafe {
        let t0 = ld(e, 0x18);
        let t1 = ld(e, 0x14);
        let t2 = ld(e, 0x10);
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
        st(e, 0x1C, t11);
        st(e, 0x20, t20);
        st(e, 0x24, t29);
        let t30 = ld(e, 0x2C);
        let t31 = t30 * ldi(mk, 0x90);
        let t32 = t31 + ld(e, 0x10);
        st(e, 0x10, t32);
        let t33 = ld(e, 0x30);
        let t34 = t33 * ldi(mk, 0x90);
        let t35 = t34 + ld(e, 0x14);
        st(e, 0x14, t35);
        let t36 = ld(e, 0x34);
        let t37 = t36 * ldi(mk, 0x90);
        let t38 = t37 + ld(e, 0x18);
        st(e, 0x18, t38);
        let t39 = ld(mk, 0x1D562E);
        let t40 = t39 * ldi(mk, 0x90);
        let t41 = t40 + ld(e, 0x30);
        st(e, 0x30, t41);
    }
}

/// `move_`'s x87 block as plain Rust on the x87 model (16 instructions), same order and memory places.
unsafe fn move_x87(mk: usize, e: usize) {
    unsafe {
        let t0 = ld(e, 0x2C);
        let t1 = t0 * ldi(mk, 0x90);
        let t2 = t1 + ld(e, 0x10);
        st(e, 0x10, t2);
        let t3 = ld(e, 0x30);
        let t4 = t3 * ldi(mk, 0x90);
        let t5 = t4 + ld(e, 0x14);
        st(e, 0x14, t5);
        let t6 = ld(e, 0x34);
        let t7 = t6 * ldi(mk, 0x90);
        let t8 = t7 + ld(e, 0x18);
        st(e, 0x18, t8);
        let t9 = ld(mk, 0x1D5632);
        let t10 = t9 * ldi(mk, 0x90);
        let t11 = t10 + ld(e, 0x30);
        st(e, 0x30, t11);
    }
}

/// `turn_place`'s x87 block as plain Rust on the x87 model (36 instructions), same order and memory places.
unsafe fn turn_place_x87(rc: usize, mk: usize, e: usize) {
    unsafe {
        let t0 = ld(e, 0x18);
        let t1 = ld(e, 0x14);
        let t2 = ld(e, 0x10);
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
        st(mk, 0x18699D, t29);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn first_frame_x87(rc: usize, mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
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
            "fstp dword ptr [{e} + 0x1C]",
            "fstp dword ptr [{e} + 0x20]",
            "fstp dword ptr [{e} + 0x24]",
            "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{mk} + 0x1D562E]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn move_x87(mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{mk} + 0x1D5632]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            mk = in(reg) mk,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn turn_place_x87(rc: usize, mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
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
            "fstp dword ptr [{mk} + 0x18699D]",
            rc = in(reg) rc,
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
    fn first_frame_x87_matches() {
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
                    lo: 0x90,
                    hi: 0x1D5632,
                    reads: &[(0x90, I32), (0x1D562E, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x38,
                    reads: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x2C, F32),
                        (0x30, F32),
                        (0x34, F32),
                    ],
                    writes: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x30, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::first_frame_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::first_frame_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn move_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x90,
                    hi: 0x1D5636,
                    reads: &[(0x90, I32), (0x1D5632, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x38,
                    reads: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x2C, F32),
                        (0x30, F32),
                        (0x34, F32),
                    ],
                    writes: &[(0x10, F32), (0x14, F32), (0x18, F32), (0x30, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::move_x87(b[0], b[1]) },
            |b| unsafe { super::move_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn turn_place_x87_matches() {
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
                    hi: 0x1869A1,
                    reads: &[],
                    writes: &[(0x186995, F32), (0x186999, F32), (0x18699D, F32)],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::turn_place_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::turn_place_x87(b[0], b[1], b[2]) },
        );
    }
}
