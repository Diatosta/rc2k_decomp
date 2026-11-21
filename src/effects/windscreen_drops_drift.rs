//! `effects::windscreen_drops_drift`: the windscreen drops' drift from the view's moves. The drift
//! is 0 on the first two calls; after that the view's move (the settings' `listener_car_place` less
//! the kept place, times the clock constant) less the decayed last move is turned by the adjugate of the
//! view's turn and by a turn about x by the drift's angle, then negated into the drops' move and
//! scaled into the drift; the move and place are kept for the next frame.
//!
//! Works on the effects settings.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`) (the three values it keeps on
//! the x87 stack across the two calls kept as 80 bits). Refuses null settings, before anything.

use crate::effects::adjugate3::adjugate3;
use crate::effects::setup_store::MarksClock;
use crate::effects::windscreen_drop_vars_store::{WindscreenDropVars, WindscreenDrops};
use crate::errors::Refused;
use crate::math::matrix3::{Axis, extended, turn_matrix3_plain};
use crate::math::x87_float::{self as bh, stf};
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner_frame::Frame;

pub(super) fn windscreen_drops_drift(
    frame: &Frame,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let m = effects.settings()?;
    let marks = effects.settings()?;
    drops.tilt.borrow_mut().point = [0.0; 3];
    let mut hex = drops.hex.borrow_mut();
    let first: u32 = hex.started;
    hex.started = 1;
    if first != 0 {
        let second: u32 = hex.moved;
        hex.moved = 1;
        if second != 0 {
            view_moved_x87(&mut hex, m, &effects.setup.clock.borrow());
            let view = &frame.view_kept.get().view_place;
            for col in 0..3usize {
                let v = [view[col].get(), view[4 + col].get(), view[8 + col].get()];
                for (k, x) in v.into_iter().enumerate() {
                    hex.turn[3 * col + k] = x;
                }
            }
            hex.adjugate = {
                let mut adjugate = hex.adjugate;
                adjugate3(&hex.turn, &mut adjugate);
                adjugate
            };
            // Two temporaries and the three 80-bit values kept across the two calls.
            let mut temps = [0.0f32; 2];
            let mut kept = [0u8; 30];
            let adjugate = hex.adjugate;
            drift_turned_x87(&hex, &adjugate, &mut temps, &mut kept);
            let angle = hex.angle;
            turn_matrix3_plain(Axis::X, &mut hex.turn, extended(angle))?;
            let turn = hex.turn;
            drift_back_x87(&mut hex, drops, &turn, &mut temps, &kept);
        }
        move_kept_x87(&mut hex, m, &effects.setup.clock.borrow());
    }
    for (k, v) in [
        marks.listener_car_place[0].get(),
        marks.listener_car_place[1].get(),
        marks.listener_car_place[2].get(),
    ]
    .into_iter()
    .enumerate()
    {
        hex.kept_place[k] = v;
    }
    Ok(())
}

/// The view's move (the settings' `listener_car_place` less the kept place, times the clock
/// constant), less the last move, plus the last move times the drift's decay, as plain Rust on
/// the x87 model, in the same order.
fn view_moved_x87(hex: &mut WindscreenDropVars, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - bh::ld(hex.kept_place[0]);
    let t2 = t1 * bh::ld(c.reciprocal);
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - bh::ld(hex.kept_place[1]);
    let t5 = t4 * bh::ld(c.reciprocal);
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - bh::ld(hex.kept_place[2]);
    let t8 = t7 * bh::ld(c.reciprocal);
    hex.view_move[2] = stf(t8);
    hex.view_move[1] = stf(t5);
    hex.view_move[0] = stf(t2);
    let t9 = bh::ld(hex.view_move[0]);
    let t10 = t9 - bh::ld(hex.kept_move[0]);
    let t11 = bh::ld(hex.view_move[1]);
    let t12 = t11 - bh::ld(hex.kept_move[1]);
    let t13 = bh::ld(hex.view_move[2]);
    let t14 = t13 - bh::ld(hex.kept_move[2]);
    hex.move_change[2] = stf(t14);
    hex.move_change[1] = stf(t12);
    hex.move_change[0] = stf(t10);
    let t15 = bh::ld(hex.kept_move[0]);
    let t16 = t15 * bh::ld(hex.decay[0]);
    let t17 = t16 + bh::ld(hex.move_change[0]);
    hex.move_change[0] = stf(t17);
    let t18 = bh::ld(hex.kept_move[1]);
    let t19 = t18 * bh::ld(hex.decay[1]);
    let t20 = t19 + bh::ld(hex.move_change[1]);
    hex.move_change[1] = stf(t20);
    let t21 = bh::ld(hex.kept_move[2]);
    let t22 = t21 * bh::ld(hex.decay[2]);
    let t23 = t22 + bh::ld(hex.move_change[2]);
    hex.move_change[2] = stf(t23);
}

