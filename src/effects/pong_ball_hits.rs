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
use crate::math::x87;

/// The centres and their difference.
fn centres(pong: &Pong) {
    centres_x87(&mut pong.state.borrow_mut(), &pong.rules);
}

/// The difference made one long, scaled.
fn new_way(pong: &Pong, effects: &crate::effects::owner::Effects) {
    new_way_x87(
        &mut pong.state.borrow_mut(),
        &pong.rules,
        effects.setup.floats.one,
    );
}

pub fn pong_ball_hits(pong: &Pong, effects: &crate::effects::owner::Effects) {
    let (over, _) = pong::overlap(pong);
    if !over {
        return;
    }
    {
        let mut p = pong.state.borrow_mut();
        p.hits = p.hits.wrapping_add(1);
        p.steps = p.steps.wrapping_add(4);
        for n in 0..2 {
            p.ball[n] = x87::sub(p.ball[n], p.ball_way[n]);
        }
    }
    centres(pong);
    let across = pong.state.borrow().difference[0];
    if across == 0.0 && across.is_sign_positive() {
        let mut p = pong.state.borrow_mut();
        p.difference[0] = x87::sub(p.difference[0], pong.rules.nudge);
    }
    new_way(pong, effects);
    let mut p = pong.state.borrow_mut();
    p.ball_way = p.difference;
}

use crate::math::x87_float::{self as bh, ld};

/// `centres`'s arithmetic as plain Rust on the x87 model, in the same order.
fn centres_x87(p: &mut PongState, r: &PongRules) {
    let t0 = bh::ld(p.ball_size[0]);
    let t1 = t0 * ld(r.centre[0]);
    let t2 = t1 + bh::ld(p.ball[0]);
    p.ball_centre[0] = f32::from_bits(bh::st(t2));
    let t3 = bh::ld(p.ball_size[1]);
    let t4 = t3 * ld(r.centre[1]);
    let t5 = t4 + bh::ld(p.ball[1]);
    p.ball_centre[1] = f32::from_bits(bh::st(t5));
    let t6 = bh::ld(p.bat_size[0]);
    let t7 = t6 * ld(r.centre[2]);
    let t8 = t7 + bh::ld(p.bat[0]);
    p.bat_centre[0] = f32::from_bits(bh::st(t8));
    let t9 = bh::ld(p.bat_size[1]);
    let t10 = t9 * ld(r.centre[3]);
    let t11 = t10 + bh::ld(p.bat[1]);
    p.bat_centre[1] = f32::from_bits(bh::st(t11));
    let t12 = bh::ld(p.ball_centre[0]);
    let t13 = t12 - bh::ld(p.bat_centre[0]);
    p.difference[0] = f32::from_bits(bh::st(t13));
    let t14 = bh::ld(p.ball_centre[1]);
    let t15 = t14 - bh::ld(p.bat_centre[1]);
    p.difference[1] = f32::from_bits(bh::st(t15));
}

/// `new_way`'s x87 block as plain Rust on the x87 model (19 instructions), same order and
/// memory places; `length` the bits of `MarksFloats.one`.
fn new_way_x87(p: &mut PongState, r: &PongRules, length: f32) {
    let t0 = bh::ld(p.difference[0]);
    let t1 = t0 * t0;
    let t2 = bh::ld(p.difference[1]);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    let t6 = ld(length) / t5;
    let t7 = bh::ld(p.difference[0]);
    let t8 = t7 * t6;
    p.difference[0] = f32::from_bits(bh::st(t8));
    let t9 = bh::ld(p.difference[1]);
    let t10 = t6 * t9;
    p.difference[1] = f32::from_bits(bh::st(t10));
    let t11 = bh::ld(p.difference[0]);
    let t12 = t11 * ld(r.way_scale[0]);
    p.difference[0] = f32::from_bits(bh::st(t12));
    let t13 = bh::ld(p.difference[1]);
    let t14 = t13 * ld(r.way_scale[1]);
    p.difference[1] = f32::from_bits(bh::st(t14));
}

#[cfg(test)]
mod tests;
