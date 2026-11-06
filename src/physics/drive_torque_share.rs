//! `physics::drive_torque_share` (0x440D56): a share of the drive by speed. n =
//! `dword_613EEC` (8) with `g_carDriveKind` 1, else `dword_613EF0` (32), into
//! `dword_62400C`; s = |`g_carSpeed`| / `dword_613EF4` (`fidiv`) into `flt_62401C`, held to
//! at most `flt_5C5D38` (1.0; one below it, or unordered, is left); d = (n - `g_car`
//! +0x28, `unk_71A000`) / n (`fild` / `fidiv`) into `dword_624020`; `flt_614758` = d +
//! (1 - d) s, and `flt_614750` times it.
//!
//! In:  nothing
//! Out: eax n less the word (no `pusha`); the rest kept; no flags read (the callers return
//!      or load floats next)
//!
//! Callers: `sub_421848`, `sub_43CAAB`, `sub_43CB14`.

use crate::math::x87;
use crate::math::x87_run::{abs_fidiv, int_fidiv, one_less_times_plus};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner::race;
use crate::race::owner_course::Course;
use crate::scene::owner::scene;
use crate::session::Cell32;
use std::cmp::Ordering;

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The core on the variables' cells: eax as the original leaves it (n less the
/// word, its `sub eax, unk_71A000`).
pub fn drive_torque_share_at(course: &Course, v: &[&dyn Cell32]) -> u32 {
    let n = if v[1].bits() == 1 {
        PHYSICS.torque_share_a.get()
    } else {
        PHYSICS.torque_share_b.get()
    };
    v[0].set_bits(n);
    put(
        v[3],
        abs_fidiv(v[2].bits(), PHYSICS.torque_share_c.get() as i32),
    );
    let one = course.numbers.one.bits();
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

fn game() -> Vec<&'static dyn Cell32> {
    let drawing = &scene().drawing;
    vec![
        &drawing.scene.scratch as &dyn Cell32,
        &CAR.drive_kind,
        &CAR.local_velocity[2],
        &drawing.scene.work_float_a,
        &drawing.scene.work_float_b,
        &CAR.gear_ratio_setting,
        &PHYSICS.torque_share,
        &PHYSICS.drive_torque,
    ]
}

/// The Rust version (the exact one, for `physics::drive_force` and
/// `physics::test_drive_run`): eax as the original leaves it.
pub fn drive_torque_share() -> u32 {
    let course = &race().course;
    drive_torque_share_at(course, &game())
}
