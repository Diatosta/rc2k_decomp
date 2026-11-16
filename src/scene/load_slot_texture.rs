//! `scene::load_slot_texture`: gives a texture slot the texture it wants, if it holds another.
//!
//! Nothing if the slot wants none (its wanted entry is -1) or holds what it wants (its loaded
//! entry). Else `scene::load_texture` (which is only `d3d::copy_to_device_texture` of the Direct3D
//! layer) with the texture and the slot, the loads counter + 1, and the slot holds it. Refuses a
//! slot past the table.

use crate::errors::BadInput;

/// `load`: what stands for `scene::load_texture`, given the texture and the slot.
// Logic: `rc2k_core::scene::textures::slot_load`
pub fn load_slot_texture(
    frame: &crate::race::owner_frame::Frame,
    slot: u32,
    load: &mut dyn FnMut(u32, u32),
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let textures = &scene.textures;
    let Some(wanted) = rc2k_core::scene::textures::slot_load(
        slot,
        || textures.places.get().slot_wanted[slot as usize].get(),
        || textures.places.get().slot_loaded[slot as usize].get(),
    )?
    else {
        return Ok(());
    };
    load(wanted, slot);
    frame.frame_work.get().kind_textures_done.set(
        frame
            .frame_work
            .get()
            .kind_textures_done
            .get()
            .wrapping_add(1),
    );
    textures.places.get().slot_loaded[slot as usize].set(wanted);
    Ok(())
}

/// With the game's loading (`scene::load_texture`, the platform's).
pub fn load_slot_texture_in_game(
    frame: &crate::race::owner_frame::Frame,
    slot: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    load_slot_texture(
        frame,
        slot,
        &mut crate::platform::direct3d::load_texture,
        scene,
    )
}
