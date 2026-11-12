//! `hud::set_batch_texture` (0x45A855): the device's texture set to the batch's
//! (`g_kindFirstSlots`[`shown_texture`], `scene::set_texture`).
//!
//! Caller: `hud::add_polygon`. Refuses a texture past `g_kindFirstSlots` (0x400 dwords; the HUD
//! asks for 0xC0 .. 0xDC), before anything.

use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::scene::owner::Scene;

// Converted: `rc2k_core::hud::batch::texture_slot`
pub fn set_batch_texture(
    part: &crate::workers::PartScratch,
    frame: &Frame,
    scene: &Scene,
) -> Result<(), Refused> {
    let n = rc2k_core::hud::batch::texture_slot(frame.panel_and_overlay.panel.shown_texture.get())?;
    crate::scene::set_texture::set_texture(
        part,
        scene,
        scene.textures.kinds.get().kind_first_slots[n].get(),
    );
    Ok(())
}
