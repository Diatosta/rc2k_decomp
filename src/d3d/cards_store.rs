//! The 3D cards' and the Direct3D device's state (`d3d::device`, the 3D switches of
//! `var\ral.cfg`): plain cells only the game's thread reaches (`D3d::cards`).

use std::cell::Cell;

/// The 3D cards, the device's render states and the 3D switches.
pub struct Cards3d {
    /// The `forcedx5` setting.
    pub force_dx5: Cell<u32>,
    /// The `agpmode` setting.
    pub agp_mode: Cell<u32>,
    /// The 3D scene is begun.
    pub in_scene: Cell<u32>,
    /// The `force3dtrans` setting.
    pub force_trans: Cell<u32>,
    /// The `force3dshadow` setting.
    pub force_shadow: Cell<u32>,
    /// The `force3dalpha` setting.
    pub force_alpha: Cell<u32>,
    /// The `force3dfogging` setting.
    pub force_fogging: Cell<u32>,
    /// The switches the graphics options read, set by `gfx::mark_3d_modes`.
    pub mode_switches: [Cell<u32>; 5],
    /// Render state 0x1A's value.
    pub fog_state: Cell<u32>,
    /// The texture figure (`gfx::screen_mode_choices`).
    pub mips: Cell<u32>,
    /// The last two switches `gfx::mark_3d_modes` sets.
    pub mode_switches_end: [Cell<u32>; 2],
}

impl Cards3d {
    pub const fn new() -> Self {
        Cards3d {
            force_dx5: Cell::new(0),
            agp_mode: Cell::new(0),
            in_scene: Cell::new(0),
            force_trans: Cell::new(0xffff_ffff),
            force_shadow: Cell::new(0xffff_ffff),
            force_alpha: Cell::new(0xffff_ffff),
            force_fogging: Cell::new(0xffff_ffff),
            mode_switches: [
                Cell::new(1),
                Cell::new(1),
                Cell::new(1),
                Cell::new(1),
                Cell::new(0),
            ],
            fog_state: Cell::new(0),
            mips: Cell::new(0),
            mode_switches_end: [const { Cell::new(0) }; 2],
        }
    }
}
