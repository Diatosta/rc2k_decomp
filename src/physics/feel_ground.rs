//! `physics::feel_ground` (0x43F1A8): the work car's ground this step:
//! `physics::note_surface_mode`, `physics::probe_ground`, `physics::lowest_body_point`;
//! then which wheels are off the ground: `front_wheels_off` 1 when the first two (the qword
//! at 0x71A5D8) are both 0, `rear_wheels_off` the same for the last two (0x71A5E0),
//! `all_wheels_off` 1 when all four are; `driven_wheels_off` the driven ones' (the first pair's,
//! or all four's with `g_carDriveKind` not 0), and `stuck_count` 0 when they are not off;
//! `feel_word` the word at +0x26 of the face under probe point 4 (`Car.probe_point_face[4]`). A
//! reading: the wheels in the air, the ground's kind.
//!
//! It begins with a call: `.private_copy` (its three calls relocated) with
//! `.manual_patch`.
//! Callers: `physics::car_physics_step` (0x43B6F6, 0x43B860, 0x43B8FB), `physics::crossed_edge`,
//! `physics::wall_contacts`
//! (twice), `physics::face_contact`, `physics::ground_contact`. Refuses (before anything) more than
//! 0x40 probe points; `probe_ground`'s refusals mid-way (it puts back what it wrote; `camera_mode_is_2_or_3`
//! is the original's to write again).

use crate::errors::{BadInput, Refused};
use crate::physics::step_families::GroundInputs;
use rc2k_core::physics::walls::WheelsOff;

// Converted: `rc2k_core::physics::walls::WheelsOff`.
pub fn feel_ground(
    c: &GroundInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if c.car.probe_count.get() > rc2k_core::physics::walls::PROBE_ROOM {
        return Err(BadInput("more than 0x40 probe points").into());
    }
    crate::physics::note_surface_mode::note_surface_mode(c.car);
    crate::physics::probe_ground::probe_ground(c, c.car, scene)?;
    crate::physics::lowest_body_point::lowest_body_point(c.car, c.numbers, c.contact)?;
    let off = WheelsOff::of([
        c.car.wheel_squeeze[0].bits(),
        c.car.wheel_squeeze[1].bits(),
        c.car.wheel_squeeze[2].bits(),
        c.car.wheel_squeeze[3].bits(),
    ]);
    c.car.front_wheels_off.set(u32::from(off.front));
    c.car.rear_wheels_off.set(u32::from(off.back));
    c.car.all_wheels_off.set(u32::from(off.all));
    let driven = u32::from(off.driven(c.car.drive_kind.get()));
    c.car.driven_wheels_off.set(driven);
    if driven == 0 {
        c.car.stuck_count.set(0);
    }
    c.tables
        .feel_word
        .set(u32::from(crate::race::course_places::face_field::<u16>(
            c.course,
            c.car.probe_point_face[4].get(),
            0x26,
        )?));
    Ok(())
}
