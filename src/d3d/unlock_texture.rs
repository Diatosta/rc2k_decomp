//! `d3d::unlock_texture` (0x41AB02): a system texture unlocked: with a surface (+0x14)
//! that is locked (+0x60 1), its `Unlock` (no rectangle; a failure is fatal error
//! 0x8021A173) and +0x60 0.
//!
//! Takes: the texture
//!
//! Callers: those of `d3d::lock_texture`. The unlock is the surface's own method. Refuses
//! a texture past the pool's 0x140, before anything.

use crate::d3d::texture_lock::*;
use crate::errors::{BadInput, Refused};
use crate::framework::CarryZeroFlags;
use crate::gfx::texture_upload::RecordCell;

/// Texture `texture` unlocked; the flags it leaves
/// (every register is kept).
pub fn unlock_texture(texture: u32) -> Result<CarryZeroFlags, Refused> {
    use rc2k_core::d3d::textures::{UNLOCK_FAILED, Unlock, unlock};
    // Converted: `rc2k_core::d3d::textures::unlock`.
    let r = record(texture);
    let surface = r.map_or(0, RecordCell::first_surface);
    let (carry, zero) = match unlock(texture, surface, || r.map_or(0, RecordCell::locked))
        .map_err(BadInput::from)?
    {
        Unlock::Flags(carry, zero) => (carry, zero),
        Unlock::Unlock => {
            if crate::platform_3d::unlock_surface(surface) != 0 {
                return Err(fatal(UNLOCK_FAILED).into());
            }
            // Only a texture in the pool gets here.
            r.expect("unlock chose Unlock for a texture in the pool")
                .set_locked(0);
            (false, true)
        }
    };
    Ok(CarryZeroFlags { carry, zero })
}
