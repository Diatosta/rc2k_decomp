//! `effects::swap_mask`: when the settings' +0x2704 differs from the last seen (then it is kept),
//! the effects' mask texture is refilled from mask block A (or mask block B when it is set; the
//! choice is kept): the mask texture slot locked (`d3d::lock_texture`, the platform's), rows 0 ..
//! 0x7D of 0x100 pixels (the locked surface's width each) put at y + 0x80 by
//! `gfx::put_surface_pixel`, unlocked (`d3d::unlock_texture`), and the effects' texture slot
//! forgotten (`scene::forget_slot_texture`).//! Works on the effects settings.
//!
//! Refuses null settings (before anything) and a locked texture (the surface after the lock) wider
//! than the mask's 0x100 (unlocked again, what was written put back).

use crate::effects::mask_store::{Mask, Masks};
use crate::errors::{BadInput, Refused};

/// The lock and unlock (`d3d::lock_texture`,
/// `unlock_texture`), `gfx::put_surface_pixel`, `scene::forget_slot_texture`.
// Converted: `rc2k_core::effects::masks::{swapped, check_width, texture_pixels}`
pub fn swap_mask(
    masks: &Masks,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use crate::effects::load_mask_pictures::{lock, put_pixel};
    use rc2k_core::effects::masks;
    let s: u32 = effects.setup.settings.is_set.get();
    if s == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let marks = effects.settings()?;
    let switch = marks.sound_objects_shown.get();
    let Some(second) = masks::swapped(switch, masks.blocks.seen.get()) else {
        return Ok(());
    };
    let kept: [u32; 2] = [masks.blocks.seen.get(), masks.blocks.chosen.get()];
    masks.blocks.seen.set(switch);
    let chosen = if second { Mask::Bordered } else { Mask::Plain };
    masks.blocks.chosen.set(chosen as u32);
    lock(
        gfx,
        scene.textures.kinds.get().kind_first_textures[202].get(),
        true,
    )?;
    // The width is the locked texture's, not the screen's: checked here (before the lock it
    // refused every frame). A row wider than the mask's would read past its block: unlocked,
    // put back and refused.
    let width: u32 = gfx.drawing.draw_surface.width.get();
    if let Err(e) = masks::check_width(width) {
        lock(
            gfx,
            scene.textures.kinds.get().kind_first_textures[202].get(),
            false,
        )?;
        masks.blocks.seen.set(kept[0]);
        masks.blocks.chosen.set(kept[1]);
        return Err(e.into());
    }
    let pixels = chosen.pixels(masks).with(|p| {
        masks::texture_pixels(width)
            .map(|(row, col, at)| Some((row, col, *p.get(at as usize)?)))
            .collect::<Option<Vec<_>>>()
    })?;
    let Some(pixels) = pixels else {
        lock(
            gfx,
            scene.textures.kinds.get().kind_first_textures[202].get(),
            false,
        )?;
        masks.blocks.seen.set(kept[0]);
        masks.blocks.chosen.set(kept[1]);
        return Err(BadInput("a mask pixel past its block").into());
    };
    for (row, col, pixel) in pixels {
        put_pixel(pixel, col, row + masks::TEXTURE_ROW_AT, gfx, system)?;
    }
    lock(
        gfx,
        scene.textures.kinds.get().kind_first_textures[202].get(),
        false,
    )?;
    crate::scene::forget_slot_texture::forget_slot_texture(
        scene.textures.kinds.get().kind_first_slots[202].get(),
        scene,
    )?;
    Ok(())
}
