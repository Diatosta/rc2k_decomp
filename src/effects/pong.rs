//! The pong game over the race (a reading: cheat bit 0x8000 and `FrameWork.lead_kept[7]`. The ball at
//! `PongState.ball[0]` (x, y), its way `PongState.ball_way[0]`, its size `PongState.ball_size[0]`; the bat at
//! `PongState.bat[0]`, its size `PongState.bat_size[0]`, its place before the move `PongState.saved_bat[0]`; the hits
//! `PongState.hits`, the best `PongState.best`, the ball's steps a frame `g_pongSteps` (named
//! before the game was seen), the score's place `PongState.digit_at[0]` / `PongState.digit_at[1]`, the
//! digits' 4 x 5 patterns `PongState.digit_patterns` (0x14 each).

use crate::session::{PlainByte, PlainBytes, PlainDword, PlainFloat};

/// The pong game over the race: the box scratch, the ball, the bat, the score (IDA 0x768658 .. 0x7687DC).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PongState {
    /// 0x768658: `flt_768658` .. `flt_768684`: the box drawn last (`effects::draw_pong_box`): the offsets of its corners from the place, twice (a reading: six, then the same plus the second shift).
    pub box_sides: [PlainFloat; 12],
    /// 0x768688: `g_pongBall`, `flt_76868C`: the ball's place (x, y).
    pub ball: [PlainFloat; 2],
    /// 0x768690: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_768690: PlainBytes<4>,
    /// 0x768694: `flt_768694`, `flt_768698`: the ball's way, added to its place at each step (starts 1, 1).
    pub ball_way: [PlainFloat; 2],
    /// 0x76869C: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_76869c: PlainBytes<4>,
    /// 0x7686A0: `flt_7686A0`, `flt_7686A4`: the ball's size (width, height).
    pub ball_size: [PlainFloat; 2],
    /// 0x7686A8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686a8: PlainBytes<4>,
    /// 0x7686AC: `dword_7686AC`, `dword_7686B0`: the bat's place before the move (kept as bits; put back when the bat overlaps the ball).
    pub saved_bat: [PlainDword; 2],
    /// 0x7686B4: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686b4: PlainBytes<4>,
    /// 0x7686B8: `g_pongBat`, `flt_7686BC`: the bat's place (x, y).
    pub bat: [PlainFloat; 2],
    /// 0x7686C0: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_7686c0: PlainBytes<16>,
    /// 0x7686D0: `flt_7686D0`, `flt_7686D4`: the bat's size.
    pub bat_size: [PlainFloat; 2],
    /// 0x7686D8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686d8: PlainBytes<4>,
    /// 0x7686DC: `flt_7686DC`, `flt_7686E0`: the ball's centre, worked out by the court's steps (`effects::pong_ball_hits`).
    pub ball_centre: [PlainFloat; 2],
    /// 0x7686E4: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686e4: PlainBytes<4>,
    /// 0x7686E8: `flt_7686E8`, `flt_7686EC`: the bat's centre (`effects::pong_ball_hits`).
    pub bat_centre: [PlainFloat; 2],
    /// 0x7686F0: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686f0: PlainBytes<4>,
    /// 0x7686F4: `flt_7686F4`, `flt_7686F8`: the ball's centre less the bat's (`effects::pong_ball_hits`); it becomes the ball's way after a hit.
    pub difference: [PlainFloat; 2],
    /// 0x7686FC: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7686fc: PlainBytes<4>,
    /// 0x768700: `g_pongSteps`: the ball's steps a frame (5 at the start, 4 more with each hit).
    pub steps: PlainDword,
    /// 0x768704: `g_pongDigitPatterns`: the digits' 4 x 5 patterns, 10 of 0x14 bytes (a byte a cell, 0 for empty).
    pub digit_patterns: [[PlainByte; 20]; 10],
    /// 0x7687CC: `g_pongBest`: the best score (hits).
    pub best: PlainDword,
    /// 0x7687D0: `g_pongHits`: the hits of this game.
    pub hits: PlainDword,
    /// 0x7687D4: `dword_7687D4`, `dword_7687D8`: where the next score digit is drawn (x, y).
    pub digit_at: [PlainDword; 2],
}

