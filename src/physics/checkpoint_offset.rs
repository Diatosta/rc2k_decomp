//! `physics::checkpoint_offset` (0x4415E7): the work car's place against its checkpoint
//! (`g_car` +0x98). With a checkpoint after it (+0x98 + 1 below `g_checkpointCount`, by
//! the sign of the difference): the checkpoint's face (`g_checkpointFaces`), that face's
//! first point (`g_courseFaces` +4, `g_scenePoints`), the car's x and z less the point's,
//! turned by minus the checkpoint's angle (`dword_9ED808`, `math::turn_point_xz`) into
//! +0x620 and +0x624. Else both 0.
//!
//! In:  nothing
//! Out: eax: x' (or +0x98 + 1); with a checkpoint after it also ebx the point's number,
//!      ecx z', edx as `turn_point_xz` leaves it, esi the point; the rest kept; no flags
//!      read (the caller returns)
//!
//! Caller: `sub_44157B`. Refuses (before anything) a checkpoint below 0 or past 0x1418, or
//! no faces or points.

use crate::errors::{BadInput, Refused};
use crate::math::turn_point_xz::turn_point_xz;
use crate::physics::car_store::CAR;
use crate::race::course_store::COURSE;
use crate::race::vars_store::RACE_VARS;
use rc2k_core::physics::progress::checkpoint_ahead;

fn at(a: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(a).read::<u32>("a checkpoint face or scene point")
}

/// The Rust version.
// Converted: `rc2k_core::physics::progress::checkpoint_ahead` (the turn stays
// `math::turn_point_xz`'s).
pub fn checkpoint_offset() -> Result<(), Refused> {
    let c = CAR.checkpoint.get();
    if checkpoint_ahead(c, RACE_VARS.checkpoint_count.get())?.is_none() {
        CAR.checkpoint_offset[0].set(0);
        CAR.checkpoint_offset[1].set(0);
        return Ok(());
    }
    if COURSE.course_faces.get() == 0 || COURSE.scene_points.get() == 0 {
        return Err(BadInput("no course faces or points").into());
    }
    let angle = crate::race::checkpoints_store::angle(c as usize).get();
    let face = crate::race::checkpoints_store::face(c as usize).get();
    let f = (face << 6).wrapping_add(COURSE.course_faces.get());
    let point = at(f + 4)?;
    let pt = point
        .wrapping_mul(12)
        .wrapping_add(COURSE.scene_points.get());
    let x = CAR.position[0].get().wrapping_sub(at(pt)?);
    let z = CAR.position[2].get().wrapping_sub(at(pt + 8)?);
    let [x2, _, z2, _] = turn_point_xz(x as i32, point as i32, z as i32, angle.wrapping_neg());
    CAR.checkpoint_offset[0].set(x2);
    CAR.checkpoint_offset[1].set(z2);
    Ok(())
}
