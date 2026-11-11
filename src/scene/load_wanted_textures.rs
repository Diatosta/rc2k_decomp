//! `scene::load_wanted_textures` (0x4256C8): works out what every texture slot should hold and loads what is missing.
//!
//! `scene::assign_kind_textures`, `scene::map_texture_places`, `scene::load_slot_textures`. One
//! caller, `scene::reload_slot_textures`.

use crate::errors::BadInput;
use crate::race::owner_frame::Frame;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- Rust version ----

/// Its three calls in order.
pub fn load_wanted_textures(
    frame: &Frame,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    assign_kind_textures(scene)?;
    map_texture_places(scene);
    load_slot_textures_in_game(frame, scene)?;
    Ok(())
}
