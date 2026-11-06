//! `physics::change_gear` (0x43C20D): the gear changed to the one asked for
//! (`dword_614C40`). Nothing when it is the gear already (eax the gear). From neutral:
//! the gear set and `physics::wheels_from_revs`. To neutral, or when `physics::gear_bit`
//! refuses the new gear (CF): the gear 0. Else `wheels_from_revs` (old gear), the gear
//! set, `physics::revs_from_wheels`. Every change sets `g_car` +0x238 and +0x23C
//! (`unk_71A210`, `unk_71A214`) to 1.
//!
//! In:  nothing
//! Out: eax the gear (or the new one; gear + 1 after a refusing `gear_bit`), ecx the new
//!      gear when `gear_bit` ran, ebx as the gearing calls leave it; the rest kept; no flags
//!      read (the caller returns)
//!
//! Caller: `sub_43C1E3` (its tail). Refuses (before anything) a gear or a new gear outside
//! -1 ..= 7 where the gearing reads its ratio, and what `gear_bit` refuses.

use crate::errors::{BadInput, Refused};
use crate::physics::gear_bit::gear_bit_at;
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at, wheels_from_revs_at};
use crate::race::owner_course::Course;
use crate::session::Cell32;
use rc2k_core::physics::gear_shifts::Change;

/// The other places: the asked gear, the gear bits, the two words set.
#[derive(Clone, Copy)]
pub struct ChangePlaces {
    pub wanted: &'static dyn Cell32,
    /// By address: `bt` on memory, its bit number past the first dword.
    pub bits: &'static dyn Cell32,
    pub changed: &'static dyn Cell32,
    pub changed_2: &'static dyn Cell32,
}

impl ChangePlaces {
    pub fn game() -> Self {
        Self {
            wanted: &crate::race::owner::race().shared.physics_tables.gear_asked,
            bits: &crate::race::owner::race().shared.car.lost_gear_bits,
            changed: &crate::race::owner::race().shared.car.gear_changed,
            changed_2: &crate::race::owner::race().shared.car.gear_changed_2,
        }
    }
}

fn ratio_gear(g: u32) -> Result<(), Refused> {
    if (-1..=7).contains(&(g as i32)) {
        Ok(())
    } else {
        Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into())
    }
}

/// The core.
// Converted: `rc2k_core::physics::gear_shifts::change`.
pub fn change_gear_at(course: &Course, gp: &GearPlaces, c: &ChangePlaces) -> Result<(), Refused> {
    let gear = gp.gear.bits();
    let w = c.wanted.bits();
    let way = rc2k_core::physics::gear_shifts::change(gear, w);
    if way == Change::Same {
        return Ok(());
    }
    let mark = || {
        c.changed.set_bits(1);
        c.changed_2.set_bits(1);
    };
    let neutral = || {
        gp.gear.set_bits(0);
        mark();
        Ok(())
    };
    if way == Change::FromNeutral {
        ratio_gear(w)?;
        gp.gear.set_bits(w);
        wheels_from_revs_at(gp)?;
        mark();
        return Ok(());
    }
    if way == Change::ToNeutral {
        return neutral();
    }
    let bit = gear_bit_at(c.bits, w)?;
    if bit.carry {
        return neutral();
    }
    ratio_gear(gear)?;
    ratio_gear(w)?;
    wheels_from_revs_at(gp)?;
    gp.gear.set_bits(w);
    revs_from_wheels_at(course, gp)?;
    mark();
    Ok(())
}

/// The Rust version.
pub fn change_gear(course: &Course) -> Result<(), Refused> {
    change_gear_at(course, &GearPlaces::game(), &ChangePlaces::game())
}
