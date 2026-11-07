//! `d3d::copy_to_device_texture` (0x41A8F9): a system texture's levels into a video
//! texture.
//!
//! Takes: the system texture, the video texture (record numbers)
//! The video record's +0x64 is set to the system texture; the device's batch is flushed
//! (`sub_41A971`: SetRenderState 0x32, FLUSHBATCH); then each level of the system record
//! (+0x10 of them): its surface page-locked (+0x38, `PageLock`), blitted (`Blt`, waiting)
//! into the video record's surface of that level where both have one, page-unlocked.
//! Callers: `scene::load_texture`, `sub_41A9FC`, `draw_tyre_change`,
//! `race::draw_viewer_panel`.
//! Platform: the original always runs. In the standalone program the platform's textures
//! do it (`platform_3d::hold_texture`, as `scene::load_texture` there; before 2026-10-19
//! the standalone skipped it: the viewer's panel did not show).

/// System texture `system`'s levels into video texture `video`.
pub fn copy_to_device_texture(system: u32, video: u32) {
    crate::platform_3d::hold_texture("d3d::copy_to_device_texture", system, video);
}
