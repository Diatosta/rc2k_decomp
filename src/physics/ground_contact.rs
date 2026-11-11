//! `physics::ground_contact` (0x45EFF3): the deepest probe point against the ground.
//! `ground_contact_started` = 0; `physics::lowest_body_point`; nothing more unless the depth
//! (`lowest_depth`) is below 0x1000 and, with `all_wheels_off` at 0, below 0 (each by the
//! sign of the `cmp`). Then `ground_contact_started` = 1; for the point n (`lowest_point`) its
//! three floats from `probe_point_x`[n]`, `probe_point_y`[n]`, `probe_point_z`[n]` into `contact_point`
//! ..; the contact taken (`physics::crash_contact`'s `take_contact`); the slide limit
//! `contact_slide_limit` = 0; `physics::contact_force` into contact record 2 (`g_contactRecords`
//! +0x40), `physics::place_sound_e1`; `contact_depth` raised to |`contact_speed_length`|; the force
//! into `ground_force` ..; with the depth below 0, `physics::restore_saved_car` and
//! `physics::feel_ground`.
//!
//! Caller: `physics::car_physics_step`, every physics step. Refuses a point past 0x3F on the long
//! path (the tables' room), and what its calls refuse; what was written is put back. Not
//! reached: a depth below 0 (`restore_saved_car` and `feel_ground` need the stage; their
//! own tests cover them).

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
fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics_store = &race.physics;
    physics_store.contact.ground_contact_started.set(0);
    {
        lowest_body_point(
            &race.shared.car,
            &race.course.numbers,
            &race.physics.contact,
        )
    }?;
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
    contact_force(race, &race.shared.physics_tables.contacts[2], scene);
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
        {
            restore_saved_car(
                &race.shared.car,
                &race.shared.physics_tables,
                &race.physics.contact,
            )
        }?;
        { feel_ground(race, scene) }?;
    }
    Ok(())
}

pub fn ground_contact(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    steps(race, scene)
}
