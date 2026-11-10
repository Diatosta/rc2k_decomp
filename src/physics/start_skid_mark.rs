//! `physics::start_skid_mark` (0x43F272): when either of the two qwords at
//! `qword_71A5D8`, `qword_71A5E0` is not 0 and `dword_71A664` is: `dword_71A660` = it less
//! 1, `dword_71A66C` 1, `dword_71A674` 0, `dword_71A670` 10, `dword_71A668` the step count
//! `dword_719FE0`; then `dword_71A664` 0. By the look of it a mark's start (a reading).
//!
//! Caller: `physics::car_physics_step` (0x43B6FB), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::race::owner_shared::Shared;
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

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.wheel_squeeze[0] as &dyn Cell32,
        &shared.car.wheel_squeeze[1],
        &shared.car.wheel_squeeze[2],
        &shared.car.wheel_squeeze[3],
        &shared.car.camera_view_return,
        &shared.car.camera_blend_view,
        &shared.car.camera_switch_active,
        &shared.car.camera_switch_share,
        &shared.car.camera_switch_length,
        &shared.car.step_clock,
        &shared.car.camera_switch_clock,
    ]
}

pub fn start_skid_mark(shared: &Shared) {
    start_at(&game(shared));
}
