//! A system texture locked for the duration of a closure, then unlocked and copied to its video
//! texture: the one way the race draws into a texture, so an early `?` cannot leave it locked.

use crate::errors::Refused;

/// When the texture is copied to the video texture after the closure.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CopyWhen {
    /// Only if the texture was locked (a texture without a surface is left alone).
    Locked,
    /// Whether or not it was locked.
    Always,
}

/// Locks `texture`, runs `draw` with whether it is locked, unlocks it and copies it to
/// `video_texture` as `copy` says. If `draw` fails the texture is unlocked (best effort) and its
/// error returned; nothing is copied.
pub fn with_locked_texture<T>(
    gfx: &crate::gfx::owner::Gfx,
    texture: u32,
    video_texture: u32,
    copy: CopyWhen,
    draw: impl FnOnce(bool) -> Result<T, Refused>,
) -> Result<T, Refused> {
    let locked = crate::d3d::lock_texture::lock_texture_in(gfx, texture)?;
    let drawn = draw(locked);
    if drawn.is_err() {
        let _ = crate::d3d::unlock_texture::unlock_texture(gfx, texture);
        return drawn;
    }
    crate::d3d::unlock_texture::unlock_texture(gfx, texture)?;
    if locked || copy == CopyWhen::Always {
        crate::d3d::copy_to_device_texture::copy_to_device_texture(texture, video_texture);
    }
    drawn
}
