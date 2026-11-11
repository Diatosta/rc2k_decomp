//! `physics::feel_ground` (0x43F1A8): the work car's ground this step:
//! `physics::note_surface_mode`, `physics::probe_ground`, `physics::lowest_body_point`;
//! then which wheels are off the ground: `dword_71A220` 1 when the first two (the qword
//! at 0x71A5D8) are both 0, `dword_71A224` the same for the last two (0x71A5E0),
//! `dword_71A22C` 1 when all four are; `dword_71A228` the driven ones' (the first pair's,
//! or all four's with `g_carDriveKind` not 0), and `dword_71A1C0` 0 when they are not off;
//! `dword_6141DC` the word at +0x26 of the face under probe point 4 (`dword_71ADD8`). A
//! reading: the wheels in the air, the ground's kind.
//!
//! It begins with a call: `.private_copy` (its three calls relocated) with
//! `.manual_patch`.
//! Callers: `physics::car_physics_step` (0x43B6F6, 0x43B860, 0x43B8FB), `physics::crossed_edge`,
//! `physics::wall_contacts`
//! (twice), `physics::face_contact`, `physics::ground_contact`. Refuses (before anything) more than
//! 0x40 probe points; `probe_ground`'s refusals mid-way (it puts back what it wrote; `dword_71A6C8`
//! is the original's to write again).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use rc2k_core::physics::walls::WheelsOff;

// Converted: `rc2k_core::physics::walls::WheelsOff`.
pub fn feel_ground(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if race.shared.car.probe_count.get() > rc2k_core::physics::walls::PROBE_ROOM {
        return Err(BadInput("more than 0x40 probe points").into());
    }
    crate::physics::note_surface_mode::note_surface_mode(&race.shared.car);
    crate::physics::probe_ground::probe_ground(race, &race.shared.car, scene)?;
    crate::physics::lowest_body_point::lowest_body_point(
        &race.shared.car,
        &race.course.numbers,
        &race.physics.contact,
    )?;
    let off = WheelsOff::of([
        race.shared.car.wheel_squeeze[0].bits(),
        race.shared.car.wheel_squeeze[1].bits(),
        race.shared.car.wheel_squeeze[2].bits(),
        race.shared.car.wheel_squeeze[3].bits(),
    ]);
    race.shared.car.front_wheels_off.set(u32::from(off.front));
    race.shared.car.rear_wheels_off.set(u32::from(off.back));
    race.shared.car.all_wheels_off.set(u32::from(off.all));
    let driven = u32::from(off.driven(race.shared.car.drive_kind.get()));
    race.shared.car.driven_wheels_off.set(driven);
    if driven == 0 {
        race.shared.car.stuck_count.set(0);
    }
    race.shared
        .physics_tables
        .feel_word
        .set(u32::from(crate::race::course_places::face_field::<u16>(
            race.shared.car.probe_point_face[4].get(),
            0x26,
            "a face's feel word",
        )?));
    Ok(())
}
