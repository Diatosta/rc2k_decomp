//! `physics::mark_car_started` (0x436AA7): `g_car` +0xA0 (`unk_71A078`) set to 1 when
//! it is 0 and +0x29C (`unk_71A274`) is not (a reading: the car has got going).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the callers return next)
//!
//! Caller: `sub_436790` (four calls).

use crate::race::owner_shared::Shared;
use crate::session::Cell32;
use rc2k_core::physics::start::CarStart;

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.started_stand_timer as &dyn Cell32,
        &shared.car.damage_failed,
    ]
}

/// The Rust version on the variables' cells.
// Converted: `rc2k_core::physics::start::CarStart::mark_started`.
fn core(v: &[&dyn Cell32]) {
    let was = CarStart {
        started: v[0].bits(),
        moving: v[1].bits(),
        ..CarStart::default()
    };
    let mut c = was;
    c.mark_started();
    crate::physics::cells::set_changed(v[0], was.started, c.started);
}

/// The Rust version.
pub fn mark_car_started(shared: &Shared) {
    core(&game(shared));
}
