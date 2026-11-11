//! `physics::wall_push_back` (0x45D8FB): the work car pushed back off a wall against its
//! motion: minus its speed (`unk_71A170` ..) in `g_workVector` (0 when its length is 0,
//! `physics::vector_length`), made one long (`race::normalise_work_vector`) as the
//! normal; the speed and the turn speed taken, the point 0, no slide limit;
//! `physics::contact_force` into contact record 4 (`g_contactRecords` + 0x80),
//! `physics::place_sound_e1`; `dword_6141C8` = -1.
//!
//! Callers: `physics::wall_contacts`, `physics::wall_edge_contact`. Refuses what
//! `place_sound_e1` refuses (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87_run::compare_80;
use crate::physics::contact_force::contact_force;
use crate::physics::flow::{ZERO_BITS as ZERO, copy_cells, fill_cells, negate};
use crate::physics::vector_length::vector_length;
use crate::race::norms::normalise_work_vector;
use crate::race::owner::Race;

/// The steps, not guarded.
pub fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let work = &race.frame.scratch.work_vector;
    copy_cells(work, &race.shared.car.velocity);
    negate(&work[0]);
    negate(&work[1]);
    negate(&work[2]);
    if compare_80(vector_length(&work), ZERO) & 0x4000 != 0 {
        fill_cells(work, 0);
    }
    normalise_work_vector(race);
    copy_cells(&race.physics.contact.contact_normal, work);
    copy_cells(
        &race.physics.contact.contact_car_speed,
        &race.shared.car.velocity,
    );
    copy_cells(
        &race.physics.contact.contact_turn_speed,
        &race.shared.car.spin,
    );
    fill_cells(&race.physics.contact.contact_point, 0);
    race.physics.contact.contact_slide_limit.set_bits(0);
    contact_force(race, &race.shared.physics_tables.contacts[4], scene);
    { crate::physics::place_sound_e1::place_sound_e1(race) }?;
    race.shared.physics_tables.wall_heading.set(u32::MAX);
    Ok(())
}