impl PongState {
    pub const fn new() -> Self {
        PongState {
            box_sides: [const { PlainFloat::new(0.0) }; 12],
            ball: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_768690: PlainBytes::new([0; 4]),
            ball_way: [PlainFloat::new(1.0), PlainFloat::new(1.0)],
            _gap_76869c: PlainBytes::new([0; 4]),
            ball_size: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_7686a8: PlainBytes::new([0; 4]),
            saved_bat: [PlainDword::new(0x0), PlainDword::new(0x0)],
            _gap_7686b4: PlainBytes::new([0; 4]),
            bat: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_7686c0: PlainBytes::new([0; 16]),
            bat_size: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_7686d8: PlainBytes::new([0; 4]),
            ball_centre: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_7686e4: PlainBytes::new([0; 4]),
            bat_centre: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_7686f0: PlainBytes::new([0; 4]),
            difference: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_7686fc: PlainBytes::new([0; 4]),
            steps: PlainDword::new(0x0),
            digit_patterns: [
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
                [
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(1),
                    PlainByte::new(1),
                    PlainByte::new(0),
                    PlainByte::new(0),
                ],
            ],
            best: PlainDword::new(0x0),
            hits: PlainDword::new(0x0),
            digit_at: [PlainDword::new(0x0), PlainDword::new(0x0)],
        }
    }

    pub const START: usize = 0x768658;
    pub const LEN: usize = 0x184;
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<PongState>() == PongState::LEN);
    assert!(offset_of!(PongState, digit_patterns) == 0x768704 - PongState::START);
    assert!(offset_of!(PongState, best) == 0x7687CC - PongState::START);
};

/// The pong game's constants (IDA 0x769182 .. 0x769232).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PongRules {
    /// 0x769182: `flt_769182` .. `flt_76919E`: the court at the start of a game (`Court::reset`), as bits: four (x, y) pairs, the ball, its size, the bat, its size.
    pub start: [PlainFloat; 8],
    /// 0x7691A2: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7691a2: PlainBytes<4>,
    /// 0x7691A6: `flt_7691A6` .. `flt_7691C2`: the court's walls the ball turns at and the places it is put back at (a reading, `effects::move_pong_ball`).
    pub walls: [PlainFloat; 8],
    /// 0x7691C6: `flt_7691C6` .. `flt_7691D2`: a half, four times: a place plus its size times it is the centre (`Court` centres).
    pub centre: [PlainFloat; 4],
    /// 0x7691D6: `flt_7691D6`: 1, what a hit changes the way by.
    pub nudge: PlainFloat,
    /// 0x7691DA: `flt_7691DA`, `flt_7691DE`: 4, 4: the way after a hit is the difference over them (`effects::pong_ball_hits`).
    pub way_scale: [PlainFloat; 2],
    /// 0x7691E2: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_7691e2: PlainBytes<24>,
    /// 0x7691FA: `flt_7691FA`, `flt_7691FE`: 16 and -16: a score cell's step across and down (`effects::pong_cell`).
    pub cell_step: [PlainFloat; 2],
    /// 0x769202: `flt_769202`, `flt_769206`: 16, 16: a score cell's size.
    pub cell_size: [PlainFloat; 2],
    /// 0x76920A: `dword_76920A` .. `dword_769216`: the texture places of a pong box (`effects::draw_pong_box`; 0.99 four times, as bits).
    pub box_places: [PlainFloat; 4],
    /// 0x76921A: `flt_76921A`: 16384: `effects::draw_pong_box` draws nothing when the first corner's third value (its depth) compares below it.
    pub near: PlainFloat,
    /// 0x76921E: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_76921e: PlainBytes<4>,
    /// 0x769222: `dword_769222` .. `dword_76922E`: the texture places the pong frame's set-up uses (`effects::pong_setup`).
    pub setup_places: [PlainFloat; 4],
}

