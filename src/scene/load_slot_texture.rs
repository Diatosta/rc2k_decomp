//! `scene::load_slot_texture`: gives a texture slot the texture it wants, if it holds another.
//!
//! Nothing if the slot wants none (its wanted entry is -1) or holds what it wants (its loaded
//! entry). Else `scene::load_texture` (which is only `d3d::copy_to_device_texture` of the Direct3D
//! layer) with the texture and the slot, the loads counter + 1, and the slot holds it. Refuses a
//! slot past the table.

use crate::errors::BadInput;

// Logic: `rc2k_core::scene::textures::slot_load`
pub fn load_slot_texture(
    frame: &crate::race::owner_frame::Frame,
    slot: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let Some(wanted) = rc2k_core::scene::textures::slot_load(
        slot,
        || scene.texture_places().slot_wanted[slot as usize],
        || scene.texture_places().slot_loaded[slot as usize],
    )?
    else {
        return Ok(());
    };
    crate::d3d::device::load_texture(wanted, slot);
    {
        let mut work = frame.frame_work.borrow_mut();
        work.kind_textures_done = work.kind_textures_done.wrapping_add(1);
    }
    scene.texture_places_mut().slot_loaded[slot as usize] = wanted;
    Ok(())
}
