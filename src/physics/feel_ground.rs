//! The work car's ground this step: `physics::note_surface_mode`, `physics::probe_ground`,
//! `physics::lowest_body_point`; then which wheels are off the ground: `front_wheels_off` 1
//! when the first two are both 0, `rear_wheels_off` the same for the last two, `all_wheels_off`
//! 1 when all four are; `driven_wheels_off` the driven ones' (the first pair's, or all four's
//! with the drive kind not 0), and `stuck_count` 0 when they are not off; the face word
//! under probe point 4 (`Car.probe_point_face[4]`) is read, to refuse a face off the course. A reading: the wheels in the air,
//! the ground's kind.
//!
//! Refuses (before anything) more than 0x40 probe points; `probe_ground`'s refusals mid-way (it
//! puts back what it wrote).

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::step_families::GroundInputs;
use rc2k_core::physics::walls::WheelsOff;

// Core: `rc2k_core::physics::walls::WheelsOff`.
pub(super) fn feel_ground(c: &GroundInputs<'_>, car: &mut Car) -> Result<(), Refused> {
    if car.probe_count > rc2k_core::physics::walls::PROBE_ROOM {
        return Err(BadInput("more than 0x40 probe points").into());
    }
    crate::physics::note_surface_mode::note_surface_mode(car);
    crate::physics::probe_ground::probe_ground(c, car)?;
    crate::physics::lowest_body_point::lowest_body_point(
        car,
        c.numbers,
        c.contact,
        &c.probes.borrow(),
    )?;
    let off = WheelsOff::of([
        car.wheel_squeeze[0].to_bits(),
        car.wheel_squeeze[1].to_bits(),
        car.wheel_squeeze[2].to_bits(),
        car.wheel_squeeze[3].to_bits(),
    ]);
    car.front_wheels_off = u32::from(off.front);
    car.rear_wheels_off = u32::from(off.back);
    car.all_wheels_off = u32::from(off.all);
    let driven = u32::from(off.driven(car.drive_kind));
    car.driven_wheels_off = driven;
    if driven == 0 {
        car.stuck_count = 0;
    }
    // The face's word is not kept; the read still refuses a face outside the course.
    crate::race::course_places::face_field::<u16>(c.course.reads(), car.probe_point_face[4], 0x26)?;
    Ok(())
}
