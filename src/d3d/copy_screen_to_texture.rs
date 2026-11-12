//! `d3d::copy_screen_to_texture` (0x455D38): while the frame's `mirror_wanted` is set (the
//! view's mirror picture wanted), the back surface is copied into the mirror picture's
//! system texture (its record's surface +0x14) by DirectDraw's `Blt` without rectangles
//! (stretched to the texture).
//!
//! Caller: `race::draw_race_scene` (one view). Platform: the original always runs. In the
//! standalone program the screen is copied into the texture's memory of ours
//! (`platform_3d::copy_screen_to_system_texture`).
use crate::race::owner_frame::Frame;
use crate::scene::owner::Scene;

pub fn copy_screen_to_texture(frame: &Frame, scene: &Scene) {
    if frame.frame_work.get().mirror_wanted.bits() != 0 {
        crate::platform_3d::copy_screen_to_system_texture(
            scene.drawing.scene.get().mirror_picture.bits(),
        );
    }
}
