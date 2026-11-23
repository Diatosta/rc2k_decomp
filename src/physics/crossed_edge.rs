//! When a probe point crossed a face's edge (`crossed_probe` not -1): its point
//! (`probe_point_x` ..) becomes `contact_point`; then `physics::restore_saved_car`,
//! `physics::crossed_edge_contact`, `contact_depth` raised to |`contact_speed_length`|,
//! `contact_flag` = 1, `wall_hit_timer` = 0x32 and `physics::feel_ground`.
//!
//! Refuses (before anything) a probe point past 0x3F (the tables' room), and what its calls refuse
//! (what was written is put back).

use crate::errors::Refused;
use crate::memory::plain_floats::FloatRowMut;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ProbeLists;
use crate::physics::crossed_edge_contact::check_probe;
use crate::physics::step_families::ContactInputs;

/// The steps, not guarded (the probe checked before).
// Core: `rc2k_core::physics::walls::{NONE_CROSSED, raised_hit}`.
pub fn steps(c: &ContactInputs<'_>, car: &mut Car) -> Result<(), Refused> {
    let n = c.ground.probes.borrow().crossed_probe;
    if n == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    let k = n as usize;
    car.contact_point.set_at(0, car.probe_point_x[k]);
    car.contact_point.set_at(1, car.probe_point_y[k]);
    car.contact_point.set_at(2, car.probe_point_z[k]);
    {
        crate::physics::restore_saved_car::restore_saved_car(
            car,
            &mut c.tables.borrow_mut(),
            &c.ground.probes.borrow(),
        )
    }?;
    crate::physics::crossed_edge_contact::steps(c, car)?;
    if let Some(hit) = rc2k_core::physics::walls::raised_hit(
        f32::from_bits(c.contact.borrow().contact_speed_length),
        car.contact_depth,
    ) {
        car.contact_depth = hit;
    }
    car.contact_flag = 1;
    car.wall_hit_timer = 0x32;
    crate::physics::feel_ground::feel_ground(&c.ground, car)
}

/// The probe checked (-1 is none).
pub fn check(probes: &ProbeLists) -> Result<(), Refused> {
    if probes.crossed_probe == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    check_probe(probes)
}

pub fn crossed_edge(c: &ContactInputs<'_>, car: &mut Car) -> Result<(), Refused> {
    check(&c.ground.probes.borrow())?;
    steps(c, car)
}
