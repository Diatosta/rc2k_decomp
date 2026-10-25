//! The pong game over the race (a reading: cheat bit 0x8000 and `dword_641678`. The ball at
//! `flt_768688` (x, y), its way `flt_768694`, its size `flt_7686A0`; the bat at
//! `flt_7686B8`, its size `flt_7686D0`, its place before the move `dword_7686AC`; the hits
//! `dword_7687D0`, the best `dword_7687CC`, the ball's steps a frame `g_pongSteps` (named
//! before the game was seen), the score's place `dword_7687D4` / `dword_7687D8`, the
//! digits' 4 x 5 patterns `unk_768704` (0x14 each).

use crate::framework::rva;
use crate::marks::store::{Slot, game_read, game_write};

pub const BALL: [Slot; 2] = [Slot::at(0x768688), Slot::at(0x76868C)];
pub const BALL_WAY: [Slot; 2] = [Slot::at(0x768694), Slot::at(0x768698)];
pub const BALL_SIZE: [Slot; 2] = [Slot::at(0x7686A0), Slot::at(0x7686A4)];
pub const SAVED_BAT: [Slot; 2] = [Slot::at(0x7686AC), Slot::at(0x7686B0)];
pub const BAT: [Slot; 2] = [Slot::at(0x7686B8), Slot::at(0x7686BC)];
pub const BAT_SIZE: [Slot; 2] = [Slot::at(0x7686D0), Slot::at(0x7686D4)];
pub const STEPS_RVA: Slot = Slot::at(0x768700); // g_pongSteps
pub const BEST_RVA: Slot = Slot::at(0x7687CC); // dword_7687CC
pub const HITS_RVA: Slot = Slot::at(0x7687D0); // dword_7687D0
pub const DIGIT_AT: [Slot; 2] = [Slot::at(0x7687D4), Slot::at(0x7687D8)];
pub const PATTERNS_RVA: Slot = Slot::at(0x768704); // unk_768704: 10 of 0x14
pub const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
pub const SHOWN_RVA: usize = rva(0x641678); // dword_641678

/// `fld a; fadd b; fcomp c; fstsw ax; and eax, 700h`.
pub fn sum_compare(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut s) = ([a, b, c], 0u16);
    unsafe { sum_compare_x87(v.as_ptr() as usize, (&raw mut s) as usize) };
    u32::from(s) & 0x700
}

/// `fld x; fld k; fsub w [; fsub m]; fcompp; fstsw ax; and eax, 700h` (`m` with `Some`).
pub fn less_compare(x: u32, k: u32, w: u32, m: Option<u32>) -> u32 {
    let (v, mut s) = ([x, k, w, m.unwrap_or(0)], 0u16);
    unsafe {
        if m.is_some() {
            less_compare_x87_1(v.as_ptr() as usize, (&raw mut s) as usize)
        } else {
            less_compare_x87_2(v.as_ptr() as usize, (&raw mut s) as usize)
        }
    };
    u32::from(s) & 0x700
}

fn get(r: Slot) -> u32 {
    unsafe { game_read(r) }
}

const CENTRE_RVA: Slot = Slot::at(0x7691C6); // flt_7691C6 .. flt_7691D2
const NUDGE_RVA: Slot = Slot::at(0x7691D6); // flt_7691D6
const WAY_SCALE_RVA: Slot = Slot::at(0x7691DA); // flt_7691DA, flt_7691DE
const LENGTH_RVA: Slot = Slot::at(0x76516C); // flt_76516C
pub const WALLS_RVA: Slot = Slot::at(0x7691A6); // flt_7691A6 .. flt_7691C2
pub const START_RVA: Slot = Slot::at(0x769182); // flt_769182 .. flt_76919E
const BALL_CENTRE: [Slot; 2] = [Slot::at(0x7686DC), Slot::at(0x7686E0)];
const BAT_CENTRE: [Slot; 2] = [Slot::at(0x7686E8), Slot::at(0x7686EC)];
const DIFFERENCE: [Slot; 2] = [Slot::at(0x7686F4), Slot::at(0x7686F8)];

fn pair(r: [Slot; 2]) -> [f32; 2] {
    r.map(|r| f32::from_bits(get(r)))
}

unsafe fn put_pair(r: [Slot; 2], v: [f32; 2]) {
    for (r, v) in r.into_iter().zip(v) {
        unsafe { game_write(r, v.to_bits()) };
    }
}

/// The court (`rc2k_core::marks::pong::Court`) from the game's variables.
pub fn court() -> rc2k_core::marks::pong::Court {
    rc2k_core::marks::pong::Court {
        ball: pair(BALL),
        way: pair(BALL_WAY),
        ball_size: pair(BALL_SIZE),
        saved_bat: SAVED_BAT.map(get),
        bat: pair(BAT),
        bat_size: pair(BAT_SIZE),
        ball_centre: pair(BALL_CENTRE),
        bat_centre: pair(BAT_CENTRE),
        difference: pair(DIFFERENCE),
        steps: get(STEPS_RVA),
        hits: get(HITS_RVA),
    }
}

