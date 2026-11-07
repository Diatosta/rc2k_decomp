//! `physics::copy_car_word_8` (0x440C66): `g_car` +8 (`dword_719FE0`) into
//! `dword_6140F0`, and `dword_6140EC` 0 (in the gearbox: by the look of it the moment a
//! gear change began; a reading).
//!
//! Returns: the word
//!
//! Caller: `sub_43C8AC`.

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

/// The Rust version.
pub fn copy_car_word_8(shared: &Shared) {
    core(&game(shared));
}
