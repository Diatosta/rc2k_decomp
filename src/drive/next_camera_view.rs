//! `drive::next_camera_view` (0x43AA38): unless the block's (esi) view is held (+0x694),
//! the next view: +0x684 + 1, but views 2 and 3 become 4 when the car has none
//! (`dword_5C5AC0[car]` 0, or the car number +0x24 past 0x21) or with two players
//! (`dword_63DB94` 2); 7 and above wrap to 0; into +0x688, then
//! `drive::set_camera_view`. A reading: the change-view key.
//!
//! In:  esi the block
//! Out: eax the step count (when a view was taken), else kept; no flags read
//!
//! Callers: `sub_43A35A` (0x43A454), `sub_43A649` (0x43A66C).

use crate::drive::set_camera_view::set_view_at;
use crate::framework::{game_read, rva};
use crate::physics::car_store::CAR;
use crate::race::view_store::RACE_VIEW;
use rc2k_core::drive::camera_views::next_view;
const HAS_VIEWS_RVA: usize = rva(0x5C5AC0); // dword_5C5AC0, by car (0x22)

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version on block `s`, the players at `players`, the step count at `step`.
// Converted: `rc2k_core::drive::camera_views::next_view`.
/// The eax it leaves is read by no caller (liveness: not live after either call).
pub unsafe fn next_view_at(s: u32, players: u32, step: u32) {
    unsafe {
        if at(s + 0x694) != 0 {
            return;
        }
        let view = next_view(
            at(s + 0x684),
            || at(s + 0x24),
            |car| game_read::<u32>(HAS_VIEWS_RVA + 4 * car as usize) != 0,
            || at(players),
        );
        (s.wrapping_add(0x688) as *mut u32).write_unaligned(view);
        set_view_at(s, step)
    }
}

/// The Rust version.
pub unsafe fn next_camera_view(s: u32) {
    unsafe {
        next_view_at(
            s,
            RACE_VIEW.players.addr() as u32,
            CAR.step_clock.addr() as u32,
        )
    }
}
