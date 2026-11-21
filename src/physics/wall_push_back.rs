//! Pushes the work car back off a wall against its motion: minus its speed (`velocity`) goes into
//! `work_vector` (0 when its length, `physics::vector_length`, is 0), made one long
//! (`race::normalise_work_vector`) as the normal; the speed and the turn speed taken, the point 0,
//! no slide limit; `physics::contact_force` into contact record 4, `physics::place_sound_e1`.
//!
//! Refuses what `place_sound_e1` refuses (what was written is put back).

use crate::errors::Refused;
use crate::math::x87::{compare_80, length_80};
use crate::physics::contact_force::contact_force;
use crate::physics::flow::{ZERO_BITS as ZERO, negated_velocity};
use crate::physics::step_families::ContactInputs;
use crate::physics::vector_bits::bits_of;
use crate::race::norms::normalise_work_vector;

/// The steps, not guarded.
pub fn steps(c: &ContactInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    {
        let mut scratch = c.frame.scratch_mut();
        scratch.work_vector = negated_velocity(c.car);
        if compare_80(length_80(scratch.work_vector), ZERO) & 0x4000 != 0 {
            scratch.work_vector = [0.0f32.to_bits(); 3];
        }
        normalise_work_vector(c.numbers, &mut scratch, scene);
        c.contact.borrow_mut().contact_normal = scratch.work_vector;
    }
    {
        let mut work = c.contact.borrow_mut();
        work.contact_car_speed = bits_of(&c.car.velocity);
        work.contact_turn_speed = bits_of(&c.car.spin);
        work.contact_point = [0; 3];
        work.contact_slide_limit = 0;
    }
    contact_force(c.contact, c.numbers, c.tables, c.car, &c.tables.contacts[4]);
    { crate::physics::place_sound_e1::place_sound_e1(&c.sound) }?;
    Ok(())
}
