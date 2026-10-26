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

use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put_changed};
use rc2k_core::physics::ground::SkidStart;

/// The Rust version on the variables at `v`. The registers the original leaves are read by none of its callers (eax).
// Converted: `rc2k_core::physics::ground::SkidStart::start`.
pub fn start_at(v: &[u32]) {
    let wheels = [get(v[0]), get(v[1]), get(v[2]), get(v[3])];
    let was = SkidStart {
        pending: get(v[4]),
        from: get(v[5]),
        on: get(v[6]),
        w674: get(v[7]),
        w670: get(v[8]),
        step: get(v[10]),
    };
    let mut s = was;
    s.start(wheels, || get(v[9]));
    for (k, before, now) in [
        (5, was.from, s.from),
        (6, was.on, s.on),
        (7, was.w674, s.w674),
        (8, was.w670, s.w670),
        (10, was.step, s.step),
        (4, was.pending, s.pending),
    ] {
        put_changed(v[k], before, now);
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.wheel_squeeze[0].addr() as u32,
        CAR.wheel_squeeze[1].addr() as u32,
        CAR.wheel_squeeze[2].addr() as u32,
        CAR.wheel_squeeze[3].addr() as u32,
        CAR.camera_view_return.addr() as u32,
        CAR.camera_blend_view.addr() as u32,
        CAR.camera_switch_active.addr() as u32,
        CAR.camera_switch_share.addr() as u32,
        CAR.camera_switch_length.addr() as u32,
        CAR.step_clock.addr() as u32,
        CAR.camera_switch_clock.addr() as u32,
    ]
}

/// The Rust version.
pub fn start_skid_mark() {
    start_at(&game())
}
