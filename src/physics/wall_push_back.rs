//! `physics::wall_push_back` (0x45D8FB): the work car pushed back off a wall against its
//! motion: minus its speed (`velocity` ..) in `g_workVector` (0 when its length is 0,
//! `physics::vector_length`), made one long (`race::normalise_work_vector`) as the
//! normal; the speed and the turn speed taken, the point 0, no slide limit;
//! `physics::contact_force` into contact record 4 (`g_contactRecords` + 0x80),
//! `physics::place_sound_e1`; `wall_heading` = -1.
//!
//! Callers: `physics::wall_contacts`, `physics::wall_edge_contact`. Refuses what
//! `place_sound_e1` refuses (what was written put back).

use crate::errors::Refused;
use crate::math::x87::compare_80;
use crate::physics::contact_force::contact_force;
use crate::physics::flow::{ZERO_BITS as ZERO, copy_cells, fill_cells, negate};
use crate::physics::step_families::ContactInputs;
use crate::physics::vector_length::vector_length;
use crate::race::norms::normalise_work_vector;

/// The steps, not guarded.
pub fn steps(c: &ContactInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let work = &c.scratch.work_vector;
    copy_cells(work, &c.car.velocity);
    negate(&work[0]);
    negate(&work[1]);
    negate(&work[2]);
    if compare_80(vector_length(&work), ZERO) & 0x4000 != 0 {
        fill_cells(work, 0.0);
    }
    normalise_work_vector(c.numbers, c.scratch, scene);
    copy_cells(&c.contact.contact_normal, work);
    copy_cells(&c.contact.contact_car_speed, &c.car.velocity);
    copy_cells(&c.contact.contact_turn_speed, &c.car.spin);
    fill_cells(&c.contact.contact_point, 0.0);
    c.contact.contact_slide_limit.set(0.0);
    contact_force(
        c.contact,
        c.numbers,
        c.tables,
        c.car,
        &c.tables.contacts[4],
        scene,
    );
    { crate::physics::place_sound_e1::place_sound_e1(&c.sound) }?;
    c.tables.wall_heading.set(u32::MAX);
    Ok(())
}
