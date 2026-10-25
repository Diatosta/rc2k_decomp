//! `drive::hold_count_tick` (0x460C10): `dword_6235EC` counted down to 0 (a reading: a
//! hold after the start).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::replay_step`.

use crate::race::vars_store::RACE_VARS;

/// The Rust version.
// Converted: `rc2k_core::drive::countdown::hold_tick`.
pub fn hold_count_tick() {
    let was = RACE_VARS.hold_count.get();
    let mut hold = was;
    rc2k_core::drive::countdown::hold_tick(&mut hold);
    if hold != was {
        RACE_VARS.hold_count.set(hold);
    }
}
