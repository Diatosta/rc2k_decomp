//! `physics::mark_car_started` (0x436AA7): `g_car` +0xA0 (`unk_71A078`) set to 1 when
//! it is 0 and +0x29C (`unk_71A274`) is not (a reading: the car has got going).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the callers return next)
//!
//! Caller: `sub_436790` (four calls).

use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put_changed};
use rc2k_core::physics::start::CarStart;

fn game() -> Vec<u32> {
    vec![CAR.started_stand_timer.addr(), CAR.damage_failed.addr()]
}

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::start::CarStart::mark_started`.
fn core(v: &[u32]) {
    let was = CarStart {
        started: get(v[0]),
        moving: get(v[1]),
        ..CarStart::default()
    };
    let mut c = was;
    c.mark_started();
    put_changed(v[0], was.started, c.started);
}

/// The Rust version.
pub fn mark_car_started() {
    core(&game())
}
