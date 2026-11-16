//! `scene::load_wanted_textures`: works out what every texture slot should hold and loads what is
//! missing: `scene::assign_kind_textures`, `scene::map_texture_places`,
//! `scene::load_slot_textures`.

use crate::errors::BadInput;
use crate::race::owner_frame::Frame;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

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
