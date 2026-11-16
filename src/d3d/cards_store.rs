//! The 3D cards' and the Direct3D device's state (`platform::direct3d`,
//! `d3d::set_render_state`, `d3d::set_texture`, the 3D switches of `var\ral.cfg`). A `#[repr(C)]`
//! struct of plain cells (the game's thread alone reaches it: `D3d::cards`), first values the
//! image's (`tests`).

use crate::memory::session::PlainDword;

/// The 3D cards, the device's render states and the 3D switches.
#[derive(rc2k_derive::Cells)]
pub struct Cards3d {
    /// The Direct3D device (a pointer).
    pub d3d_device: PlainDword,
    /// Each render state's last value (`d3d::set_render_state`); the drawing parts keep it in
    /// their own caches now (`d3d::render_cache`), this is the image's place for it.
    pub _kept_states: [PlainDword; 160],
    /// The texture set last (`d3d::set_texture`); in `d3d::render_cache` likewise.
    pub _current_texture: PlainDword,
    /// The `forcedx5` setting.
    pub force_dx5: PlainDword,
    /// The `agpmode` setting.
    pub agp_mode: PlainDword,
    /// The 3D scene is begun.
    pub in_scene: PlainDword,
    /// The fade's start (`scene::draw_polygon`). Unused: each view has its own in
    /// `scene::polygon_work::PolygonWork`.
    pub fade_from: PlainDword,
    /// The `force3dtrans` setting.
    pub force_trans: PlainDword,
    /// The `force3dshadow` setting.
    pub force_shadow: PlainDword,
    /// The `force3dalpha` setting.
    pub force_alpha: PlainDword,
    /// The `force3dfogging` setting.
    pub force_fogging: PlainDword,
    /// The switches the graphics options read, set by `gfx::mark_3d_modes`.
    pub mode_switches: [PlainDword; 5],
    /// Render state 0x1A's value.
    pub fog_state: PlainDword,
    /// The texture figure (`gfx::screen_mode_choices`).
    pub mips: PlainDword,
    /// The last two switches `gfx::mark_3d_modes` sets.
    pub mode_switches_end: [PlainDword; 2],
}

impl Cards3d {
    pub const fn new() -> Self {
        Cards3d {
            d3d_device: PlainDword::new(0x0),
            _kept_states: [const { PlainDword::new(0xfffffc43) }; 160],
            _current_texture: PlainDword::new(0xfffffc43),
            force_dx5: PlainDword::new(0x0),
            agp_mode: PlainDword::new(0x0),
            in_scene: PlainDword::new(0x0),
            fade_from: PlainDword::new(0x0),
            force_trans: PlainDword::new(0xffffffff),
            force_shadow: PlainDword::new(0xffffffff),
            force_alpha: PlainDword::new(0xffffffff),
            force_fogging: PlainDword::new(0xffffffff),
            mode_switches: [
                PlainDword::new(0x1),
                PlainDword::new(0x1),
                PlainDword::new(0x1),
                PlainDword::new(0x1),
                PlainDword::new(0x0),
            ],
            fog_state: PlainDword::new(0x0),
            mips: PlainDword::new(0x0),
            mode_switches_end: [const { PlainDword::new(0x0) }; 2],
        }
    }
}
