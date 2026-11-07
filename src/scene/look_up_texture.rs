//! `scene::look_up_texture` (0x44E668): `g_polygonTexture` for the texture word
//! `dword_6240C4` (stream A, part R3): -1 none; with the top bit a place
//! (`dword_5D3AE8[]`, below 0xFA0); else a kind's first slot (`g_kindFirstSlots[]`, below
//! 0x400); a slot not below `g_textureSlotCount` (or none loaded) none (0xFFFFFC43).
//!
//! Callers: `scene::choose_texture` (0x44CCB8), `race::flush_texture_batch`.

/// The Rust version: what it leaves in eax.
pub fn look_up_texture(scene: &crate::scene::owner::Scene) -> u32 {
    // Converted: `rc2k_core::scene::textures::look_up`.
    let found = rc2k_core::scene::textures::look_up(
        scene.drawing.scene.texture_page.bits(),
        |p| scene.textures.places.place_slot[p].get(),
        |k| scene.textures.kinds.kind_first_slots[k].get(),
        || scene.textures.places.totals[2].get(),
    );
    scene.drawing.scene.polygon_texture.set_bits(found.texture);
    found.eax
}
