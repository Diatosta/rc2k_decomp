//! `physics::cap_drive_speed` (0x440DF3): a slip allowance `flt_614168` =
//! `flt_613E48` / (|`g_carSpeed`| + `flt_613E4C`) - `flt_613E50`, held at 0 or more (an
//! unordered one counts as below); `g_carDriveSpeed` held to at most the speed plus it
//! (through `dword_62400C`; a drive speed below that, or unordered, is left).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a `mov`)
//!
//! Caller: `sub_43DFB1` (0x43E0DD).

use crate::math::x87;
use crate::math::x87_run::over_abs_plus_minus;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner::race;
use crate::scene::draw_store::DRAW;
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
pub type CapPlaces = [&'static dyn Cell32; 4];

/// The core.
pub fn cap_drive_speed_at(v: CapPlaces) {
    let zero = race().course.numbers.zero.bits();
    put(
        v[0],
        over_abs_plus_minus(
            PHYSICS.drive_speed_cap.bits(),
            at(v[1]),
            PHYSICS.drive_speed_cap_a.bits(),
            PHYSICS.drive_speed_cap_b.bits(),
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

fn game() -> CapPlaces {
    [
        &PHYSICS.drive_speed_figure,
        &CAR.local_velocity[2],
        &DRAW.scratch,
        &CAR.drive_speed_a,
    ]
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub fn cap_drive_speed() {
    cap_drive_speed_at(game());
}
