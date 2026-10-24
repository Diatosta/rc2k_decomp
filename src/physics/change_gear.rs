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
use crate::framework::{game_addr, rva};
use crate::physics::gear_bit::gear_bit_at;
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at, wheels_from_revs_at};
use rc2k_core::physics::gear_shifts::Change;

/// The other places: the asked gear, the gear bits, the two words set.
#[derive(Clone, Copy)]
pub struct ChangePlaces {
    pub wanted: u32,
    pub bits: u32,
    pub changed: u32,
    pub changed_2: u32,
}

impl ChangePlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            wanted: g(0x614C40),
            bits: g(0x71A40C),
            changed: g(0x71A210),
            changed_2: g(0x71A214),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
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
pub unsafe fn change_gear_at(gp: &GearPlaces, c: &ChangePlaces) -> Result<(), Refused> {
    unsafe {
        let gear = at(gp.gear);
        let w = at(c.wanted);
        let way = rc2k_core::physics::gear_shifts::change(gear, w);
        if way == Change::Same {
            return Ok(());
        }
        let mark = || {
            put(c.changed, 1);
            put(c.changed_2, 1);
        };
        let neutral = || {
            put(gp.gear, 0);
            mark();
            Ok(())
        };
        if way == Change::FromNeutral {
            ratio_gear(w)?;
            put(gp.gear, w);
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
        put(gp.gear, w);
        revs_from_wheels_at(gp)?;
        mark();
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn change_gear() -> Result<(), Refused> {
    unsafe { change_gear_at(&GearPlaces::game(), &ChangePlaces::game()) }
}
