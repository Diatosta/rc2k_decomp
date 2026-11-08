//! `physics::note_time_mark` (0x440C38): when `flt_71AEE0` is not below `flt_5C5D5C`
//! (60.0; `fcomp`, C0 clear: also not unordered) and `dword_6140EC` is 0:
//! `dword_6140EC` = `dword_719FE0` (the work copy's +8, a step count by `race::step_replay_cars`'s
//! `inc`) less `dword_6140F0`. By the look of it the step at which a figure first
//! reached 60 (a reading).
//!
//! Caller: `physics::car_physics_step` (0x43B9A4), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;
use rc2k_core::physics::motion::time_mark;

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.engine_speed_scaled as &dyn Cell32,
        &shared.physics_tables.time_marks[0],
        &shared.car.step_clock,
        &shared.physics_tables.time_marks[1],
    ]
}

/// The converted version (`rc2k_core::physics::motion`) on the variables' cells `v`. The eax
/// it leaves (the mark) is read by none of its callers.
// Converted: `rc2k_core::physics::motion::time_mark`.
fn core(course: &Course, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let limit = f32::from_bits(course.numbers.sixty.bits());
    // The mark, count and start read only where the original reads them.
    if let Some(mark) = time_mark(
        f32::from_bits(get(0)),
        limit,
        || get(1),
        || (get(2), get(3)),
    ) {
        v[1].set_bits(mark);
    }
}

/// The Rust version (a compare of floats only: the same at any precision); `r` as the
/// original leaves are read by none of its callers.
pub fn note_time_mark(race: &Race) {
    core(&race.course, &game(&race.shared));
}
