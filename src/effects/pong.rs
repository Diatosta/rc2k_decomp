//! The pong game over the race (a reading: cheat bit 0x8000 and `dword_641678`. The ball at
//! `flt_768688` (x, y), its way `flt_768694`, its size `flt_7686A0`; the bat at
//! `flt_7686B8`, its size `flt_7686D0`, its place before the move `dword_7686AC`; the hits
//! `dword_7687D0`, the best `dword_7687CC`, the ball's steps a frame `g_pongSteps` (named
//! before the game was seen), the score's place `dword_7687D4` / `dword_7687D8`, the
//! digits' 4 x 5 patterns `unk_768704` (0x14 each).

use crate::framework::rva;
use crate::session::{Bytes, Dword, Float};

/// The pong game over the race: the box scratch, the ball, the bat, the score (IDA 0x768658 .. 0x7687DC, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct PongState {
    /// 0x768658: `flt_768658` .. `flt_768684`: the box drawn last (`effects::draw_pong_box`): the offsets of its corners from the place, twice (a reading: six, then the same plus the second shift).
    pub box_sides: [Float; 12],
    /// 0x768688: `g_pongBall`, `flt_76868C`: the ball's place (x, y).
    pub ball: [Float; 2],
    /// 0x768690: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_768690: Bytes<4>,
    /// 0x768694: `flt_768694`, `flt_768698`: the ball's way, added to its place at each step (starts 1, 1).
    pub ball_way: [Float; 2],
    /// 0x76869C: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_76869c: Bytes<4>,
    /// 0x7686A0: `flt_7686A0`, `flt_7686A4`: the ball's size (width, height).
    pub ball_size: [Float; 2],
    /// 0x7686A8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686a8: Bytes<4>,
    /// 0x7686AC: `dword_7686AC`, `dword_7686B0`: the bat's place before the move (kept as bits; put back when the bat overlaps the ball).
    pub saved_bat: [Dword; 2],
    /// 0x7686B4: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686b4: Bytes<4>,
    /// 0x7686B8: `g_pongBat`, `flt_7686BC`: the bat's place (x, y).
    pub bat: [Float; 2],
    /// 0x7686C0: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_7686c0: Bytes<16>,
    /// 0x7686D0: `flt_7686D0`, `flt_7686D4`: the bat's size.
    pub bat_size: [Float; 2],
    /// 0x7686D8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686d8: Bytes<4>,
    /// 0x7686DC: `flt_7686DC`, `flt_7686E0`: the ball's centre (a reading), worked out by the court's steps.
    pub ball_centre: [Float; 2],
    /// 0x7686E4: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686e4: Bytes<4>,
    /// 0x7686E8: `flt_7686E8`, `flt_7686EC`: the bat's centre (a reading).
    pub bat_centre: [Float; 2],
    /// 0x7686F0: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686f0: Bytes<4>,
    /// 0x7686F4: `flt_7686F4`, `flt_7686F8`: the ball's centre less the bat's (a reading); the way after a hit.
    pub difference: [Float; 2],
    /// 0x7686FC: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686fc: Bytes<4>,
    /// 0x768700: `g_pongSteps`: the ball's steps a frame (5 at the start, 4 more with each hit).
    pub steps: Dword,
    /// 0x768704: `g_pongDigitPatterns`: the digits' 4 x 5 patterns, 10 of 0x14 bytes (a byte a cell, 0 for empty).
    pub digit_patterns: Bytes<200>,
    /// 0x7687CC: `g_pongBest`: the best score (hits).
    pub best: Dword,
    /// 0x7687D0: `g_pongHits`: the hits of this game.
    pub hits: Dword,
    /// 0x7687D4: `dword_7687D4`, `dword_7687D8`: where the next score digit is drawn (x, y).
    pub digit_at: [Dword; 2],
}

pub static PONG: PongState = PongState {
    box_sides: [const { Float::from_bits(0x0) }; 12],
    ball: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_768690: Bytes::new([0; 4]),
    ball_way: [Float::from_bits(0x3F800000), Float::from_bits(0x3F800000)],
    _gap_76869c: Bytes::new([0; 4]),
    ball_size: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_7686a8: Bytes::new([0; 4]),
    saved_bat: [Dword::new(0x0), Dword::new(0x0)],
    _gap_7686b4: Bytes::new([0; 4]),
    bat: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_7686c0: Bytes::new([0; 16]),
    bat_size: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_7686d8: Bytes::new([0; 4]),
    ball_centre: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_7686e4: Bytes::new([0; 4]),
    bat_centre: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_7686f0: Bytes::new([0; 4]),
    difference: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_7686fc: Bytes::new([0; 4]),
    steps: Dword::new(0x0),
    digit_patterns: Bytes::new([
        0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01, 0x01, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x00,
        0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x01,
        0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01, 0x01, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x00, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x01, 0x00, 0x00,
    ]),
    best: Dword::new(0x0),
    hits: Dword::new(0x0),
    digit_at: [Dword::new(0x0), Dword::new(0x0)],
};

