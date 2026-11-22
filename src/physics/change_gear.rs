//! Changes the gear to the one asked for (`gear_asked`); nothing when it is the gear already.
//! From neutral: the gear is set and `physics::wheels_from_revs` runs. To neutral, or when
//! `physics::gear_bit` refuses the new gear: the gear becomes 0. Otherwise `wheels_from_revs`
//! (old gear), the gear set, `physics::revs_from_wheels`. Every change sets the car's
//! `gear_changed` and `gear_change_pending` to 1.
//!
//! Refuses (before anything) a gear or new gear outside -1 ..= 7 where the gearing reads its ratio,
//! and what `gear_bit` refuses.

use crate::errors::{BadInput, Refused};
use crate::memory::session::{PlainDword, PlainInt};
use crate::physics::gear_bit::gear_bit_at;
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at, wheels_from_revs_at};
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::gear_shifts::Change;

/// The other places: the asked gear, the gear bits, the two words set.
#[derive(Clone, Copy)]
struct ChangePlaces<'a> {
    pub wanted: i32,
    /// By address: `bt` on memory, its bit number past the first dword.
    pub bits: &'a PlainDword,
    pub changed: &'a PlainInt,
    pub pending: &'a PlainDword,
}

impl<'a> ChangePlaces<'a> {
    pub fn game(shared: &'a RaceShared) -> Self {
        Self {
            wanted: shared.physics_tables().gear_asked,
            bits: &shared.car().lost_gear_bits,
            changed: &shared.car().gear_changed,
            pending: &shared.car().gear_change_pending,
        }
    }
}

fn ratio_gear(g: i32) -> Result<(), Refused> {
    if (-1..=7).contains(&g) {
        Ok(())
    } else {
        Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into())
    }
}

/// The core.
// Core: `rc2k_core::physics::gear_shifts::change`.
fn change_gear_at(
    numbers: &Numbers,
    gp: &GearPlaces<'_>,
    c: &ChangePlaces<'_>,
) -> Result<(), Refused> {
    let gear = gp.gear.get();
    let w = c.wanted;
    let way = rc2k_core::physics::gear_shifts::change(gear, w);
    if way == Change::Same {
        return Ok(());
    }
    let mark = || {
        c.changed.set(1);
        c.pending.set(1);
    };
    let neutral = || {
        gp.gear.set(0);
        mark();
        Ok(())
    };
    if way == Change::FromNeutral {
        ratio_gear(w)?;
        gp.gear.set(w);
        wheels_from_revs_at(gp)?;
        mark();
        return Ok(());
    }
    if way == Change::ToNeutral {
        return neutral();
    }
    if gear_bit_at(c.bits, w)? {
        return neutral();
    }
    ratio_gear(gear)?;
    ratio_gear(w)?;
    wheels_from_revs_at(gp)?;
    gp.gear.set(w);
    revs_from_wheels_at(numbers, gp)?;
    mark();
    Ok(())
}

pub fn change_gear(numbers: &Numbers, shared: &RaceShared) -> Result<(), Refused> {
    change_gear_at(
        numbers,
        &GearPlaces::game(shared.car(), &shared.physics_tables()),
        &ChangePlaces::game(shared),
    )
}
