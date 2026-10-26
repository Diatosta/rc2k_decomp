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
use crate::errors::BadInput;
use crate::physics::car_store::CAR;
use crate::pointers::GamePtr;
use crate::race::kind_store::KIND_TABLES;
use crate::race::view_store::RACE_VIEW;
use crate::session::Dword;
use rc2k_core::drive::camera_views::next_view;

fn at(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a car's block")
}

/// The Rust version on block `s`, the players `players`, the step count `step`.
// Converted: `rc2k_core::drive::camera_views::next_view`.
/// The eax it leaves is read by no caller (liveness: not live after either call).
pub fn next_view_at(s: u32, players: &Dword, step: &Dword) -> Result<(), BadInput> {
    if at(s + 0x694)? != 0 {
        return Ok(());
    }
    let reading = crate::scene::reading::Reading::new();
    let view = next_view(
        at(s + 0x684)?,
        || reading.read::<u32>(s.wrapping_add(0x24), "a car's block"),
        |car| KIND_TABLES.kind_has_views[car as usize].get() != 0, /* dword_5C5AC0, by car (0x22) */
        || players.bits(),
    );
    reading.finish()?;
    put(s.wrapping_add(0x688), view)?;
    set_view_at(s, step)
}

fn put(a: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(a).write(v, "a car's block")
}

/// The Rust version.
pub fn next_camera_view(s: u32) -> Result<(), BadInput> {
    next_view_at(s, &RACE_VIEW.players, &CAR.step_clock)
}
