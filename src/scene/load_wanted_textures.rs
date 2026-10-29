//! `scene::load_wanted_textures` (0x4256C8): works out what every texture slot should hold and loads what is missing.
//!
//! In:  nothing
//! Out: as `scene::load_slot_textures` leaves them
//!
//! `scene::assign_kind_textures`, `scene::map_texture_places`, `scene::load_slot_textures`. One
//! caller, `scene::reload_slot_textures`.

use crate::errors::BadInput;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- Rust version ----

/// Its three calls in order. What they leave in the registers no caller reads (register
/// liveness, 2026-10-23: `scene::reload_slot_textures`, whose caller reads none).
pub fn load_wanted_textures() -> Result<(), BadInput> {
    assign_kind_textures()?;
    map_texture_places();
    load_slot_textures_in_game()?;
    Ok(())
}
