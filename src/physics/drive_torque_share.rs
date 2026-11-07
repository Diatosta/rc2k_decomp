//! `physics::drive_torque_share` (0x440D56): a share of the drive by speed. n =
//! `dword_613EEC` (8) with `g_carDriveKind` 1, else `dword_613EF0` (32), into
//! `dword_62400C`; s = |`g_carSpeed`| / `dword_613EF4` (`fidiv`) into `flt_62401C`, held to
//! at most `flt_5C5D38` (1.0; one below it, or unordered, is left); d = (n - `g_car`
//! +0x28, `unk_71A000`) / n (`fild` / `fidiv`) into `dword_624020`; `flt_614758` = d +
//! (1 - d) s, and `flt_614750` times it.
//!
//! Callers: `physics::test_drive_run`, 0x43CAAB, `physics::drive_force`.

use crate::math::x87;
use crate::math::x87_run::{abs_fidiv, int_fidiv, one_less_times_plus};
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;
use std::cmp::Ordering;

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The core on the variables' cells: eax as the original leaves it (n less the
/// word, its `sub eax, unk_71A000`).
pub fn drive_torque_share_at(race: &Race, v: &[&dyn Cell32]) -> u32 {
    let n = if v[1].bits() == 1 {
        race.shared.physics_tables.torque_share_a.get()
    } else {
        race.shared.physics_tables.torque_share_b.get()
    };
    v[0].set_bits(n);
    put(
        v[3],
        abs_fidiv(
            v[2].bits(),
            race.shared.physics_tables.torque_share_c.get() as i32,
        ),
    );
    let one = race.course.numbers.one.bits();
    let below = !matches!(
        f32::from_bits(v[3].bits()).partial_cmp(&f32::from_bits(one)),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !below {
        v[3].set_bits(x87::loaded(one));
    }
    let eax = v[0].bits().wrapping_sub(v[5].bits());
    v[4].set_bits(eax);
    v[4].set_bits(int_fidiv(v[4].bits() as i32, v[0].bits() as i32));
    v[6].set_bits(one_less_times_plus(v[4].bits(), v[3].bits()));
    v[7].set_bits(x87::mul(v[7].bits(), v[6].bits()));
    eax
}

fn game<'a>(shared: &'a Shared, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    let drawing = &scene.drawing;
    vec![
        &drawing.scene.scratch as &dyn Cell32,
        &shared.car.drive_kind,
        &shared.car.local_velocity[2],
        &drawing.scene.work_float_a,
        &drawing.scene.work_float_b,
        &shared.car.gear_ratio_setting,
        &shared.physics_tables.torque_share,
        &shared.physics_tables.drive_torque,
    ]
}

/// The Rust version (the exact one, for `physics::drive_force` and
/// `physics::test_drive_run`): eax as the original leaves it.
pub fn drive_torque_share(race: &Race, scene: &crate::scene::owner::Scene) -> u32 {
    drive_torque_share_at(race, &game(&race.shared, scene))
}
