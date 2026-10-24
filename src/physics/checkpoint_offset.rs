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
use crate::framework::{game_read, game_write, rva};
use crate::math::turn_point_xz::turn_point_xz;
use rc2k_core::physics::progress::checkpoint_ahead;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const CHECKPOINT: usize = 0x71A070; // g_car + 0x98
const COUNT: usize = 0x623640; // g_checkpointCount
const ANGLES: usize = 0x9ED808; // dword_9ED808
const CHECK_FACES: usize = 0x9E3738; // g_checkpointFaces
const FACES: usize = 0x5CCCBC; // g_courseFaces
const POINTS: usize = 0x5CCCC0; // g_scenePoints
const OFFSET: usize = 0x71A5F8; // g_car + 0x620

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version.
// Converted: `rc2k_core::physics::progress::checkpoint_ahead` (the turn stays
// `math::turn_point_xz`'s).
pub unsafe fn checkpoint_offset() -> Result<(), Refused> {
    let c = g(CHECKPOINT);
    if checkpoint_ahead(c, g(COUNT))?.is_none() {
        p(OFFSET, 0);
        p(OFFSET + 4, 0);
        return Ok(());
    }
    if g(FACES) == 0 || g(POINTS) == 0 {
        return Err(BadInput("no course faces or points").into());
    }
    unsafe {
        let angle = g(ANGLES + 4 * c as usize);
        let face = g(CHECK_FACES + 4 * c as usize);
        let f = (face << 6).wrapping_add(g(FACES));
        let point = at(f + 4);
        let pt = point.wrapping_mul(12).wrapping_add(g(POINTS));
        let x = g(0x71A10C).wrapping_sub(at(pt));
        let z = g(0x71A114).wrapping_sub(at(pt + 8));
        let [x2, _, z2, _] = turn_point_xz(x as i32, point as i32, z as i32, angle.wrapping_neg());
        p(OFFSET, x2);
        p(OFFSET + 4, z2);
        Ok(())
    }
}
