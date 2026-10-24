//! `d3d::copy_screen_to_texture` (0x455D38): while `dword_6415E0` is set (the view's
//! mirror picture wanted, from its block's +0x6F0), the back surface (`g_backSurface`)
//! copied into the mirror picture's system texture (`dword_624234`, its record's surface
//! +0x14) by DirectDraw's `Blt` without rectangles (stretched to the texture).
//!
//! In:  nothing
//! Out: ebx the record's kind's address and eax .. edx as `Blt` leaves them on the copy's
//!      way; no flags read (its caller goes on with a `mov`)
//!
//! Caller: `race::draw_race_scene` (one view). Platform: the original always runs. In the
//! standalone program the screen is copied into the texture's memory of ours
//! (`platform_3d::copy_screen_to_system_texture`). No self-test.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{game_read, rva};

const WANTED_RVA: usize = rva(0x6415E0); // dword_6415E0
const PICTURE_RVA: usize = rva(0x624234); // dword_624234: the mirror picture's texture

/// The Rust version (see the header).
pub fn copy_screen_to_texture() {
    if unsafe { game_read::<u32>(WANTED_RVA) } != 0 {
        crate::platform_3d::copy_screen_to_system_texture(unsafe { game_read(PICTURE_RVA) });
    }
}