/// The arithmetic before the matrix turn as plain Rust on the x87 model, in the same order.
fn drift_turned_x87(hex: &WindscreenDropVars, m: &[f32; 9], t: &mut [f32; 2], l: &mut [u8; 30]) {
    let t0 = bh::ld(hex.move_change[2]);
    let t1 = bh::ld(hex.move_change[1]);
    let t2 = bh::ld(hex.move_change[0]);
    let t3 = t2 * bh::ld(m[0]);
    let t4 = t1 * bh::ld(m[1]);
    let t5 = t0 * bh::ld(m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    t[0] = stf(t7);
    let t8 = t2 * bh::ld(m[3]);
    let t9 = t1 * bh::ld(m[4]);
    let t10 = t0 * bh::ld(m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    t[1] = stf(t12);
    let t13 = t2 * bh::ld(m[6]);
    let t14 = t1 * bh::ld(m[7]);
    let t15 = t0 * bh::ld(m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = bh::ld(t[1]);
    let t19 = bh::ld(t[0]);
    l[0..10].copy_from_slice(&bh::f80(t19));
    l[10..20].copy_from_slice(&bh::f80(t18));
    l[20..30].copy_from_slice(&bh::f80(t17));
}

/// The arithmetic after the matrix turn as plain Rust on the x87 model, in the same order.
fn drift_back_x87(
    hex: &mut WindscreenDropVars,
    drops: &WindscreenDrops,
    m: &[f32; 9],
    t: &mut [f32; 2],
    l: &[u8; 30],
) {
    let kept = |at: usize| -> [u8; 10] { std::array::from_fn(|i| l[at + i]) };
    let t0 = bh::from_f80(kept(20));
    let t1 = bh::from_f80(kept(10));
    let t2 = bh::from_f80(kept(0));
    let t3 = t2 * bh::ld(m[0]);
    let t4 = t1 * bh::ld(m[1]);
    let t5 = t0 * bh::ld(m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    t[0] = stf(t7);
    let t8 = t2 * bh::ld(m[3]);
    let t9 = t1 * bh::ld(m[4]);
    let t10 = t0 * bh::ld(m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    t[1] = stf(t12);
    let t13 = t2 * bh::ld(m[6]);
    let t14 = t1 * bh::ld(m[7]);
    let t15 = t0 * bh::ld(m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = bh::ld(t[1]);
    let t19 = bh::ld(t[0]);
    let t20 = -t19;
    hex.drift[0] = stf(t20);
    let t21 = t20 * bh::ld(hex.result_k[0]);
    drops.tilt.borrow_mut().point[0] = stf(t21);
    hex.drift[1] = stf(t18);
    let t22 = t18 * bh::ld(hex.result_k[1]);
    drops.tilt.borrow_mut().point[1] = stf(t22);
    hex.drift[2] = stf(t17);
    let t23 = t17 * bh::ld(hex.result_k[2]);
    drops.tilt.borrow_mut().point[2] = stf(t23);
}

/// `move_kept`'s arithmetic as plain Rust on the x87 model, in the same order.
fn move_kept_x87(hex: &mut WindscreenDropVars, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - bh::ld(hex.kept_place[0]);
    let t2 = t1 * bh::ld(c.reciprocal);
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - bh::ld(hex.kept_place[1]);
    let t5 = t4 * bh::ld(c.reciprocal);
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - bh::ld(hex.kept_place[2]);
    let t8 = t7 * bh::ld(c.reciprocal);
    hex.kept_move[2] = stf(t8);
    hex.kept_move[1] = stf(t5);
    hex.kept_move[0] = stf(t2);
}

use crate::effects::plain::ld_cell;

#[cfg(test)]
mod tests;
