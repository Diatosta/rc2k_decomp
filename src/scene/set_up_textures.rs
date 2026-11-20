//! `scene::set_up_textures`: the texture memory for everything that is needed, and one texture
//! more for the game's own use.
//!
//! `scene::build_texture_catalogue`; the mirror picture = 0; then one system texture of size code 3
//! and pixel kind 4 without mip levels from `gfx::allocate_textures`, whose number goes to the
//! mirror picture. If that one is refused: fatal error 0x80600943.

use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;
use crate::scene::build_texture_catalogue::build_texture_catalogue_in_game;

// ---- game data ----

/// The catalogue built, then one system texture of the scene's own (sizes 3, pixel kind 4, no
/// mips; `gfx::allocate_textures`) kept in `SceneDraw.mirror_picture`. The allocation cannot
/// come back refused (a failure raises its own fatal error), so fatal error 0x80600943 ("cant
/// alloc sys video texture") is never reached.
pub fn set_up_textures(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    d3d: &crate::d3d::owner::D3d,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    crate::logfile::load_times::mark("build_texture_catalogue");
    build_texture_catalogue_in_game(shared, scene, gfx, d3d)?;
    drawing.scene.get().mirror_picture.set(0u32);
    crate::logfile::load_times::mark("allocate_textures_for");
    let (_, system) = crate::d3d::device::allocate_textures_for(0, 1, 3, 4, 0, gfx, d3d);
    drawing.scene.get().mirror_picture.set(system);
    Ok(())
}
