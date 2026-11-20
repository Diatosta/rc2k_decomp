//! The pong game over the race (a reading: cheat bit 0x8000 and `FrameWork.lead_kept[7]`). The ball
//! at `PongState.ball[0]` (x, y), its way `PongState.ball_way[0]`, its size
//! `PongState.ball_size[0]`; the bat at `PongState.bat[0]`, its size `PongState.bat_size[0]`, its
//! place before the move `PongState.saved_bat[0]`; the hits `PongState.hits`, the best
//! `PongState.best`, the ball's steps a frame `PongState.steps`, the score's place
//! `PongState.digit_at[0]` / `PongState.digit_at[1]`, the digits' 4 x 5 patterns
//! `PongState.digit_patterns` (0x14 each).

use crate::effects::pong_store::Pong;
use crate::memory::session::PlainFloat;

/// The status bits (0x700 mask) of `a + b` compared with `c`.
fn sum_compare(a: f32, b: f32, c: f32) -> u32 {
    u32::from(sum_status(a, b, c)) & 0x700
}

/// The status bits (0x700 mask) of `x - k - w` (less `m` with `Some`) compared.
pub(super) fn less_compare(x: f32, k: f32, w: f32, m: Option<f32>) -> u32 {
    u32::from(less_status(x, k, w, m)) & 0x700
}

/// `a + b` compared with `c`: the status word's condition bits.
fn sum_status(a: f32, b: f32, c: f32) -> u16 {
    bh::compare(bh::ld(a) + bh::ld(b), bh::ld(c))
}

/// `k - w` (less `m` with `Some`) compared with `x`: the status word's condition bits.
fn less_status(x: f32, k: f32, w: f32, m: Option<f32>) -> u16 {
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
        saved_bat: pair(&p.saved_bat).map(f32::to_bits),
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
fn put_court(
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
    put_pair(&p.saved_bat, c.saved_bat.map(f32::from_bits));
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
pub(super) fn with_court<T>(
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
        s = sum_compare(a.get(), b.get(), c.get());
        if s == 0x100 {
            return (false, s);
        }
    }
    (true, s)
}

use crate::math::x87_float as bh;

#[cfg(test)]
mod tests;
