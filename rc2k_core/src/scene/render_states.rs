//! The device's render states the scene and the HUD set as groups (converted 2026-10-21
//! from `scene::glow_render_states` 0x44E44D, `scene::set_filters` 0x44E8FB,
//! `scene::set_fog_colour` 0x44E76B and `hud::overlay_states` 0x45A943). The numbers are
//! Direct3D's `D3DRENDERSTATETYPE` (the names a reading of that numbering); each group is
//! set in the order given, one state at a time.

/// A render state and the value it is set to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setting {
    pub state: u32,
    pub value: u32,
}

const fn set(state: u32, value: u32) -> Setting {
    Setting { state, value }
}

pub const Z_ENABLE: u32 = 7;
pub const Z_WRITE_ENABLE: u32 = 0xE;
pub const TEXTURE_MAG: u32 = 0x11;
pub const TEXTURE_MIN: u32 = 0x12;
pub const SRC_BLEND: u32 = 0x13;
pub const DEST_BLEND: u32 = 0x14;
pub const TEXTURE_MAP_BLEND: u32 = 0x15;
pub const Z_FUNC: u32 = 0x17;
pub const ALPHA_BLEND_ENABLE: u32 = 0x1B;
pub const FOG_ENABLE: u32 = 0x1C;
pub const FOG_COLOUR: u32 = 0x22;
pub const ANISOTROPY: u32 = 0x31;

/// `glow_render_states`: what is drawn glowing (the sky's sun, the views' backdrop): the
/// depth test and depth writing on, the comparison "always" (8).
pub const GLOW: [Setting; 3] = [set(Z_ENABLE, 1), set(Z_WRITE_ENABLE, 1), set(Z_FUNC, 8)];

/// `hud::overlay_states`: the 2D overlay's.
pub const OVERLAY: [Setting; 5] = [
    set(TEXTURE_MAP_BLEND, 4),
    set(ALPHA_BLEND_ENABLE, 1),
    set(SRC_BLEND, 5),
    set(DEST_BLEND, 6),
    set(FOG_ENABLE, 0),
];

/// `set_filters`: the textures' filters by the setting (`dword_623898`): made larger 2;
/// made smaller 2 with the setting 0, else 6; then 0x31 1.
pub fn filters(setting: u32) -> [Setting; 3] {
    let smaller = if setting == 0 { 2 } else { 6 };
    [
        set(TEXTURE_MAG, 2),
        set(TEXTURE_MIN, smaller),
        set(ANISOTROPY, 1),
    ]
}

/// `set_fog_colour`'s state: the colour with its top byte set.
pub fn fog_colour(colour: u32) -> Setting {
    set(FOG_COLOUR, colour | 0xFF00_0000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups() {
        assert_eq!(filters(0)[1].value, 2);
        assert_eq!(filters(7)[1].value, 6);
        assert_eq!(
            fog_colour(0x12_3456),
            Setting {
                state: 0x22,
                value: 0xFF12_3456
            }
        );
        assert_eq!(GLOW[2], set(0x17, 8));
        assert_eq!(OVERLAY[4], set(0x1C, 0));
    }
}
