//! `d3d::copy_screen_to_texture`: while the frame's `mirror_wanted` is set (the view's mirror
//! picture wanted), the back surface is copied, stretched, into the mirror picture's system
//! texture (`platform_3d::copy_screen_to_system_texture`). Caller: `race::draw_race_scene`
//! (one view).
use crate::race::owner_frame::Frame;
use crate::scene::owner::Scene;

pub fn copy_screen_to_texture(frame: &Frame, scene: &Scene, screen: &crate::session::Screen) {
    if frame.frame_work.get().mirror_wanted.is_set() {
        crate::platform_3d::copy_screen_to_system_texture(
            scene.drawing.scene.get().mirror_picture.get(),
            screen,
        );
    }
}
