//! `scene::set_up_textures` (0x44E7A5): the texture memory for everything that is
//! needed, and one texture more for the game's own use.
//!
//! In:  nothing
//! Out: carry clear; eax and ebx as `gfx::allocate_textures` leaves them
//!
//! `scene::build_texture_catalogue`; `dword_624234` = 0; then one system texture of
//! size code 3 and pixel kind 4 without mip levels from `gfx::allocate_textures`
//! (0x4051ED, Direct3D: not ported; eax = 0, ebx = 1, ecx = 3, edx = 4, ebp = 0), whose
//! number (ebx) goes to `dword_624234`. If that one is refused: fatal error
//! 0x80600943.
//! Two callers.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, game_write, rva};
use crate::scene::build_texture_catalogue::build_texture_catalogue_in_game;

// ---- game data ----

const OWN_TEXTURE_RVA: usize = rva(0x624234); // dword_624234
const ALLOCATE_RVA: usize = rva(0x4051ED); // gfx::allocate_textures

const NO_OWN_TEXTURE: FatalError = FatalError {
    code: 0x8060_0943,
    message: "cant alloc sys video texture",
};

// ---- Rust version ----

/// `catalogue`: what stands for `scene::build_texture_catalogue`; `allocate`: for
/// `gfx::allocate_textures`, given the registers it is entered with, answering false
/// if it refuses.
pub unsafe fn set_up_textures(
    r: &mut Regs,
    catalogue: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
    allocate: &mut dyn FnMut(&mut Regs) -> bool,
) -> Result<(), Refused> {
    unsafe {
        catalogue(r)?;
        game_write(OWN_TEXTURE_RVA, 0u32);
        (r.eax, r.ebx, r.ecx, r.edx, r.ebp) = (0, 1, 3, 4, 0);
        if !allocate(r) {
            return Err(NO_OWN_TEXTURE.into());
        }
        game_write(OWN_TEXTURE_RVA, r.ebx);
        Ok(())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let done = unsafe {
        set_up_textures(r, &mut |_| build_texture_catalogue_in_game(), &mut |r| {
            crate::calls::call(ALLOCATE_RVA, r);
            !r.carry()
        })
    };
    match done {
        Ok(()) => r.set_carry(false),
        Err(e) => {
            *r = entry;
            return Err(e);
        }
    }
    Ok(())
}
