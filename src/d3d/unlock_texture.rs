//! `d3d::unlock_texture`: a system texture unlocked: with a surface (`surfaces[0]`) that is locked
//! (`locked` 1), its unlock (a failure is fatal error 0x8021A173) and `locked` 0. Takes the
//! texture.
//!
//! Refuses a texture past the pool's 0x140, before anything.

use crate::d3d::texture_lock::*;
use crate::errors::{BadInput, Refused};
use crate::gfx::texture_upload::RecordCell;

/// Unlocks texture `texture` if it is locked.
pub fn unlock_texture(gfx: &crate::gfx::owner::Gfx, texture: u32) -> Result<(), Refused> {
    use rc2k_core::d3d::textures::{UNLOCK_FAILED, Unlock, unlock};
    // Logic: `rc2k_core::d3d::textures::unlock`.
    let r = record(gfx, texture);
    let surface = r.map_or(0, RecordCell::first_surface);
    match unlock(texture, surface, || r.map_or(0, RecordCell::locked)).map_err(BadInput::from)? {
        Unlock::Nothing => {}
        Unlock::Unlock => {
            if crate::d3d::device::unlock_surface(surface) != 0 {
                return Err(fatal(UNLOCK_FAILED).into());
            }
            // Only a texture in the pool gets here.
            r.expect("unlock chose Unlock for a texture in the pool")
                .set_locked(0);
        }
    }
    Ok(())
}
