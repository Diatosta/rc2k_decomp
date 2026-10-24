use crate::framework::{game_read, rva};

const SWITCH_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
/// The texture number that means none.
pub const NO_TEXTURE: u32 = 0xFFFF_FC43;

/// No texture set (`d3d::set_texture` with `NO_TEXTURE`) unless the switch is 0: true
/// when it was set (the original then leaves `NO_TEXTURE` in eax).
pub fn clear_texture() -> bool {
    if unsafe { game_read::<u32>(SWITCH_RVA) } == 0 {
        return false;
    }
    crate::d3d::set_texture::set_texture(NO_TEXTURE);
    true
}
