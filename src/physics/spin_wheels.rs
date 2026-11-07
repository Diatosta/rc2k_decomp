//! `physics::spin_wheels` (0x43CD1A): the driven wheels' turns (`dword_71A1D0`,
//! `dword_71A1D4`) on by `g_carDriveSpeed` / `g_carDriveSpeed2` over `flt_613EBC`, times
//! the step `flt_61388C`, 65536 and `flt_5C5DC8` (`fistp` through `dword_62400C`). By the
//! look of it the wheels' angles for the drawing (a reading).
//!
//! In:  nothing
//! Out: eax the second's turn; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B90A), every physics step.

use crate::math::x87_run::div_mul3_to_int;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// The Rust core on the variables' cells `v`: the last turn added.
pub fn spin_at(race: &Race, v: &[&dyn Cell32]) -> u32 {
    let (wheel, dt, k65536, scale) = (
        race.shared.physics_tables.wheel_spin_share.bits(),
        race.shared.physics_tables.step_time.bits(),
        race.course.numbers.k65536.bits(),
        race.course.numbers.one_over_pi.bits(),
    );
    let mut last = 0;
    for (speed, turn) in [(0, 2), (1, 3)] {
        last = div_mul3_to_int(get(v, speed), wheel, dt, k65536, scale) as u32;
        put(v, 4, last);
        put(v, turn, get(v, turn).wrapping_add(last));
    }
    last
}

fn game<'a>(shared: &'a Shared, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    vec![
        &shared.car.drive_speed_a as &dyn Cell32,
        &shared.car.drive_speed_b,
        &shared.car.driven_wheel_turns[0],
        &shared.car.driven_wheel_turns[1],
        &scene.drawing.scene.scratch,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub fn spin_wheels(race: &Race, scene: &crate::scene::owner::Scene) -> u32 {
    spin_at(race, &game(&race.shared, scene))
}
