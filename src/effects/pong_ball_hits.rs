//! `effects::pong_ball_hits`: when the pong ball overlaps the bat (`effects::pong`'s four
//! compares): a hit (`PongState.hits` plus 1), four more steps a frame (`PongState.steps`), the
//! ball a step back, and its new way: from the bat's centre to the ball's
//! (`PongState.ball_centre[0]` .. `PongState.difference[1]`; an across of exactly 0 bits less
//! `PongRules.nudge`), made one long times `MarksFloats.one`, times `PongRules.way_scale[0]` /
//! `PongRules.way_scale[1]`.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`).

use crate::effects::pong::{self};
use crate::effects::pong_store::{Pong, PongRules, PongState};
use crate::math::cell_x87::CellX87;
use crate::math::x87;

/// The centres and their difference.
fn centres(pong: &Pong) {
    centres_x87(&pong.state, &pong.rules);
}

/// The difference made one long, scaled.
fn new_way(pong: &Pong, effects: &crate::effects::owner::Effects) {
    new_way_x87(&pong.state, &pong.rules, effects.setup.floats.one.get());
}

pub fn pong_ball_hits(pong: &Pong, effects: &crate::effects::owner::Effects) {
    let (over, _) = pong::overlap(pong);
    if !over {
        return;
    }
    let p = &pong.state;
    p.hits.set(p.hits.get().wrapping_add(1));
    p.steps.set(p.steps.get().wrapping_add(4));
    for n in 0..2 {
        p.ball[n].set(x87::sub(&p.ball[n], &p.ball_way[n]));
    }
    centres(pong);
    let across = p.difference[0].get();
    if across == 0.0 && across.is_sign_positive() {
        p.difference[0].set(x87::sub(&p.difference[0], &pong.rules.nudge));
    }
    new_way(pong, effects);
    p.ball_way[0].set(p.difference[0].get());
    p.ball_way[1].set(p.difference[1].get());
}

use crate::math::x87_float::ld;

/// `centres`'s arithmetic as plain Rust on the x87 model, in the same order.
fn centres_x87(p: &PongState, r: &PongRules) {
    let t0 = p.ball_size[0].ld();
    let t1 = t0 * r.centre[0].ld();
    let t2 = t1 + p.ball[0].ld();
    p.ball_centre[0].st(t2);
    let t3 = p.ball_size[1].ld();
    let t4 = t3 * r.centre[1].ld();
    let t5 = t4 + p.ball[1].ld();
    p.ball_centre[1].st(t5);
    let t6 = p.bat_size[0].ld();
    let t7 = t6 * r.centre[2].ld();
    let t8 = t7 + p.bat[0].ld();
    p.bat_centre[0].st(t8);
    let t9 = p.bat_size[1].ld();
    let t10 = t9 * r.centre[3].ld();
    let t11 = t10 + p.bat[1].ld();
    p.bat_centre[1].st(t11);
    let t12 = p.ball_centre[0].ld();
    let t13 = t12 - p.bat_centre[0].ld();
    p.difference[0].st(t13);
    let t14 = p.ball_centre[1].ld();
    let t15 = t14 - p.bat_centre[1].ld();
    p.difference[1].st(t15);
}

/// `new_way`'s x87 block as plain Rust on the x87 model (19 instructions), same order and
/// memory places; `length` the bits of `MarksFloats.one`.
fn new_way_x87(p: &PongState, r: &PongRules, length: f32) {
    let t0 = p.difference[0].ld();
    let t1 = t0 * t0;
    let t2 = p.difference[1].ld();
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    let t6 = ld(length) / t5;
    let t7 = p.difference[0].ld();
    let t8 = t7 * t6;
    p.difference[0].st(t8);
    let t9 = p.difference[1].ld();
    let t10 = t6 * t9;
    p.difference[1].st(t10);
    let t11 = p.difference[0].ld();
    let t12 = t11 * r.way_scale[0].ld();
    p.difference[0].st(t12);
    let t13 = p.difference[1].ld();
    let t14 = t13 * r.way_scale[1].ld();
    p.difference[1].st(t14);
}

#[cfg(test)]
mod tests;
