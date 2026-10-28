//! `physics::feel_ground` (0x43F1A8): the work car's ground this step:
//! `physics::note_surface_mode`, `physics::probe_ground`, `physics::lowest_body_point`;
//! then which wheels are off the ground: `dword_71A220` 1 when the first two (the qword
//! at 0x71A5D8) are both 0, `dword_71A224` the same for the last two (0x71A5E0),
//! `dword_71A22C` 1 when all four are; `dword_71A228` the driven ones' (the first pair's,
//! or all four's with `g_carDriveKind` not 0), and `dword_71A1C0` 0 when they are not off;
//! `dword_6141DC` the word at +0x26 of the face under probe point 4 (`dword_71ADD8`). A
//! reading: the wheels in the air, the ground's kind.
//!
//! In:  nothing
//! Out: eax the face's word, ebx the face, esi its address; ecx, edx as `probe_ground` /
//!      `lowest_body_point` leave them; no flags read
//!
//! It begins with a call: `.private_copy` (its three calls relocated) with
//! `.manual_patch`.
//! Callers: `sub_43B635` (0x43B6F6, 0x43B860, 0x43B8FB), `sub_45D737`, `sub_45D7B6`
//! (twice), `sub_45E3BF`, `sub_45EFF3`. Refuses (before anything) more than 0x40 probe
//! points; `probe_ground`'s refusals mid-way (it puts back what it wrote; `dword_71A6C8` is
//! the original's to write again).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::course_store::COURSE;
use rc2k_core::physics::walls::WheelsOff;

/// The Rust version. The registers it leaves (eax the face's word +0x26, ebx the face,
/// esi its address) are read by none of its callers.
// Converted: `rc2k_core::physics::walls::WheelsOff`.
pub unsafe fn feel_ground() -> Result<(), Refused> {
    if CAR.probe_count.get() > rc2k_core::physics::walls::PROBE_ROOM {
        return Err(BadInput("more than 0x40 probe points").into());
    }
    crate::physics::note_surface_mode::note_surface_mode();
    crate::physics::probe_ground::probe_ground()?;
    crate::physics::lowest_body_point::lowest_body_point()?;
    let off = WheelsOff::of([
        CAR.wheel_squeeze[0].bits(),
        CAR.wheel_squeeze[1].bits(),
        CAR.wheel_squeeze[2].bits(),
        CAR.wheel_squeeze[3].bits(),
    ]);
    CAR.front_wheels_off.set(u32::from(off.front));
    CAR.rear_wheels_off.set(u32::from(off.back));
    CAR.all_wheels_off.set(u32::from(off.all));
    let driven = u32::from(off.driven(CAR.drive_kind.get()));
    CAR.driven_wheels_off.set(driven);
    if driven == 0 {
        CAR.stuck_count.set(0);
    }
    let face = (CAR.probe_point_face[4].get() << 6).wrapping_add(COURSE.course_faces.get());
    PHYSICS
        .feel_word
        .set(u32::from(crate::race::course_places::read::<u16>(
            face + 0x26,
            "a face's feel word",
        )?));
    Ok(())
}
