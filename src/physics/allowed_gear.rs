//! `physics::allowed_gear` (0x4651B5): eax kept or made 0. 0 when `g_car` +0x2B8
//! (`unk_71A290`) holds the bits of `flt_5C5D14` (100.0, compared as a dword); for eax
//! 1 or more (by the sign of eax - 1) also 0 when `physics::gear_bit` of it gives CF;
//! then 0 unless +0x3F4 (`unk_71A3CC`) is set. A reading: the gear asked for, unless the
//! gearbox refuses it.
//!
//! In:  eax the gear
//! Out: eax the gear or 0; the rest kept (ebx and ecx pushed around); no flags read
//!      (the caller stores eax)
//!
//! Caller: `sub_43C1E3` (0x43C1FD). Refuses what `gear_bit` refuses (a bit past 31).

use crate::errors::Refused;
use crate::physics::gear_bit::bits_at;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// The Rust version on the word +0x2B8, the word +0x3F4 and the gear bits at these places.
// Converted: `rc2k_core::physics::gearbox::allowed_gear`.
pub fn allowed_gear_at(
    course: &Course,
    locked: &dyn crate::session::Cell32,
    open: &dyn crate::session::Cell32,
    bits: &dyn crate::session::Cell32,
    eax: u32,
) -> Result<u32, Refused> {
    let locked = locked.bits() == course.numbers.hundred.bits();
    Ok(rc2k_core::physics::gearbox::allowed_gear(
        locked,
        eax,
        || bits_at(bits),
        || open.bits(),
    )?)
}

/// The Rust version.
pub fn allowed_gear(race: &Race, eax: u32) -> Result<u32, Refused> {
    allowed_gear_at(
        &race.course,
        &race.shared.car.engine_damage,
        &race.shared.car.engine_power_factor,
        &race.shared.car.lost_gear_bits,
        eax,
    )
}
