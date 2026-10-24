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

use crate::errors::BadInput;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- Rust version ----

/// Its three calls in order. What they leave in the registers no caller reads (register
/// liveness, 2026-10-23: `scene::reload_slot_textures`, whose caller reads none).
pub unsafe fn load_wanted_textures() -> Result<(), BadInput> {
    unsafe {
        assign_kind_textures()?;
        map_texture_places();
        load_slot_textures_in_game()?;
        Ok(())
    }
}