impl PongState {
    pub const START: usize = 0x768658;
    pub const LEN: usize = 0x184;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const PONG) as usize
    }
}

/// The pong game's constants (IDA 0x769182 .. 0x769232, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct PongRules {
    /// 0x769182: `flt_769182` .. `flt_76919E`: the court at the start of a game (`Court::reset`), as bits: four (x, y) pairs, the ball, its size, the bat, its size.
    pub start: [Float; 8],
    /// 0x7691A2: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7691a2: Bytes<4>,
    /// 0x7691A6: `flt_7691A6` .. `flt_7691C2`: the court's walls the ball turns at and the places it is put back at (a reading, `effects::move_pong_ball`).
    pub walls: [Float; 8],
    /// 0x7691C6: `flt_7691C6` .. `flt_7691D2`: a half, four times: a place plus its size times it is the centre (`Court` centres).
    pub centre: [Float; 4],
    /// 0x7691D6: `flt_7691D6`: 1, what a hit changes the way by.
    pub nudge: Float,
    /// 0x7691DA: `flt_7691DA`, `flt_7691DE`: 4, 4: the way after a hit is the difference over them (`effects::pong_ball_hits`).
    pub way_scale: [Float; 2],
    /// 0x7691E2: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_7691e2: Bytes<24>,
    /// 0x7691FA: `flt_7691FA`, `flt_7691FE`: 16 and -16: a score cell's step across and down (`effects::pong_cell`).
    pub cell_step: [Float; 2],
    /// 0x769202: `flt_769202`, `flt_769206`: 16, 16: a score cell's size.
    pub cell_size: [Float; 2],
    /// 0x76920A: `dword_76920A` .. `dword_769216`: the texture places of a pong box (`effects::draw_pong_box`; 0.99 four times, as bits).
    pub box_places: [Float; 4],
    /// 0x76921A: `flt_76921A`: 16384: a box whose third corner value is under it is not drawn (a reading).
    pub near: Float,
    /// 0x76921E: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_76921e: Bytes<4>,
    /// 0x769222: `dword_769222` .. `dword_76922E`: the texture places the pong frame's set-up uses (`effects::pong_setup`).
    pub setup_places: [Float; 4],
}

pub static PONG_RULES: PongRules = PongRules {
    start: [
        Float::from_bits(0x44800000),
        Float::from_bits(0x44000000),
        Float::from_bits(0x42800000),
        Float::from_bits(0x42800000),
        Float::from_bits(0x44E00000),
        Float::from_bits(0x43800000),
        Float::from_bits(0x42800000),
        Float::from_bits(0x43800000),
    ],
    _gap_7691a2: Bytes::new([0; 4]),
    walls: [
        Float::from_bits(0x43000000),
        Float::from_bits(0x43000000),
        Float::from_bits(0x45000000),
        Float::from_bits(0x43000000),
        Float::from_bits(0x43000000),
        Float::from_bits(0x43000000),
        Float::from_bits(0x44800000),
        Float::from_bits(0x44800000),
    ],
    centre: [
        Float::from_bits(0x3F000000),
        Float::from_bits(0x3F000000),
        Float::from_bits(0x3F000000),
        Float::from_bits(0x3F000000),
    ],
    nudge: Float::from_bits(0x3F800000),
    way_scale: [Float::from_bits(0x40800000), Float::from_bits(0x40800000)],
    _gap_7691e2: Bytes::new([
        0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x43, 0x00, 0x00, 0x00, 0x43, 0x00, 0x00, 0x00,
        0x42, 0x00, 0x00, 0x80, 0x44, 0x00, 0x00, 0x80, 0x44,
    ]),
    cell_step: [Float::from_bits(0x41800000), Float::from_bits(0xC1800000)],
    cell_size: [Float::from_bits(0x41800000), Float::from_bits(0x41800000)],
    box_places: [
        Float::from_bits(0x3F7D70A4),
        Float::from_bits(0x3F7D70A4),
        Float::from_bits(0x3F7D70A4),
        Float::from_bits(0x3F7D70A4),
    ],
    near: Float::from_bits(0x46800000),
    _gap_76921e: Bytes::new([0x00, 0x00, 0x80, 0x43]),
    setup_places: [
        Float::from_bits(0x3DE80000),
        Float::from_bits(0x3D000000),
        Float::from_bits(0x3E7C0000),
        Float::from_bits(0x3E280000),
    ],
};

