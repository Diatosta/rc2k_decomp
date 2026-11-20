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
use crate::effects::setup::MarksClock;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::{Axis, extended, turn_matrix3};
use crate::memory::session::{PlainBytes, PlainFloat};
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner_frame::Frame;

/// The view's move (the settings' `listener_car_place` less the kept place, times the clock
/// constant), less the last move, plus the last move times the drift's decay.
fn view_moved(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    view_moved_x87(drops, m, &effects.setup.clock);
}

/// The view's move kept for the next frame.
fn move_kept(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    move_kept_x87(drops, m, &effects.setup.clock);
}

pub(super) fn windscreen_drops_drift(
    frame: &Frame,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let m = effects.settings()?;
    let marks = effects.settings()?;
    for item in drops.tilt.point.iter().take(3) {
        item.set(0.0);
    }
    let first: u32 = drops.hex.started.get();
    drops.hex.started.set(1);
    if first != 0 {
        let second: u32 = drops.hex.moved.get();
        drops.hex.moved.set(1);
        if second != 0 {
            view_moved(drops, m, effects);
            let view = &frame.matrices.get().view_place;
            for col in 0..3usize {
                let v = [view[col].get(), view[4 + col].get(), view[8 + col].get()];
                for (k, x) in v.into_iter().enumerate() {
                    drops.hex.turn[3 * col + k].set(x);
                }
            }
            adjugate3(&drops.hex.turn, &drops.hex.adjugate);
            // Two temporaries and the three 80-bit values kept across the two calls.
            let temps = [PlainFloat::new(0.0), PlainFloat::new(0.0)];
            let kept = PlainBytes::new([0u8; 30]);
            drift_turned_x87(drops, &drops.hex.adjugate, &temps, &kept);
            turn_matrix3(Axis::X, &drops.hex.turn, extended(drops.hex.angle.get()))?;
            drift_back_x87(drops, &drops.hex.turn, &temps, &kept);
        }
        move_kept(drops, m, effects);
    }
    for (k, v) in [
        marks.listener_car_place[0].get(),
        marks.listener_car_place[1].get(),
        marks.listener_car_place[2].get(),
    ]
    .into_iter()
    .enumerate()
    {
        drops.hex.kept_place[k].set(v);
    }
    Ok(())
}

use crate::effects::plain::{ld_cell, ld80_at, st_cell, st80_at};
use crate::math::x87_float as bh;

/// `view_moved`'s arithmetic as plain Rust on the x87 model, in the same order.
fn view_moved_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - ld_cell(&drops.hex.kept_place[0]);
    let t2 = t1 * c.reciprocal.ld();
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - ld_cell(&drops.hex.kept_place[1]);
    let t5 = t4 * c.reciprocal.ld();
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - ld_cell(&drops.hex.kept_place[2]);
    let t8 = t7 * c.reciprocal.ld();
    st_cell(&drops.hex.view_move[2], t8);
    st_cell(&drops.hex.view_move[1], t5);
    st_cell(&drops.hex.view_move[0], t2);
    let t9 = ld_cell(&drops.hex.view_move[0]);
    let t10 = t9 - ld_cell(&drops.hex.kept_move[0]);
    let t11 = ld_cell(&drops.hex.view_move[1]);
    let t12 = t11 - ld_cell(&drops.hex.kept_move[1]);
    let t13 = ld_cell(&drops.hex.view_move[2]);
    let t14 = t13 - ld_cell(&drops.hex.kept_move[2]);
    st_cell(&drops.hex.move_change[2], t14);
    st_cell(&drops.hex.move_change[1], t12);
    st_cell(&drops.hex.move_change[0], t10);
    let t15 = ld_cell(&drops.hex.kept_move[0]);
    let t16 = t15 * ld_cell(&drops.hex.decay[0]);
    let t17 = t16 + ld_cell(&drops.hex.move_change[0]);
    st_cell(&drops.hex.move_change[0], t17);
    let t18 = ld_cell(&drops.hex.kept_move[1]);
    let t19 = t18 * ld_cell(&drops.hex.decay[1]);
    let t20 = t19 + ld_cell(&drops.hex.move_change[1]);
    st_cell(&drops.hex.move_change[1], t20);
    let t21 = ld_cell(&drops.hex.kept_move[2]);
    let t22 = t21 * ld_cell(&drops.hex.decay[2]);
    let t23 = t22 + ld_cell(&drops.hex.move_change[2]);
    st_cell(&drops.hex.move_change[2], t23);
}

/// The arithmetic before the matrix turn as plain Rust on the x87 model, in the same order.
fn drift_turned_x87(
    drops: &WindscreenDrops,
    m: &[PlainFloat; 9],
    t: &[PlainFloat; 2],
    l: &PlainBytes<30>,
) {
    let t0 = ld_cell(&drops.hex.move_change[2]);
    let t1 = ld_cell(&drops.hex.move_change[1]);
    let t2 = ld_cell(&drops.hex.move_change[0]);
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    st80_at(l, 0, t19);
    st80_at(l, 10, t18);
    st80_at(l, 20, t17);
}

/// The arithmetic after the matrix turn as plain Rust on the x87 model, in the same order.
fn drift_back_x87(
    drops: &WindscreenDrops,
    m: &[PlainFloat; 9],
    t: &[PlainFloat; 2],
    l: &PlainBytes<30>,
) {
    let t0 = bh::from_f80(ld80_at(l, 20));
    let t1 = bh::from_f80(ld80_at(l, 10));
    let t2 = bh::from_f80(ld80_at(l, 0));
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    let t20 = -t19;
    st_cell(&drops.hex.drift[0], t20);
    let t21 = t20 * ld_cell(&drops.hex.result_k[0]);
    drops.tilt.point[0].st(t21);
    st_cell(&drops.hex.drift[1], t18);
    let t22 = t18 * ld_cell(&drops.hex.result_k[1]);
    drops.tilt.point[1].st(t22);
    st_cell(&drops.hex.drift[2], t17);
    let t23 = t17 * ld_cell(&drops.hex.result_k[2]);
    drops.tilt.point[2].st(t23);
}

/// `move_kept`'s arithmetic as plain Rust on the x87 model, in the same order.
fn move_kept_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - ld_cell(&drops.hex.kept_place[0]);
    let t2 = t1 * c.reciprocal.ld();
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - ld_cell(&drops.hex.kept_place[1]);
    let t5 = t4 * c.reciprocal.ld();
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - ld_cell(&drops.hex.kept_place[2]);
    let t8 = t7 * c.reciprocal.ld();
    st_cell(&drops.hex.kept_move[2], t8);
    st_cell(&drops.hex.kept_move[1], t5);
    st_cell(&drops.hex.kept_move[0], t2);
}

#[cfg(test)]
mod tests;