impl PongRules {
    pub const fn new() -> Self {
        PongRules {
            start: [
                PlainFloat::new(1024.0),
                PlainFloat::new(512.0),
                PlainFloat::new(64.0),
                PlainFloat::new(64.0),
                PlainFloat::new(1792.0),
                PlainFloat::new(256.0),
                PlainFloat::new(64.0),
                PlainFloat::new(256.0),
            ],
            _gap_7691a2: PlainBytes::new([0; 4]),
            walls: [
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
                PlainFloat::new(2048.0),
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
                PlainFloat::new(128.0),
                PlainFloat::new(1024.0),
                PlainFloat::new(1024.0),
            ],
            centre: [
                PlainFloat::new(0.5),
                PlainFloat::new(0.5),
                PlainFloat::new(0.5),
                PlainFloat::new(0.5),
            ],
            nudge: PlainFloat::new(1.0),
            way_scale: [PlainFloat::new(4.0), PlainFloat::new(4.0)],
            _gap_7691e2: PlainBytes::new([
                0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x43, 0x00, 0x00, 0x00, 0x43, 0x00, 0x00,
                0x00, 0x42, 0x00, 0x00, 0x80, 0x44, 0x00, 0x00, 0x80, 0x44,
            ]),
            cell_step: [PlainFloat::new(16.0), PlainFloat::new(-16.0)],
            cell_size: [PlainFloat::new(16.0), PlainFloat::new(16.0)],
            box_places: [
                PlainFloat::new(0.99),
                PlainFloat::new(0.99),
                PlainFloat::new(0.99),
                PlainFloat::new(0.99),
            ],
            near: PlainFloat::new(16384.0),
            _gap_76921e: PlainBytes::new([0x00, 0x00, 0x80, 0x43]),
            setup_places: [
                PlainFloat::new(0.11328125),
                PlainFloat::new(0.03125),
                PlainFloat::new(0.24609375),
                PlainFloat::new(0.1640625),
            ],
        }
    }

    pub const START: usize = 0x769182;
    pub const LEN: usize = 0xB0;
}

/// The pong game's state and constants, one part of the effects' owner (`effects::owner::Effects`).
pub struct Pong {
    pub state: PongState,
    pub rules: PongRules,
}

impl Pong {
    pub const fn new() -> Self {
        Pong {
            state: PongState::new(),
            rules: PongRules::new(),
        }
    }
}

/// The court's state in the test buffer `mk` standing for the marks' block, as the x87
/// blocks' differential tests give it to the typed versions.
#[cfg(test)]
pub fn state_at(mk: crate::effects::asm_test::Operand) -> &'static PongState {
    unsafe {
        &*(mk
            .at(
                PongState::START.wrapping_sub(crate::effects::store::START),
                size_of::<PongState>(),
            )
            .cast::<PongState>())
    }
}

/// The court's constants in the same buffer, copied out (they sit 2 bytes off a four in the
/// block's layout, so the buffer's copy of them is not aligned for the cells).
#[cfg(test)]
pub fn rules_at(mk: crate::effects::asm_test::Operand) -> PongRules {
    unsafe {
        mk.at(
            PongRules::START.wrapping_sub(crate::effects::store::START),
            size_of::<PongRules>(),
        )
        .cast::<PongRules>()
        .read_unaligned()
    }
}

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

fn pair(r: &[PlainFloat; 2]) -> [f32; 2] {
    r.each_ref().map(PlainFloat::get)
}

fn put_pair(r: &[PlainFloat; 2], v: [f32; 2]) {
    for (r, v) in r.iter().zip(v) {
        r.set(v);
    }
}

