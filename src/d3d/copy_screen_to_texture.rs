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
//! (`platform_3d::copy_screen_to_system_texture`).

use crate::race::frame_store::FRAME_WORK;
use crate::scene::draw_store::DRAW;

/// The Rust version (see the header).
pub fn copy_screen_to_texture() {
    if FRAME_WORK.mirror_wanted.bits() != 0 {
        crate::platform_3d::copy_screen_to_system_texture(DRAW.mirror_picture.bits());
    }
}
