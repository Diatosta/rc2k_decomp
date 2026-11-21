//! The device's texture set to the batch's (`scene::set_texture`). Refuses a texture past the
//! kind table's 0x400 dwords (the HUD asks for 0xC0 .. 0xDC), before anything.

use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::scene::owner::Scene;

pub fn set_batch_texture(
    part: &crate::scene::scratch_store::PartScratch,
    frame: &Frame,
    scene: &Scene,
) -> Result<(), Refused> {
    let n = rc2k_core::hud::batch::texture_slot(
        frame.panel_and_overlay.get().panel.shown_texture.get(),
    )?;
    crate::scene::set_texture::set_texture(
        part,
        scene,
        scene.textures.kinds.borrow().kind_first_slots[n],
    );
    Ok(())
}
