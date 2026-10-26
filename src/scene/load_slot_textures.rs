//! `scene::load_slot_textures` (0x425778): every texture slot in use gets the texture
//! it wants.
//!
//! In:  nothing
//! Out: ecx = the number of slots; everything else preserved
//!
//! `scene::load_slot_texture` for the slots 0 .. `dword_5DC970`.
//! Six callers.
//! Refuses no slots (the original goes through 2^32) and more than the table has.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{ECX, Regs, game_read};
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::textures::SLOT_COUNT_RVA;
use crate::screens::harness::Left;

// ---- Rust version ----

// Converted: `rc2k_core::scene::textures::slots_in_use`
pub unsafe fn load_slot_textures(
    load: &mut dyn FnMut(u32) -> Result<(), BadInput>,
) -> Result<Left, BadInput> {
    unsafe {
        let count = rc2k_core::scene::textures::slots_in_use(game_read(SLOT_COUNT_RVA))?;
        for slot in 0..count {
            load(slot)?;
        }
        Ok(Left::nothing().with(ECX, count))
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    match unsafe { load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry)) } {
        Ok(left) => left.apply(r),
        Err(e) => {
            let e: Refused = e.into();
            return Err(e);
        }
    }
    Ok(())
}
