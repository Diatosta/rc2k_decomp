//! `physics::start_skid_mark` (0x43F272): when either of the two qwords at
//! `qword_71A5D8`, `qword_71A5E0` is not 0 and `dword_71A664` is: `dword_71A660` = it less
//! 1, `dword_71A66C` 1, `dword_71A674` 0, `dword_71A670` 10, `dword_71A668` the step count
//! `dword_719FE0`; then `dword_71A664` 0. By the look of it a mark's start (a reading).
//!
//! In:  nothing
//! Out: eax the step count (`dword_71A664` not 0), 0 (it 0), kept (both qwords 0); no
//!      flags read
//!
//! Caller: `sub_43B635` (0x43B6FB), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::session::Cell32;
use rc2k_core::physics::ground::SkidStart;

/// The Rust version on the variables' cells `v`. The registers the original leaves are read by none of its callers (eax).
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

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &crate::race::owner::race().shared.car.wheel_squeeze[0] as &dyn Cell32,
        &crate::race::owner::race().shared.car.wheel_squeeze[1],
        &crate::race::owner::race().shared.car.wheel_squeeze[2],
        &crate::race::owner::race().shared.car.wheel_squeeze[3],
        &crate::race::owner::race().shared.car.camera_view_return,
        &crate::race::owner::race().shared.car.camera_blend_view,
        &crate::race::owner::race().shared.car.camera_switch_active,
        &crate::race::owner::race().shared.car.camera_switch_share,
        &crate::race::owner::race().shared.car.camera_switch_length,
        &crate::race::owner::race().shared.car.step_clock,
        &crate::race::owner::race().shared.car.camera_switch_clock,
    ]
}

/// The Rust version.
pub fn start_skid_mark() {
    start_at(&game());
}
