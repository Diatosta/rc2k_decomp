//! `effects::windscreen_flakes_spawn`: new windscreen flakes this frame, under x87 chop rounding
//! (restored at the end). A spawn rate worked out from the spawn angle, the shape tilt and the
//! flakes' `rate` is kept; unless it is below a limit, its whole part (held to 0x80) of
//! `effects::pick_windscreen_flake`s are made, and one more when bits 8 .. 23 of the flakes' random
//! (times 0xC5 plus 0x5D) are at most its fraction times a constant. The x87 run is plain Rust on
//! the x87 model (`math::x87_float`).

use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::effects::windscreen_flakes::pick_windscreen_flake;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner_shared::RaceShared;

const MOST: i32 = 0x80;

/// The rate, compared (its status word goes to the shared FPU status cell).
fn rate(drops: &WindscreenDrops, fl: &WindscreenFlakes, effects: &crate::effects::owner::Effects) {
    rate_x87(
        drops,
        &effects.frame.kept_view.borrow(),
        &mut fl.timing.borrow_mut(),
        &fl.state.borrow(),
        &effects.setup.work,
    );
}

pub(super) fn windscreen_flakes_spawn(
    shared: &RaceShared,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        {
            let mut timing = fl.timing.borrow_mut();
            timing.frames = timing.frames.wrapping_add(1);
        }
        if !crate::effects::spawn_gate::FLAKES.due(setup.clock.borrow().now) {
            return Ok(());
        }
        let old = x87::set_chop();
        rate(drops, fl, effects);
        let status = setup.work.fpu_status.get() & 0x700;
        setup.work.fpu_status.set(status);
        if status != 0x100 {
            let v = fl.timing.borrow().spawn_rate;
            let n = x87::to_int(v).min(MOST);
            for _ in 0..n.max(0) {
                pick_windscreen_flake(shared, fl)?;
            }
            let share = x87::fraction_times_to_int(v, fl.state.borrow().share_k);
            let s = fl
                .state
                .borrow()
                .spawn_random
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            fl.state.borrow_mut().spawn_random = s;
            if ((s >> 8) & 0xFFFF) as i32 <= share {
                pick_windscreen_flake(shared, fl)?;
            }
        }
        x87::set_control(old);
    }
    Ok(())
}

use crate::effects::view_vectors_store::KeptView;
use crate::effects::windscreen_flakes_store::{WindscreenFlakeState, WindscreenFlakeTiming};
use crate::math::x87_float as bh;

/// `rate`'s arithmetic as plain Rust on the x87 model, in the same order.
fn rate_x87(
    drops: &WindscreenDrops,
    kv: &KeptView,
    sti: &mut WindscreenFlakeTiming,
    ss: &WindscreenFlakeState,
    work: &crate::effects::work_store::MarksWork,
) {
    let t0 = bh::ld(kv.first_angle);
    let t1 = t0 + bh::ld(drops.tilt.borrow().tilt);
    let t2 = t1.sin();
    let t3 = t2 * bh::ld(sti.rate);
    sti.spawn_rate = f32::from_bits(bh::st(t3));
    let t4 = bh::ld(sti.rate);
    let t5 = t4 * bh::ld(drops.tilt.borrow().point[2]);
    let t6 = -t5;
    let t7 = bh::ld(kv.first_angle);
    let t8 = t7 + bh::ld(drops.tilt.borrow().tilt);
    let t9 = t8.cos();
    let t10 = t6 * t9;
    let t11 = t10 * bh::ld(ss.rate_constants[0]);
    let t12 = t11 + bh::ld(sti.spawn_rate);
    sti.spawn_rate = f32::from_bits(bh::st(t12));
    let t13 = bh::ld(sti.spawn_rate);
    crate::effects::plain::set_status(work, bh::compare(t13, bh::ld(ss.rate_constants[1])));
}

#[cfg(test)]
mod tests;
