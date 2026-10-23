//! `scene::load_wanted_textures` (0x4256C8): works out what every texture slot should hold and loads what is missing.
//!
//! In:  nothing
//! Out: as `scene::load_slot_textures` leaves them
//!
//! `scene::assign_kind_textures`, `scene::map_texture_places`,
//! `scene::load_slot_textures`.
//! One caller, `scene::reload_slot_textures`.
//! Manual patch with a private copy: the function begins with a call.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;

/// The calls made, as places in `CALLEES`.
const ORDER: [usize; 3] = [0, 1, 2];

// ---- Rust version ----

/// `step`: what stands for each call, by its place in `ORDER`; it is entered with the
/// registers the call before left.
pub unsafe fn load_wanted_textures(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for n in 0..ORDER.len() {
        step(n, r)?;
    }
    Ok(())
}

/// With the Rust versions of what it calls.
pub unsafe fn load_wanted_textures_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        load_wanted_textures(r, &mut |n, r| {
            match n {
                0 => assign_kind_textures()?.apply(r),
                1 => map_texture_places().apply(r),
                _ => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
            }
            Ok(())
        })
    }
}