impl PongRules {
    pub const START: usize = 0x769182;
    pub const LEN: usize = 0xB0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const PONG_RULES) as usize
    }
}

/// A test buffer standing for the marks' block at `mk` (IDA 0x764F40): the court's state
/// in it, as the x87 blocks' differential tests give it to the typed versions.
#[cfg(all(test, target_arch = "x86"))]
pub fn state_at(mk: usize) -> &'static PongState {
    unsafe {
        &*(crate::pointers::host_at(
            mk.wrapping_add(PongState::START)
                .wrapping_sub(crate::effects::store::START),
        ) as *const PongState)
    }
}

/// The court's constants in the same buffer, copied out (they sit 2 bytes off a four in the
/// image's addresses, so the buffer's copy of them is not aligned for the cells).
#[cfg(all(test, target_arch = "x86"))]
pub fn rules_at(mk: usize) -> PongRules {
    unsafe {
        (crate::pointers::host_at(
            mk.wrapping_add(PongRules::START)
                .wrapping_sub(crate::effects::store::START),
        ) as *const PongRules)
            .read_unaligned()
    }
}

pub const SHOWN_RVA: usize = rva(0x641678); // dword_641678

/// `fld a; fadd b; fcomp c; fstsw ax; and eax, 700h`.
pub fn sum_compare(a: u32, b: u32, c: u32) -> u32 {
    u32::from(sum_status(a, b, c)) & 0x700
}

/// `fld x; fld k; fsub w [; fsub m]; fcompp; fstsw ax; and eax, 700h` (`m` with `Some`).
pub fn less_compare(x: u32, k: u32, w: u32, m: Option<u32>) -> u32 {
    u32::from(less_status(x, k, w, m)) & 0x700
}

/// `fld a; fadd b; fcomp c; fnstsw`: a + b against c, the status word's condition bits.
fn sum_status(a: u32, b: u32, c: u32) -> u16 {
    bh::compare(bh::ld(a) + bh::ld(b), bh::ld(c))
}

/// `fld x; fld k; fsub w [; fsub m]; fcompp; fnstsw`: k - w [- m] against x, the status
/// word's condition bits.
fn less_status(x: u32, k: u32, w: u32, m: Option<u32>) -> u16 {
    let mut t = bh::ld(k) - bh::ld(w);
    if let Some(m) = m {
        t -= bh::ld(m);
    }
    bh::compare(t, bh::ld(x))
}

fn pair(r: &[Float; 2]) -> [f32; 2] {
    r.each_ref().map(Float::get)
}

fn put_pair(r: &[Float; 2], v: [f32; 2]) {
    for (r, v) in r.iter().zip(v) {
        r.set(v);
    }
}

/// The court (`rc2k_core::effects::pong::Court`) from the game's variables.
pub fn court() -> rc2k_core::effects::pong::Court {
    let p = &PONG;
    rc2k_core::effects::pong::Court {
        ball: pair(&p.ball),
        way: pair(&p.ball_way),
        ball_size: pair(&p.ball_size),
        saved_bat: p.saved_bat.each_ref().map(Dword::get),
        bat: pair(&p.bat),
        bat_size: pair(&p.bat_size),
        ball_centre: pair(&p.ball_centre),
        bat_centre: pair(&p.bat_centre),
        difference: pair(&p.difference),
        steps: p.steps.get(),
        hits: p.hits.get(),
    }
}

/// The court back into the game's variables: what differs from `before` (the rest is left
/// as it is, its bits untouched).
pub fn put_court(c: &rc2k_core::effects::pong::Court, before: &rc2k_core::effects::pong::Court) {
    let p = &PONG;
    for (r, v, b) in [
        (&p.ball, c.ball, before.ball),
        (&p.ball_way, c.way, before.way),
        (&p.ball_size, c.ball_size, before.ball_size),
        (&p.bat, c.bat, before.bat),
        (&p.bat_size, c.bat_size, before.bat_size),
        (&p.ball_centre, c.ball_centre, before.ball_centre),
        (&p.bat_centre, c.bat_centre, before.bat_centre),
        (&p.difference, c.difference, before.difference),
    ] {
        if v.map(f32::to_bits) != b.map(f32::to_bits) {
            put_pair(r, v);
        }
    }
    p.saved_bat[0].set(c.saved_bat[0]);
    p.saved_bat[1].set(c.saved_bat[1]);
    p.steps.set(c.steps);
    p.hits.set(c.hits);
}

/// The court's constants from the game.
pub fn rules() -> rc2k_core::effects::pong::Rules {
    let r = &PONG_RULES;
    rc2k_core::effects::pong::Rules {
        centre: r.centre.each_ref().map(Float::get),
        nudge: r.nudge.get(),
        way_scale: pair(&r.way_scale),
        length: f32::from_bits(MARKS_FLOATS.one.bits()),
        walls: r.walls.each_ref().map(Float::get),
        start: r.start.each_ref().map(Float::bits),
    }
}

