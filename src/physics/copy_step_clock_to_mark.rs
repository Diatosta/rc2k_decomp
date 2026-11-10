//! `physics::copy_step_clock_to_mark` (0x440C66): `g_car` +8 (`dword_719FE0`) into
//! `dword_6140F0`, and `dword_6140EC` 0 (called in the gearbox: it starts the
//! timing `note_time_mark` finishes, which nothing reads).
//!
//! Returns: the word
//!
//! Caller: `physics::pick_start_gear`.

use crate::race::owner_shared::Shared;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    let w = v[0].bits();
    v[1].set_bits(w);
    v[2].set_bits(0);
}

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.step_clock as &dyn Cell32,
        &shared.physics_tables.time_marks[1],
        &shared.physics_tables.time_marks[0],
    ]
}

pub fn copy_step_clock_to_mark(shared: &Shared) {
    core(&game(shared));
}
