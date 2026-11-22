//! Moves the work car, after a reset, to a place where nothing touches it. Its speed times `clear_place_share` (`physics::scale_vector`);
//! `physics::restore_saved_car`; `physics::lowest_body_point`, the depth into
//! `clear_place_face`. Then tries n = 0x70 + (`step_clock` & 0xF) .. 0xFF: `clear_place_count`
//! (counted by `restore_saved_car`) = 0, the place moved by 2^((n >> 4) + 1) x
//! (`clear_place_steps_a`[n & 0xF], `clear_place_steps_b`[n & 0xF]) (x, z),
//! `physics::probe_ground`, `physics::crossed_edge`, `physics::wall_contacts`,
//! `physics::car_collisions`, `physics::face_contacts`; a try that restored nothing ends it.
//! None: `physics::reset_car`. Found: `lowest_body_point`, a depth below 0 taken off the
//! height; the log line "porky" with n, the first depth and the last (level 1);
//! `physics::limit_motion`.
//!
//! Refuses (before anything) a probe count of 0 or past 0x40; what its calls refuse.

use crate::errors::Refused;
use crate::math::matrix3::extended;
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::physics::figures::{CLEAR_PLACE_SHARE, CLEAR_PLACE_STEPS_A, CLEAR_PLACE_STEPS_B};
use crate::physics::step_families::ClearPlaceInputs;
use rc2k_core::physics::clear_place;

/// The log line's text.
const PORKY: &std::ffi::CStr = c"porky";

/// One try's calls.
fn try_place(
    c: &ClearPlaceInputs<'_>,
    scene: &crate::scene::owner::Scene,
    car: &mut Car,
) -> Result<(), Refused> {
    let contact = &c.collisions.contact;
    { crate::physics::probe_ground::probe_ground(&c.reset.ground, car) }?;
    crate::physics::crossed_edge::check(&contact.ground.probes.borrow())?;
    crate::physics::crossed_edge::steps(contact, car)?;
    crate::physics::wall_contacts::check(car)?;
    crate::physics::wall_contacts::steps(contact, scene, car)?;
    { crate::physics::car_collisions::car_collisions(&c.collisions, car) }?;
    crate::physics::face_contacts_around::face_contacts_around(contact, scene, car)
}

/// The steps, not guarded.
// Core: `rc2k_core::physics::clear_place`.
pub fn steps(c: &ClearPlaceInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let (tables, numbers) = (c.reset.tables, c.reset.numbers);
    let mut work_car = c.reset.shared.car_mut();
    let car = &mut *work_car;
    let contact = c.collisions.contact.contact;
    let factor = extended(CLEAR_PLACE_SHARE);
    crate::physics::scale_vector::scale_vector_plain(&mut car.velocity, factor);
    {
        crate::physics::restore_saved_car::restore_saved_car(
            car,
            &mut tables.borrow_mut(),
            &c.reset.ground.probes.borrow(),
        )
    }?;
    {
        crate::physics::lowest_body_point::lowest_body_point(
            car,
            numbers,
            contact,
            &c.reset.ground.probes.borrow(),
        )
    }?;
    let lowest_depth = contact.borrow().lowest_depth;
    tables.borrow_mut().clear_place_face = lowest_depth;
    let found = clear_place::tries(clear_place::first_try(car.step_clock), |n| {
        tables.borrow_mut().clear_place_count = 0;
        let m = clear_place::try_move(n);
        for (table, axis) in [(&CLEAR_PLACE_STEPS_A, 0), (&CLEAR_PLACE_STEPS_B, 2)] {
            let (part, _) = clear_place::move_part(m.step, table[m.entry as usize]);
            car.position[axis] = f32::from_bits(car.position[axis].to_bits().wrapping_add(part));
        }
        try_place(c, scene, car)?;
        Ok::<_, Refused>(tables.borrow().clear_place_count == 0)
    })?;
    let Some(n) = found else {
        drop(work_car);
        return crate::physics::reset_car::reset_car(&c.reset, scene);
    };
    {
        crate::physics::lowest_body_point::lowest_body_point(
            car,
            numbers,
            contact,
            &c.reset.ground.probes.borrow(),
        )
    }?;
    let depth = contact.borrow().lowest_depth;
    if let Some(height) = clear_place::lifted(car.position.bits_at(1), depth) {
        car.position[1] = f32::from_bits(height);
    }
    crate::logfile::log_write::log_nul(&PORKY, false, 1);
    crate::logfile::log_int::log_int(n as i32, false, 1);
    let face = tables.borrow().clear_place_face;
    crate::logfile::log_hex8::log_hex8(face, false, 1);
    crate::logfile::log_hex8::log_hex8(contact.borrow().lowest_depth, true, 1);
    crate::physics::limit_motion::limit_motion(car, &tables.borrow());
    Ok(())
}

pub(super) fn find_clear_place(
    c: &ClearPlaceInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::physics::wall_contacts::check(&c.reset.shared.car())?;
    steps(c, scene)
}
