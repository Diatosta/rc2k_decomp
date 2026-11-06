//! `physics::ground_contact` (0x45EFF3): the deepest probe point against the ground.
//! `dword_6D3028` = 0; `physics::lowest_body_point`; nothing more unless the depth
//! (`dword_6D3068`) is below 0x1000 and, with `unk_71A22C` at 0, below 0 (each by the
//! sign of the `cmp`). Then `dword_6D3028` = 1; for the point n (`dword_6D306C`) its
//! three floats from `unk_71AAC8[n]`, `unk_71ABC8[n]`, `unk_71ACC8[n]` into `unk_71A1FC`
//! ..; the contact taken (`physics::crash_contact`'s `take_contact`); the slide limit
//! `flt_6D3100` = 0; `physics::contact_force` into contact record 2 (`g_contactRecords`
//! +0x40), `physics::place_sound_e1`; `unk_71A208` raised to |`flt_6D3108`|; the force
//! into `dword_6D3018` ..; with the depth below 0, `physics::restore_saved_car` and
//! `physics::feel_ground`.
//!
//! In:  nothing
//! Out: the registers as `lowest_body_point` and, on the long path, `contact_force`,
//!      `restore_saved_car` and `feel_ground` leave them, eax the depth; no flags read
//!      (the caller calls on)
//!
//! Caller: `sub_43B635`, every physics step. Refuses a point past 0x3F on the long path
//! (the tables' room), and what its calls refuse; what was written is put back. Not
//! reached: a depth below 0 (`restore_saved_car` and `feel_ground` need the stage; their
//! own tests cover them).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::contact_force::contact_force;
use crate::physics::crash_contact::{point_number, sign, take_contact};
use crate::physics::feel_ground::feel_ground;
use crate::physics::flow::copy_cells;
use crate::physics::lowest_body_point::lowest_body_point;
use crate::physics::place_sound_e1::place_sound_e1;
use crate::physics::restore_saved_car::restore_saved_car;
use crate::race::owner::Race;

// Converted: `rc2k_core::physics::contacts::touches` and `walls::raised_hit` (the x87 steps stay).
fn steps(race: &Race) -> Result<(), Refused> {
    let physics_store = &race.physics;
    physics_store.contact.ground_contact_started.set(0);
    { lowest_body_point(race) }?;
    let depth = physics_store.contact.lowest_depth.get();
    let limit = rc2k_core::physics::contacts::GROUND_LIMIT;
    if !rc2k_core::physics::contacts::touches(
        depth,
        limit,
        race.shared.car.all_wheels_off.get() != 0,
    ) {
        return Ok(());
    }
    let n = point_number(&race.physics)?;
    physics_store.contact.ground_contact_started.set(1);
    race.shared.car.contact_point[0].set_bits(race.shared.car.probe_point_x[n].bits());
    race.shared.car.contact_point[1].set_bits(race.shared.car.probe_point_y[n].bits());
    race.shared.car.contact_point[2].set_bits(race.shared.car.probe_point_z[n].bits());
    take_contact(race, n)?;
    physics_store.contact.contact_slide_limit.set_bits(0);
    contact_force(race, &race.shared.physics_tables.contacts[2]);
    { place_sound_e1(race) }?;
    if let Some(hit) = rc2k_core::physics::walls::raised_hit(
        f32::from_bits(physics_store.contact.contact_speed_length.bits()),
        f32::from_bits(race.shared.car.contact_depth.bits()),
    ) {
        race.shared.car.contact_depth.set_bits(hit.to_bits());
    }
    copy_cells(
        &physics_store.contact.ground_force,
        &race.shared.physics_tables.contacts[2].force,
    );
    if sign(physics_store.contact.lowest_depth.get()) {
        { restore_saved_car(race) }?;
        { feel_ground(race) }?;
    }
    Ok(())
}

/// The Rust version. The registers it leaves are read by none of its callers.
pub fn ground_contact(race: &Race) -> Result<(), Refused> {
    steps(race)
}
