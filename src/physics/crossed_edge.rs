//! When a probe point crossed a face's edge (`crossed_probe` not -1): its point
//! (`probe_point_x` ..) becomes `contact_point`; then `physics::restore_saved_car`,
//! `physics::crossed_edge_contact`, `contact_depth` raised to |`contact_speed_length`|,
//! `contact_flag` = 1, `wall_hit_timer` = 0x32 and `physics::feel_ground`.
//!
//! Callers: `physics::car_physics_step` (every physics step), `physics::find_clear_place`.
//! Refuses (before anything) a probe point past 0x3F (the tables' room), and what its calls
//! refuse (what was written is put back).

use crate::errors::Refused;
use crate::physics::contact_store::ContactWork;
use crate::physics::crossed_edge_contact::check_probe;
use crate::physics::step_families::ContactInputs;

/// The steps, not guarded (the probe checked before).
// Core: `rc2k_core::physics::walls::{NONE_CROSSED, raised_hit}`.
pub fn steps(c: &ContactInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let contact = c.contact;
    let n = contact.crossed_probe.get();
    if n == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    let k = n as usize;
    c.car.contact_point[0].set(c.car.probe_point_x[k].get());
    c.car.contact_point[1].set(c.car.probe_point_y[k].get());
    c.car.contact_point[2].set(c.car.probe_point_z[k].get());
    { crate::physics::restore_saved_car::restore_saved_car(c.car, c.tables, c.contact) }?;
    crate::physics::crossed_edge_contact::steps(c, scene)?;
    if let Some(hit) = rc2k_core::physics::walls::raised_hit(
        contact.contact_speed_length.get(),
        c.car.contact_depth.get(),
    ) {
        c.car.contact_depth.set(hit);
    }
    c.car.contact_flag.set(1);
    c.car.wall_hit_timer.set(0x32);
    crate::physics::feel_ground::feel_ground(&c.ground, scene)
}

/// The probe checked (-1 is none).
pub fn check(contact: &ContactWork) -> Result<(), Refused> {
    if contact.crossed_probe.get() == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    check_probe(contact)
}

pub fn crossed_edge(
    c: &ContactInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    check(c.contact)?;
    steps(c, scene)
}
