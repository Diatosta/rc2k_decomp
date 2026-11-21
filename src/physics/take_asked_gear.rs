//! `gear_changed` = 0; the gear asked for, bits 8 .. 11 of `input_keys` as a signed nibble, goes
//! through `physics::allowed_gear` into `gear_asked`; then `physics::change_gear`.
//!
//! Refuses what the two calls refuse; what was written by then is put back.

use crate::errors::Refused;
use crate::physics::allowed_gear::allowed_gear;
use crate::physics::change_gear::change_gear;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;

// Core: `rc2k_core::physics::gearbox::asked_gear`.
pub(super) fn take_asked_gear(numbers: &Numbers, shared: &RaceShared) -> Result<(), Refused> {
    let run = || -> Result<(), Refused> {
        shared.car().gear_changed.set(0);
        let word: u32 = shared.car().input_keys.get();
        let asked = rc2k_core::physics::gearbox::asked_gear(word);
        let gear = allowed_gear(shared.car(), numbers, asked)?;
        shared.physics_tables.get().gear_asked.set(gear);
        change_gear(numbers, shared)
    };

    run()
}