/// The court (`rc2k_core::effects::pong::Court`) from the game's variables.
pub fn court(pong: &Pong) -> rc2k_core::effects::pong::Court {
    let p = &pong.state;
    rc2k_core::effects::pong::Court {
        ball: pair(&p.ball),
        way: pair(&p.ball_way),
        ball_size: pair(&p.ball_size),
        saved_bat: p.saved_bat.each_ref().map(PlainDword::get),
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
pub fn put_court(
    pong: &Pong,
    c: &rc2k_core::effects::pong::Court,
    before: &rc2k_core::effects::pong::Court,
) {
    let p = &pong.state;
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
pub fn rules(
    pong: &Pong,
    effects: &crate::effects::owner::Effects,
) -> rc2k_core::effects::pong::Rules {
    let r = &pong.rules;
    rc2k_core::effects::pong::Rules {
        centre: r.centre.each_ref().map(PlainFloat::get),
        nudge: r.nudge.get(),
        way_scale: pair(&r.way_scale),
        length: effects.setup.floats.one.get(),
        walls: r.walls.each_ref().map(PlainFloat::get),
        start: r.start.each_ref().map(PlainFloat::bits),
    }
}

/// Runs `step` on the court loaded from the game and puts it back.
pub fn with_court<T>(
    pong: &Pong,
    step: impl FnOnce(&mut rc2k_core::effects::pong::Court) -> T,
) -> T {
    let before = court(pong);
    let mut c = before;
    let out = step(&mut c);
    put_court(pong, &c, &before);
    out
}

/// The four compares of the ball against the bat (`effects::pong_ball_hits`,
/// `effects::pong_bat_back`): whether they overlap (no compare gave C0 alone) and the
/// last status.
pub fn overlap(pong: &Pong) -> (bool, u32) {
    let p = &pong.state;
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

use crate::math::x87_float as bh;

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Operand, Spec, recorded_writes};
    use crate::scene::x87_mem::Base;

    /// The float at `base + off` of a test buffer.
    fn at(base: Operand, off: usize) -> u32 {
        base.dword(off)
    }

    /// The status word written as `fnstsw` does.
    fn put(base: Operand, status: u16) {
        base.set_word(0, status);
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
        recorded_writes("effects.pong.sum_compare_x87", &SPEC, |b| {
            put(
                b[1],
                super::sum_status(at(b[0], 0), at(b[0], 4), at(b[0], 8)),
            );
        });
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
        recorded_writes("effects.pong.less_compare_x87_1", &SPEC, |b| {
            let m = Some(at(b[0], 0xC));
            put(
                b[1],
                super::less_status(at(b[0], 0), at(b[0], 4), at(b[0], 8), m),
            );
        });
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
        recorded_writes("effects.pong.less_compare_x87_2", &SPEC, |b| {
            put(
                b[1],
                super::less_status(at(b[0], 0), at(b[0], 4), at(b[0], 8), None),
            );
        });
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..core::mem::size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_state_and_the_rules_start_as_the_games() {
        assert_eq!(size_of::<PongState>(), PongState::LEN);
        assert_eq!(size_of::<PongRules>(), PongRules::LEN);
        assert!(
            bytes(&Pong::new().state)
                == crate::test_support::first_values::bytes("effects.owner.EFFECTS.pong.state/388"),
            "PONG starts differently from the image"
        );
        assert!(
            bytes(&Pong::new().rules)
                == crate::test_support::first_values::bytes("effects.owner.EFFECTS.pong.rules/176"),
            "PONG_RULES starts differently from the image"
        );
        use core::mem::offset_of;
        assert_eq!(offset_of!(PongState, ball), 0x30);
        assert_eq!(offset_of!(PongState, bat), 0x60);
        assert_eq!(offset_of!(PongState, difference), 0x9c);
        assert_eq!(offset_of!(PongState, steps), 0xa8);
        assert_eq!(offset_of!(PongState, digit_patterns), 0xac);
        assert_eq!(offset_of!(PongState, best), 0x174);
        assert_eq!(offset_of!(PongState, digit_at), 0x17c);
        assert_eq!(offset_of!(PongRules, walls), 0x24);
        assert_eq!(offset_of!(PongRules, cell_step), 0x78);
        assert_eq!(offset_of!(PongRules, near), 0x98);
        assert_eq!(offset_of!(PongRules, setup_places), 0xa0);
    }
}
