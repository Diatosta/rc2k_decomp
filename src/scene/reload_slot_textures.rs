//! `scene::reload_slot_textures` (0x425669): every texture slot is loaded anew ("reload video plates").
//!
//! In:  nothing
//! Out: as `scene::load_wanted_textures` leaves them
//!
//! Two lines in the game's log (a rule and "reload video plates - start");
//! `scene::clear_slot_textures`; `scene::load_wanted_textures`; two lines more
//! ("reload video plates - finis" and a rule).
//! Two callers (the front end's 3D set-up and a race's).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::game_ptr;
use crate::framework::{Regs, rva};
use crate::logfile::log_write::log_write;
use crate::scene::clear_slot_textures::clear_slot_textures;
use crate::scene::load_wanted_textures::load_wanted_textures_in_game;

/// The calls made, as places in `CALLEES`.
const ORDER: [usize; 6] = [0, 0, 1, 2, 0, 0];

// ---- Rust version ----

/// `step`: what stands for each call, by its place in `ORDER`; it is entered with the
/// registers the call before left.
pub unsafe fn reload_slot_textures(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for n in 0..ORDER.len() {
        step(n, r)?;
    }
    Ok(())
}

/// With the Rust versions of what it calls.
pub unsafe fn reload_slot_textures_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        reload_slot_textures(r, &mut |n, r| {
            match n {
                0 => log_write(game_ptr(rva(0x4DB858)), true, 1),
                1 => log_write(game_ptr(rva(0x4DB87D)), true, 1),
                2 => clear_slot_textures().apply(r),
                3 => load_wanted_textures_in_game(r)?,
                4 => log_write(game_ptr(rva(0x4DB899)), true, 1),
                _ => log_write(game_ptr(rva(0x4DB8B5)), true, 1),
            }
            Ok(())
        })
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    if let Err(e) = unsafe { reload_slot_textures_in_game(r) } {
        *r = entry;
        return Err(e);
    }
    Ok(())
}