/// Runs `step` on the court loaded from the game and puts it back.
pub fn with_court<T>(step: impl FnOnce(&mut rc2k_core::effects::pong::Court) -> T) -> T {
    let before = court();
    let mut c = before;
    let out = step(&mut c);
    put_court(&c, &before);
    out
}

/// The four compares of the ball against the bat (`effects::pong_ball_hits`,
/// `effects::pong_bat_back`): whether they overlap (no compare gave C0 alone) and the
/// last status.
pub fn overlap() -> (bool, u32) {
    let p = &PONG;
    let tests = [
        (&p.ball[0], &p.ball_size[0], &p.bat[0]),
        (&p.bat[0], &p.bat_size[0], &p.ball[0]),
        (&p.ball[1], &p.ball_size[1], &p.bat[1]),
        (&p.bat[1], &p.bat_size[1], &p.ball[1]),
    ];
    let mut s = 0;
    for (a, b, c) in tests {
        s = sum_compare(a.bits(), b.bits(), c.bits());
        if s == 0x100 {
            return (false, s);
        }
    }
    (true, s)
}

use crate::effects::tuning::MARKS_FLOATS;
use crate::math::x87_float as bh;

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn sum_compare_x87(v: usize, s: usize) {
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
            );
        }
    }

    pub(super) fn less_compare_x87_1(v: usize, s: usize) {
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
            );
        }
    }

    pub(super) fn less_compare_x87_2(v: usize, s: usize) {
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
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::pointers::{read_or_end, write_or_end};

    /// The float at `base + off` of a test buffer.
    fn at(base: usize, off: usize) -> u32 {
        read_or_end(base.wrapping_add(off) as u32, "a test operand")
    }

    /// The status word written as `fnstsw` does.
    fn put(base: usize, status: u16) {
        write_or_end(base as u32, status, "a test status");
    }

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
            |b| reference::sum_compare_x87(b[0], b[1]),
            |b| {
                put(
                    b[1],
                    super::sum_status(at(b[0], 0), at(b[0], 4), at(b[0], 8)),
                )
            },
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
            |b| reference::less_compare_x87_1(b[0], b[1]),
            |b| {
                let m = Some(at(b[0], 0xC));
                put(
                    b[1],
                    super::less_status(at(b[0], 0), at(b[0], 4), at(b[0], 8), m),
                );
            },
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
            |b| reference::less_compare_x87_2(b[0], b[1]),
            |b| {
                put(
                    b[1],
                    super::less_status(at(b[0], 0), at(b[0], 4), at(b[0], 8), None),
                )
            },
        );
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn bytes<T>(v: &'static T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..core::mem::size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_state_and_the_rules_start_as_the_games() {
        assert_eq!(size_of::<PongState>(), PongState::LEN);
        assert_eq!(size_of::<PongRules>(), PongRules::LEN);
        let (s, r) = (PongState::START, PongRules::START);
        assert!(
            bytes(&PONG) == crate::moved::image_bytes(s, PongState::LEN),
            "PONG starts differently from the image"
        );
        assert!(
            bytes(&PONG_RULES) == crate::moved::image_bytes(r, PongRules::LEN),
            "PONG_RULES starts differently from the image"
        );
        use core::mem::offset_of;
        assert_eq!(offset_of!(PongState, ball), 0x768688 - s);
        assert_eq!(offset_of!(PongState, bat), 0x7686B8 - s);
        assert_eq!(offset_of!(PongState, difference), 0x7686F4 - s);
        assert_eq!(offset_of!(PongState, steps), 0x768700 - s);
        assert_eq!(offset_of!(PongState, digit_patterns), 0x768704 - s);
        assert_eq!(offset_of!(PongState, best), 0x7687CC - s);
        assert_eq!(offset_of!(PongState, digit_at), 0x7687D4 - s);
        assert_eq!(offset_of!(PongRules, walls), 0x7691A6 - r);
        assert_eq!(offset_of!(PongRules, cell_step), 0x7691FA - r);
        assert_eq!(offset_of!(PongRules, near), 0x76921A - r);
        assert_eq!(offset_of!(PongRules, setup_places), 0x769222 - r);
    }

    #[test]
    fn moved_sends_the_addresses_to_the_statics() {
        assert_eq!(crate::moved::place(0x768688), PONG.ball[0].addr() as usize);
        assert_eq!(
            crate::moved::place(0x76921A),
            PONG_RULES.near.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x7687DC),
            (crate::effects::store::STORE.addr() + (0x7687DC - crate::effects::store::START) as u32)
                as usize
        );
    }
}
