//! `physics::drive_torque_share` (0x440D56): a share of the drive by speed. n is `torque_share_a`
//! (8) with `g_carDriveKind` 1, else `torque_share_b` (32); s = |`g_carSpeed`| / `torque_share_c`
//! held to at most 1.0 (a value below it, or unordered, is left); d = (n - the car's
//! `gear_ratio_setting`) / n; `torque_share` = d + (1 - d) s, and `drive_torque` is multiplied by it.
//!
//! Callers: `physics::test_drive_run`, `physics::drive_force`.

use crate::effects::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87::toward_one;
use crate::math::x87_float as bh;
use crate::physics::car_store::Car;
use crate::physics::flow::FloatView;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::session::Cell32;
use std::cmp::Ordering;

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The core on the variables' cells: returns n less the gear ratio setting word.
pub fn drive_torque_share_at(tables: &PhysicsTables, numbers: &Numbers, v: &[&dyn Cell32]) -> u32 {
    let n = if v[1].bits() == 1 {
        tables.torque_share_a.get()
    } else {
        tables.torque_share_b.get()
    };
    v[0].set_bits(n);
    put(
        v[3],
        bh::st(v[2].ld().abs() / bh::int(tables.torque_share_c.get() as i32)),
    );
    let one = numbers.one.bits();
    let below = !matches!(
        v[3].float().partial_cmp(&f32::from_bits(one)),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !below {
        v[3].set_float(x87::loaded(one));
    }
    let eax = v[0].bits().wrapping_sub(v[5].bits());
    v[4].set_bits(eax);
    v[4].set_float(x87::int_ratio(v[4].bits() as i32, v[0].bits() as i32));
    v[6].set_float(toward_one(v[4], v[3]));
    v[7].set_float(x87::mul(v[7], v[6]));
    eax
}

fn game<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
    scene: &'a crate::scene::owner::Scene,
) -> Vec<&'a dyn Cell32> {
    let drawing = &scene.drawing;
    vec![
        &drawing.scene.scratch as &dyn Cell32,
        &car.drive_kind,
        &car.local_velocity[2],
        &drawing.scene.work_float_a,
        &drawing.scene.work_float_b,
        &car.gear_ratio_setting,
        &tables.torque_share,
        &tables.drive_torque,
    ]
}

/// The exact version, for `physics::drive_force` and `physics::test_drive_run`: returns the same
/// value as the core.
pub fn drive_torque_share(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    drive_torque_share_at(tables, numbers, &game(car, tables, scene))
}
