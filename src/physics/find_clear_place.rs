//! Moves the work car, after a reset, to a place where nothing touches it. `clear_place_mark` =
//! 0x5A; its speed times `clear_place_share` (`physics::scale_vector`);
//! `physics::restore_saved_car`; `physics::lowest_body_point`, the depth into
//! `clear_place_face`. Then tries n = 0x70 + (`step_clock` & 0xF) .. 0xFF: `clear_place_count`
//! (counted by `restore_saved_car`) = 0, the place moved by 2^((n >> 4) + 1) x
//! (`clear_place_steps_a`[n & 0xF], `clear_place_steps_b`[n & 0xF]) (x, z),
//! `physics::probe_ground`, `physics::crossed_edge`, `physics::wall_contacts`,
//! `physics::car_collisions`, `physics::face_contacts`; a try that restored nothing ends it.
//! None: `physics::reset_car`. Found: `lowest_body_point`, a depth below 0 taken off the
//! height; the log line "porky" with n, the first depth and the last (level 1);
//! `clear_place_tries` + 1; `physics::limit_motion`.
//!
//! Refuses (before anything) a probe count of 0 or past 0x40; what its calls refuse.

use crate::errors::Refused;
use crate::math::matrix3::extended;
use crate::physics::figures::CLEAR_PLACE_SHARE;
use crate::physics::step_families::ClearPlaceInputs;
use rc2k_core::physics::clear_place;

/// The log line's text.
const PORKY: &std::ffi::CStr = c"porky";

/// One try's calls.
fn try_place(c: &ClearPlaceInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let contact = &c.collisions.contact;
    { crate::physics::probe_ground::probe_ground(&c.reset.ground, c.reset.car) }?;
    crate::physics::crossed_edge::check(contact.contact)?;
    crate::physics::crossed_edge::steps(contact)?;
    crate::physics::wall_contacts::check(c.reset.car)?;
    crate::physics::wall_contacts::steps(contact, scene)?;
    { crate::physics::car_collisions::car_collisions(&c.collisions) }?;
    crate::physics::face_contacts_around::face_contacts_around(contact, scene)
}

/// The steps, not guarded.
// Core: `rc2k_core::physics::clear_place`.
pub fn steps(c: &ClearPlaceInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let (car, tables, numbers) = (c.reset.car, c.reset.tables, c.reset.numbers);
    let contact = c.collisions.contact.contact;
    tables.clear_place_mark.set(clear_place::TRY_MARK);
    let factor = extended(CLEAR_PLACE_SHARE);
    crate::physics::scale_vector::scale_vector(&car.velocity, factor);
    { crate::physics::restore_saved_car::restore_saved_car(car, tables, contact) }?;
    { crate::physics::lowest_body_point::lowest_body_point(car, numbers, contact) }?;
    tables.clear_place_face.set(contact.lowest_depth.get());
    let found = clear_place::tries(clear_place::first_try(car.step_clock.get()), |n| {
        tables.clear_place_count.set(0);
        let m = clear_place::try_move(n);
        for (table, place) in [
            (&tables.clear_place_steps_a, &car.position[0]),
            (&tables.clear_place_steps_b, &car.position[2]),
        ] {
            let (part, _) = clear_place::move_part(m.step, table[m.entry as usize].get());
            place.set(place.get().wrapping_add(part));
        }
        try_place(c, scene)?;
        Ok::<_, Refused>(tables.clear_place_count.get() == 0)
    })?;
    let Some(n) = found else {
        return crate::physics::reset_car::reset_car(&c.reset, scene);
    };
    { crate::physics::lowest_body_point::lowest_body_point(car, numbers, contact) }?;
    let depth = contact.lowest_depth.get();
    if let Some(height) = clear_place::lifted(car.position[1].get(), depth) {
        car.position[1].set(height);
    }
    crate::logfile::log_write::log_nul(&PORKY, false, 1);
    crate::logfile::log_int::log_int(n as i32, false, 1);
    crate::logfile::log_hex8::log_hex8(tables.clear_place_face.get(), false, 1);
    crate::logfile::log_hex8::log_hex8(contact.lowest_depth.get(), true, 1);
    tables
        .clear_place_tries
        .set(tables.clear_place_tries.get().wrapping_add(1));
    crate::physics::limit_motion::limit_motion(car, tables);
    Ok(())
}

pub(super) fn find_clear_place(
    c: &ClearPlaceInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::physics::wall_contacts::check(c.reset.car)?;
    steps(c, scene)
}
