//! `d3d::lock_texture` (0x41AA4B): system texture eax locked, made the blitters' surface.
//! With no surface (+0x14 0) nothing; locked already (+0x60 1): fatal error 0x8021A171;
//! else its surface's `Lock` (no rectangle, the description `dword_5ABA18`, flags 1): a
//! failure is fatal error 0x8021A172; +0x60 1 and `d3d::texture_surface_info`.
//!
//! In:  eax the texture
//! Out: the flags of `cmp [record + 0x60], 1` (carry: not locked); every register kept
//!
//! Callers: the name and time plates, the mud, the mirror, the marks' masks, ... (most
//! `jb`). The lock is the surface's own method (`d3d::texture_lock`): DirectDraw in the
//! game, `platform_3d`'s in the standalone. Refuses a texture past the pool's 0x140,
//! before anything.

use crate::d3d::texture_lock::*;
use crate::errors::{BadInput, Refused};
use crate::framework::Flags;
use crate::gfx::owner::gfx;

/// The Rust version (see the header): texture `texture` locked; the flags it leaves
/// (every register is kept).
pub fn lock_texture(texture: u32) -> Result<Flags, Refused> {
    use rc2k_core::d3d::textures::{LOCK_FAILED, LOCKED_TWICE, Lock, lock};
    // Converted: `rc2k_core::d3d::textures::lock`.
    let r = record(texture);
    let surface = if texture < 0x140 { field(r, 0x14) } else { 0 };
    match lock(texture, surface, || field(r, 0x60)).map_err(BadInput::from)? {
        Lock::Nothing => {}
        Lock::Twice => return Err(fatal(LOCKED_TWICE).into()),
        Lock::Lock => {
            if crate::platform_3d::lock_surface(surface, &gfx().card.upload.description) != 0 {
                return Err(fatal(LOCK_FAILED).into());
            }
            put_field(r, 0x60, 1);
            crate::d3d::texture_surface_info::texture_surface_info(r)?;
        }
    }
    let (carry, zero) = compared_with_one(field(r, 0x60));
    Ok(Flags { carry, zero })
}
