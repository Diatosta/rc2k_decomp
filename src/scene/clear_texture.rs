/// The texture number that means none.
pub const NO_TEXTURE: u32 = 0xFFFF_FC43;

/// No texture set (`d3d::set_texture` with `NO_TEXTURE`) unless the switch is 0: true
/// when it was set (the original then leaves `NO_TEXTURE` in eax).
pub fn clear_texture(scene: &crate::scene::owner::Scene) -> bool {
    if scene.objects.tables.has_3d.get() == 0 {
        return false;
    }
    crate::d3d::set_texture::set_texture(NO_TEXTURE);
    true
}
