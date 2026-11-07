//! `scene::load_slot_texture` (0x425ADE): gives a texture slot the texture it wants,
//! if it holds another.
//!
//! Takes: the slot
//!
//! Nothing if the slot wants none (`dword_5DA968[slot]` is -1) or holds what it wants
//! (`dword_5DB968[slot]`). Else `scene::load_texture` (0x44E778, which is only `sub_41A8F9` of
//! the Direct3D layer: not ported) with eax = the texture and ebx = the slot, `dword_6415F8` +
//! 1, and the slot holds it. One caller, `scene::load_slot_textures`. Refuses a slot past the
//! table.

use crate::errors::BadInput;

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture`, given the texture and the slot.
// Converted: `rc2k_core::scene::textures::slot_load`
pub fn load_slot_texture(
    frame: &crate::race::owner_frame::Frame,
    slot: u32,
    load: &mut dyn FnMut(u32, u32),
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let textures = &scene.textures;
    let Some(wanted) = rc2k_core::scene::textures::slot_load(
        slot,
        || textures.places.slot_wanted[slot as usize].get(),
        || textures.places.slot_loaded[slot as usize].get(),
    )?
    else {
        return Ok(());
    };
    load(wanted, slot);
    frame
        .frame_work
        .kind_textures_done
        .set_bits(frame.frame_work.kind_textures_done.bits().wrapping_add(1));
    textures.places.slot_loaded[slot as usize].set(wanted);
    Ok(())
}

/// With the game's loading (`scene::load_texture`, the platform's).
pub fn load_slot_texture_in_game(
    frame: &crate::race::owner_frame::Frame,
    slot: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    load_slot_texture(frame, slot, &mut crate::platform_3d::load_texture, scene)
}
