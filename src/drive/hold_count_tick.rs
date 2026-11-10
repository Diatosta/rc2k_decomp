//! `drive::hold_count_tick` (0x460C10): `dword_6235EC` counted down to 0 (a reading: a hold after
//! the start).
//!
//! Caller: `drive::replay_step`.

use crate::race::owner_shared::Shared;

// Converted: `rc2k_core::drive::countdown::hold_tick`.
pub fn hold_count_tick(shared: &Shared) {
    let was = shared.vars.hold_count.get();
    let mut hold = was;
    rc2k_core::drive::countdown::hold_tick(&mut hold);
    if hold != was {
        shared.vars.hold_count.set(hold);
    }
}
