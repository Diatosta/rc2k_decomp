/// The texture number that means none.
pub const NO_TEXTURE: u32 = 0xFFFF_FC43;

/// No texture set (`d3d::set_texture` with `NO_TEXTURE`) unless the switch is 0: true
/// when it was set (the texture is then none).
pub fn clear_texture(
    part: &crate::scene::workers::PartScratch,
    scene: &crate::scene::owner::Scene,
) -> bool {
    if scene.objects.tables.get().d3d_drawing_on.get() == 0 {
        return false;
    }
    crate::d3d::set_texture::set_texture(part, NO_TEXTURE);
    true
}