/// The court back into the game's variables: what differs from `before` (the rest is left
/// as it is, its bits untouched).
pub unsafe fn put_court(c: &rc2k_core::marks::pong::Court, before: &rc2k_core::marks::pong::Court) {
    unsafe {
        for (r, v, b) in [
            (BALL, c.ball, before.ball),
            (BALL_WAY, c.way, before.way),
            (BALL_SIZE, c.ball_size, before.ball_size),
            (BAT, c.bat, before.bat),
            (BAT_SIZE, c.bat_size, before.bat_size),
            (BALL_CENTRE, c.ball_centre, before.ball_centre),
            (BAT_CENTRE, c.bat_centre, before.bat_centre),
            (DIFFERENCE, c.difference, before.difference),
        ] {
            if v.map(f32::to_bits) != b.map(f32::to_bits) {
                put_pair(r, v);
            }
        }
        game_write(SAVED_BAT[0], c.saved_bat[0]);
        game_write(SAVED_BAT[1], c.saved_bat[1]);
        game_write(STEPS_RVA, c.steps);
        game_write(HITS_RVA, c.hits);
    }
}

/// The court's constants from the game.
pub fn rules() -> rc2k_core::marks::pong::Rules {
    let f = |r: Slot| f32::from_bits(get(r));
    rc2k_core::marks::pong::Rules {
        centre: std::array::from_fn(|n| f(CENTRE_RVA + 4 * n)),
        nudge: f(NUDGE_RVA),
        way_scale: [f(WAY_SCALE_RVA), f(WAY_SCALE_RVA + 4)],
        length: f(LENGTH_RVA),
        walls: std::array::from_fn(|n| f(WALLS_RVA + 4 * n)),
        start: std::array::from_fn(|n| get(START_RVA + 4 * n)),
    }
}

/// Runs `step` on the court loaded from the game and puts it back.
pub fn with_court<T>(step: impl FnOnce(&mut rc2k_core::marks::pong::Court) -> T) -> T {
    let before = court();
    let mut c = before;
    let out = step(&mut c);
    unsafe { put_court(&c, &before) };
    out
}

/// The four compares of the ball against the bat (`marks::pong_ball_hits`,
/// `marks::pong_bat_back`): whether they overlap (no compare gave C0 alone) and the
/// last status.
pub fn overlap() -> (bool, u32) {
    let tests = [
        (BALL[0], BALL_SIZE[0], BAT[0]),
        (BAT[0], BAT_SIZE[0], BALL[0]),
        (BALL[1], BALL_SIZE[1], BAT[1]),
        (BAT[1], BAT_SIZE[1], BALL[1]),
    ];
    let mut s = 0;
    for (a, b, c) in tests {
        s = sum_compare(get(a), get(b), get(c));
        if s == 0x100 {
            return (false, s);
        }
    }
    (true, s)
}

use crate::marks::plain::{ld, st16};
use crate::math::x87_float as bh;

/// `sum_compare`'s x87 block as plain Rust on the x87 model (4 instructions), same order and memory places.
unsafe fn sum_compare_x87(v: usize, s: usize) {
    unsafe {
        let t0 = ld(v, 0);
        let t1 = t0 + ld(v, 0x4);
        st16(s, 0, bh::compare(t1, ld(v, 0x8)));
    }
}

/// `less_compare`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
unsafe fn less_compare_x87_1(v: usize, s: usize) {
    unsafe {
        let t0 = ld(v, 0);
        let t1 = ld(v, 0x4);
        let t2 = t1 - ld(v, 0x8);
        let t3 = t2 - ld(v, 0xC);
        st16(s, 0, bh::compare(t3, t0));
    }
}

/// `less_compare`'s x87 block as plain Rust on the x87 model (5 instructions), same order and memory places.
unsafe fn less_compare_x87_2(v: usize, s: usize) {
    unsafe {
        let t0 = ld(v, 0);
        let t1 = ld(v, 0x4);
        let t2 = t1 - ld(v, 0x8);
        st16(s, 0, bh::compare(t2, t0));
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn sum_compare_x87(v: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{v}]",
            "fadd dword ptr [{v} + 4]",
            "fcomp dword ptr [{v} + 8]",
            "fnstsw word ptr [{s}]",
            v = in(reg) v,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn less_compare_x87_1(v: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{v}]",
            "fld dword ptr [{v} + 4]",
            "fsub dword ptr [{v} + 8]",
            "fsub dword ptr [{v} + 12]",
            "fcompp",
            "fnstsw word ptr [{s}]",
            v = in(reg) v,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn less_compare_x87_2(v: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{v}]",
            "fld dword ptr [{v} + 4]",
            "fsub dword ptr [{v} + 8]",
            "fcompp",
            "fnstsw word ptr [{s}]",
            v = in(reg) v,
            s = in(reg) s,
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
    fn sum_compare_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x0,
                    hi: 0xC,
                    reads: &[(0x0, F32), (0x4, F32), (0x8, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x2,
                    reads: &[],
                    writes: &[(0x0, Status)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::sum_compare_x87(b[0], b[1]) },
            |b| unsafe { super::sum_compare_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn less_compare_x87_1_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x0,
                    hi: 0x10,
                    reads: &[(0x0, F32), (0x4, F32), (0x8, F32), (0xC, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x2,
                    reads: &[],
                    writes: &[(0x0, Status)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::less_compare_x87_1(b[0], b[1]) },
            |b| unsafe { super::less_compare_x87_1(b[0], b[1]) },
        );
    }

    #[test]
    fn less_compare_x87_2_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x0,
                    hi: 0xC,
                    reads: &[(0x0, F32), (0x4, F32), (0x8, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x2,
                    reads: &[],
                    writes: &[(0x0, Status)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::less_compare_x87_2(b[0], b[1]) },
            |b| unsafe { super::less_compare_x87_2(b[0], b[1]) },
        );
    }
}
