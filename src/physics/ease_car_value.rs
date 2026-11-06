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

use crate::physics::cells::set_changed;
use rc2k_core::physics::easing::ease_value;

/// The Rust version.
pub fn ease_car_value() {
    let was = crate::race::owner::race().shared.car.eased_value.get();
    let mut value = was;
    ease_value(
        crate::race::owner::race().shared.vars.scene_switch.get(),
        crate::race::owner::race().shared.car.value_jump_steps.get(),
        crate::race::owner::race().shared.car.eased_target.get(),
        &mut value,
    );
    set_changed(
        &crate::race::owner::race().shared.car.eased_value,
        was,
        value,
    );
}
