//! `d3d::copy_to_device_texture`: a system texture's levels into a video texture. Takes the
//! system texture and the video texture (record numbers).
//!
//! The video record's +0x64 is set to the system texture, and the platform's textures do the copy
//! (`d3d::device::hold_texture`, as `scene::load_texture`).

/// System texture `system`'s levels into video texture `video`.
pub fn copy_to_device_texture(system: u32, video: u32) {
    crate::d3d::device::hold_texture("d3d::copy_to_device_texture", system, video);
}
