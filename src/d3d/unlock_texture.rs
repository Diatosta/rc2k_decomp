//! `d3d::unlock_texture` (0x41AB02): system texture eax unlocked: with a surface (+0x14)
//! that is locked (+0x60 1), its `Unlock` (no rectangle; a failure is fatal error
//! 0x8021A173) and +0x60 0.
//!
//! Takes: the texture
//!
//! Callers: those of `d3d::lock_texture`. The unlock is the surface's own method. Refuses
//! a texture past the pool's 0x140, before anything.

use crate::d3d::texture_lock::*;
use crate::errors::{BadInput, Refused};
use crate::framework::Flags;

/// The Rust version (see the header): texture `texture` unlocked; the flags it leaves
/// (every register is kept).
pub fn unlock_texture(texture: u32) -> Result<Flags, Refused> {
    use rc2k_core::d3d::textures::{UNLOCK_FAILED, Unlock, unlock};
    // Converted: `rc2k_core::d3d::textures::unlock`.
    let r = record(texture);
    let surface = if texture < 0x140 { field(r, 0x14) } else { 0 };
    let (carry, zero) = match unlock(texture, surface, || field(r, 0x60)).map_err(BadInput::from)? {
        Unlock::Flags(carry, zero) => (carry, zero),
        Unlock::Unlock => {
            if crate::platform_3d::unlock_surface(surface) != 0 {
                return Err(fatal(UNLOCK_FAILED).into());
            }
            put_field(r, 0x60, 0);
            (false, true)
        }
    };
    Ok(Flags { carry, zero })
}
