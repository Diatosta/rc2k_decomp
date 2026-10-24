//! `physics::scene_point_hits_car` (0x45E7BF): `physics::point_hits_car` for the scene
//! point whose number is the dword at esi+4 (`g_scenePoints` + 12 n), with no offset:
//! `dword_614108` + 1, `dword_6D3B40` .. = 0.
//!
//! In:  esi the record (+4 the point's number), ebp the object
//! Out: CF as `point_hits_car` leaves it; ebx the number, esi the point; the rest kept
//!
//! Caller: `sub_45E3BF` (0x45E48C, `jb`).

use crate::physics::flow::{fill3, g, p};
use crate::physics::point_hits_car::point_hits_car;

/// `g_scenePoints`: the pointer to the scene's points (three whole numbers each).
pub const SCENE_POINTS: usize = 0x5CCCC0;

/// The Rust version: (inside, ebx, esi).
pub unsafe fn scene_point_hits_car(record: u32, object: u32) -> (bool, u32, u32) {
    let number = unsafe { ((record + 4) as *const u32).read_unaligned() };
    let point = number.wrapping_mul(12).wrapping_add(g(SCENE_POINTS));
    p(0x614108, g(0x614108).wrapping_add(1));
    fill3(0x6D3B40, 0);
    (unsafe { point_hits_car(point, object) }, number, point)
}
