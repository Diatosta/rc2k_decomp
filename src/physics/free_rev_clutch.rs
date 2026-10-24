//! `physics::free_rev_clutch` (0x43EA63): in gear with the clutch out (+0x250,
//! `unk_71A228`): `physics::free_rev`, then `physics::wheels_from_revs`.
//!
//! In:  nothing
//! Out: ebx the gear (`wheels_from_revs`), and what `free_rev` leaves; no flags read
//!
//! Caller: `sub_43B635` (0x43B7B7), every physics step. Refuses (before anything) a gear
//! outside -1 ..= 7.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::engine_torque::g;
use crate::physics::free_rev::free_rev;
use crate::physics::gearing::{GearPlaces, wheels_from_revs_at};

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn free_rev_clutch() -> Result<(), Refused> {
    let gear = g(0x71A1AC);
    if gear == 0 || g(0x71A228) == 0 {
        return Ok(());
    }
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    unsafe {
        free_rev();
        wheels_from_revs_at(&GearPlaces::game())?;
    }
    Ok(())
}
