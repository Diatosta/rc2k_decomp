//! `physics::cap_drive_speed` (0x440DF3): a slip allowance `flt_614168` =
//! `flt_613E48` / (|`g_carSpeed`| + `flt_613E4C`) - `flt_613E50`, held at 0 or more (an
//! unordered one counts as below); `g_carDriveSpeed` held to at most the speed plus it
//! (through `dword_62400C`; a drive speed below that, or unordered, is left).
//!
//! Caller: `physics::drive_wheels_step` (0x43E0DD).

use crate::math::x87;
use crate::math::x87_run::over_abs_plus_minus;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The places: the allowance, the speed, the scratch, the drive speed.
pub type CapPlaces<'a> = [&'a dyn Cell32; 4];

/// The core.
pub fn cap_drive_speed_at(race: &Race, v: CapPlaces<'_>) {
    let zero = race.course.numbers.zero.bits();
    put(
        v[0],
        over_abs_plus_minus(
            race.shared.physics_tables.drive_speed_cap.bits(),
            at(v[1]),
            race.shared.physics_tables.drive_speed_cap_a.bits(),
            race.shared.physics_tables.drive_speed_cap_b.bits(),
        ),
    );
    if below(at(v[0]), zero) {
        put(v[0], x87::loaded(zero));
    }
    put(v[2], x87::add(at(v[0]), at(v[1])));
    if !below(at(v[3]), at(v[2])) {
        put(v[3], x87::loaded(at(v[2])));
    }
}

fn game<'a>(shared: &'a Shared, scene: &'a crate::scene::owner::Scene) -> CapPlaces<'a> {
    [
        &shared.physics_tables.drive_speed_figure,
        &shared.car.local_velocity[2],
        &scene.drawing.scene.scratch,
        &shared.car.drive_speed_a,
    ]
}

/// The exact version, for `physics::drive_wheels_step`.
pub fn cap_drive_speed(race: &Race, scene: &crate::scene::owner::Scene) {
    cap_drive_speed_at(race, game(&race.shared, scene));
}
