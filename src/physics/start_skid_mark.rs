//! `physics::start_skid_mark` (0x43F272): when either of the two qwords at
//! `qword_71A5D8`, `qword_71A5E0` is not 0 and `dword_71A664` is: `dword_71A660` = it less
//! 1, `dword_71A66C` 1, `dword_71A674` 0, `dword_71A670` 10, `dword_71A668` the step count
//! `dword_719FE0`; then `dword_71A664` 0. This starts a skid mark (the start's place, a count of 10, the step it began).
//!
//! Caller: `physics::car_physics_step` (0x43B6FB), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::Car;
use crate::session::Cell32;
use rc2k_core::physics::ground::SkidStart;

/// On the variables' cells `v`.
// Converted: `rc2k_core::physics::ground::SkidStart::start`.
pub fn start_at(v: &[&dyn Cell32]) {
    let wheels = [v[0].bits(), v[1].bits(), v[2].bits(), v[3].bits()];
    let was = SkidStart {
        pending: v[4].bits(),
        from: v[5].bits(),
        on: v[6].bits(),
        w674: v[7].bits(),
        w670: v[8].bits(),
        step: v[10].bits(),
    };
    let mut s = was;
    s.start(wheels, || v[9].bits());
    for (k, before, now) in [
        (5, was.from, s.from),
        (6, was.on, s.on),
        (7, was.w674, s.w674),
        (8, was.w670, s.w670),
        (10, was.step, s.step),
        (4, was.pending, s.pending),
    ] {
        crate::physics::cells::set_changed(v[k], before, now);
    }
}

fn game(car: &Car) -> Vec<&dyn Cell32> {
    vec![
        &car.wheel_squeeze[0] as &dyn Cell32,
        &car.wheel_squeeze[1],
        &car.wheel_squeeze[2],
        &car.wheel_squeeze[3],
        &car.camera_view_return,
        &car.camera_blend_view,
        &car.camera_switch_active,
        &car.camera_switch_share,
        &car.camera_switch_length,
        &car.step_clock,
        &car.camera_switch_clock,
    ]
}

pub fn start_skid_mark(car: &Car) {
    start_at(&game(car));
}
