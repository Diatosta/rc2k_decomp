//! `drive::finish_race_0` (0x4278C7): `dword_621870` 0 and `drive::finish_race`.
//!
//! In:  nothing
//! Out: as `finish_race` leaves the registers; no flags read
//!
//! Callers: `sub_488155`, `sub_4A14CB`, `sub_4A9439` (the championship's flow). Refuses
//! what `finish_race` refuses.

use crate::errors::Refused;

/// The Rust version.
pub fn finish_race_0() -> Result<(), Refused> {
    crate::race::owner::race().shared.switches.mask_kind.set(0);
    crate::drive::finish_race::finish_race()
}
