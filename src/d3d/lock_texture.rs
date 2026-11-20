//! `d3d::lock_texture`: a system texture locked, made the blitters' surface. Takes the texture.
//! With no surface nothing; locked already: fatal error 0x8021A171; a failure to lock is fatal
//! error 0x8021A172.
//!
//! Refuses a texture past the pool's 0x140, before anything.

use crate::d3d::texture_lock::*;
use crate::errors::{BadInput, Refused};
use crate::gfx::texture_upload::RecordCell;

/// Locks texture `texture`: whether it is locked after (false for one without a surface).
pub fn lock_texture_in(gfx: &crate::gfx::owner::Gfx, texture: u32) -> Result<bool, Refused> {
    use rc2k_core::d3d::textures::{LOCK_FAILED, LOCKED_TWICE, Lock, lock};
    // Logic: `rc2k_core::d3d::textures::lock`.
    let r = record(gfx, texture);
    let surface = r.map_or(0, RecordCell::first_surface);
    match lock(texture, surface, || r.map_or(0, RecordCell::locked)).map_err(BadInput::from)? {
        Lock::Nothing => {}
        Lock::Twice => return Err(fatal(LOCKED_TWICE).into()),
        Lock::Lock => {
            if crate::d3d::device::lock_surface(surface, &gfx.card.upload.description) != 0 {
                return Err(fatal(LOCK_FAILED).into());
            }
            // Only a texture in the pool gets here.
            let r = r.expect("lock chose Lock for a texture in the pool");
            r.set_locked(1);
            crate::d3d::texture_surface_info::texture_surface_info(r, gfx)?;
        }
    }
    Ok(r.map_or(0, RecordCell::locked) != 0)
}
