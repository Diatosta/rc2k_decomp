//! `physics::ease_car_value` (0x4554CA): unless `dword_6235E8` is set, `dword_71AF94`
//! follows `dword_71AF90` << 8: set to it when `dword_71AF88` is set, else stepped by 4
//! towards it (up when the difference's sign is clear, `jns`). What the values are is
//! not known.
//!
//! In:  nothing
//! Out: eax `dword_71AF90` << 8 (when `dword_6235E8` is 0); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B705), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put_changed};
use crate::race::vars_store::RACE_VARS;
use rc2k_core::physics::easing::ease_value;

/// The Rust version on the variables at `v`. The registers the original leaves are read by none of its callers (eax).
// Converted: `rc2k_core::physics::easing::ease_value`.
pub fn ease_at(v: &[u32; 4]) {
    let was = get(v[3]);
    let mut value = was;
    ease_value(get(v[0]), get(v[1]), get(v[2]), &mut value);
    put_changed(v[3], was, value);
}

fn game() -> [u32; 4] {
    [
        RACE_VARS.scene_switch.addr() as u32,
        CAR.value_jump_steps.addr() as u32,
        CAR.eased_target.addr() as u32,
        CAR.eased_value.addr() as u32,
    ]
}

/// The Rust version.
pub fn ease_car_value() {
    ease_at(&game())
}
