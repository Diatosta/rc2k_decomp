//! `physics::find_clear_place` (0x43BDD1): the work car moved to a place where nothing
//! touches it, after a reset. `clear_place_mark` = 0x5A; its speed times `clear_place_share`
//! (`physics::scale_vector`); `physics::restore_saved_car`;
//! `physics::lowest_body_point`, the depth into `clear_place_face`. Then tries n = 0x70 +
//! (`step_clock` & 0xF) .. 0xFF: `clear_place_count` (counted by `restore_saved_car`) = 0, the
//! place moved by 2^((n >> 4) + 1) x (`clear_place_steps_a`[n & 0xF]`, `clear_place_steps_b`[n & 0xF]`)
//! (x, z; `imul`, low dwords), `physics::probe_ground`, `physics::crossed_edge`,
//! `physics::wall_contacts`, `physics::car_collisions`, `physics::face_contacts`; a try
//! that restored nothing ends it. None: `physics::reset_car`. Found:
//! `lowest_body_point`, a depth below 0 taken off the height (+0x800); "porky", n, the
//! first depth and the last logged (level 1); `clear_place_tries` + 1;
//! `physics::limit_motion`.
//!
//! Caller: `physics::car_physics_step` (0x43B8DB). Refuses (before anything) a probe count of 0 or
//! past 0x40; what its calls refuse.

use crate::errors::Refused;
use crate::physics::step_inputs::StepInputs;
use crate::scene::matrix3::{extended, rounded};
use rc2k_core::physics::clear_place;

// g_logEnabled
const PORKY: &std::ffi::CStr = c"porky"; // aPorky

/// One try's calls (0x43BE4D .. 0x43BE61).
fn try_place(c: &StepInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    { crate::physics::probe_ground::probe_ground(&c.ground(), c.car, scene) }?;
    crate::physics::crossed_edge::check(&c.physics.contact)?;
    crate::physics::crossed_edge::steps(&c.contact(), scene)?;
    crate::physics::wall_contacts::check(c.car)?;
    crate::physics::wall_contacts::steps(&c.contact(), scene)?;
    { crate::physics::car_collisions::car_collisions(c, scene) }?;
    crate::physics::face_contacts_around::face_contacts_around(&c.contact(), scene)
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::clear_place`.
pub fn steps(c: &StepInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics_store = c.physics;
    c.tables.clear_place_mark.set(clear_place::TRY_MARK);
    let factor = extended(c.tables.clear_place_share.bits());
    crate::physics::scale_vector::scale_vector(&c.car.velocity, factor);
    scene.drawing.scene.scratch.set(rounded(factor));
    { crate::physics::restore_saved_car::restore_saved_car(c.car, c.tables, &c.physics.contact) }?;
    { crate::physics::lowest_body_point::lowest_body_point(c.car, c.numbers, &c.physics.contact) }?;
    c.tables
        .clear_place_face
        .set(physics_store.contact.lowest_depth.get());
    let found = clear_place::tries(clear_place::first_try(c.car.step_clock.get()), |n| {
        c.tables.clear_place_count.set(0);
        let m = clear_place::try_move(n);
        for (table, place) in [
            (&c.tables.clear_place_steps_a, &c.car.position[0]),
            (&c.tables.clear_place_steps_b, &c.car.position[2]),
        ] {
            let (part, _) = clear_place::move_part(m.step, table[m.entry as usize].get());
            place.set(place.get().wrapping_add(part));
        }
        try_place(c, scene)?;
        Ok::<_, Refused>(c.tables.clear_place_count.get() == 0)
    })?;
    let Some(n) = found else {
        return crate::physics::reset_car::reset_car(c, scene);
    };
    { crate::physics::lowest_body_point::lowest_body_point(c.car, c.numbers, &c.physics.contact) }?;
    let depth = physics_store.contact.lowest_depth.get();
    if let Some(height) = clear_place::lifted(c.car.position[1].get(), depth) {
        c.car.position[1].set(height);
    }
    crate::logfile::log_write::log_nul(&PORKY, false, 1);
    crate::logfile::log_int::log_int(n as i32, false, 1);
    crate::logfile::log_hex8::log_hex8(c.tables.clear_place_face.get(), false, 1);
    crate::logfile::log_hex8::log_hex8(physics_store.contact.lowest_depth.get(), true, 1);
    c.tables
        .clear_place_tries
        .set(c.tables.clear_place_tries.get().wrapping_add(1));
    crate::physics::limit_motion::limit_motion(c.car, c.tables, scene);
    Ok(())
}

pub fn find_clear_place(
    c: &StepInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::physics::wall_contacts::check(c.car)?;
    steps(c, scene)
}
