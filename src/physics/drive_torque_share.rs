//! A share of the drive by speed. n is `torque_share_a` (8) with the drive kind 1, else
//! `torque_share_b` (32); s = |car speed| / `torque_share_c` held to at most 1.0 (a lower or
//! unordered value is left); d = (n - the car's `gear_ratio_setting`) / n; `torque_share` = d +
//! (1 - d) s, and `drive_torque` is multiplied by it.

use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87::toward_one;
use crate::math::x87_float as bh;
use crate::physics::car_store::Car;
use crate::physics::figures::{TORQUE_SHARE_A, TORQUE_SHARE_B, TORQUE_SHARE_C};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use std::cmp::Ordering;

/// The drive torque's share of this step: the torque times the share of the gears left
/// above the setting, eased toward 1 by the forward speed's share; returns the gear count
/// less the gear ratio setting word.
pub fn drive_torque_share(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    let (speed_share, gears_share) = (
        crate::memory::session::PlainFloat::new(scene.draw_work().work_float_a),
        crate::memory::session::PlainFloat::new(scene.draw_work().work_float_b),
    );
    // The step's count: stored before it is read, as a whole number.
    let count = crate::memory::session::PlainDword::new(0);
    let n = if car.drive_kind.get() == 1 {
        TORQUE_SHARE_A
    } else {
        TORQUE_SHARE_B
    };
    count.set(n);
    speed_share.set(bh::stf(
        car.local_velocity[2].ld().abs() / bh::int(TORQUE_SHARE_C),
    ));
    let one = numbers.one.get();
    let below = !matches!(
        speed_share.get().partial_cmp(&one),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !below {
        speed_share.set(x87::loaded(one));
    }
    // The gears left go through the cell as a whole number, then their share of the count
    // replaces it.
    let gears_left = count.get().wrapping_sub(car.gear_ratio_setting.get());
    gears_share.set(x87::int_ratio(gears_left as i32, count.get() as i32));
    tables
        .torque_share
        .set(toward_one(gears_share.get(), speed_share.get()));
    tables.drive_torque.set(x87::mul(
        tables.drive_torque.get(),
        tables.torque_share.get(),
    ));
    {
        let mut work = scene.draw_work_mut();
        work.work_float_a = speed_share.get();
        work.work_float_b = gears_share.get();
    }
    gears_left
}
