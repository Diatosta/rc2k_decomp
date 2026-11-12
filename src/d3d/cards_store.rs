//! The 3D cards' and the Direct3D device's state (`gfx::find_3d_cards`, `platform_3d`,
//! `d3d::set_render_state`, `d3d::set_texture`, the 3D switches of `var\ral.cfg`).
//! `#[repr(C)]` struct of plain cells (the game's thread alone reaches it: `D3d::cards`),
//! first values the image's (`tests`); the settings table names the switches by IDA address.

use crate::session::{PlainBytes, PlainDword};

/// The 3D cards, the device's render states and the 3D switches (IDA 0x5AB4B4 ..).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Cards3d {
    /// 0x5AB4B4: `g_secondaryCardGuid`: the secondary card's GUID.
    pub secondary_card_guid: PlainBytes<16>,
    /// 0x5AB678: `g_d3dDevice`: the Direct3D device (a pointer).
    pub d3d_device: PlainDword,
    /// 0x5AB6BC: `dword_5AB6BC`: each render state's last value (`d3d::set_render_state`); the
    /// drawing parts keep it in their own caches now (`d3d::render_cache`), this is the image's
    /// place for it.
    pub _kept_states: [PlainDword; 160],
    /// 0x5AB93C: `g_currentTexture`: the texture set last (`d3d::set_texture`); in
    /// `d3d::render_cache` likewise.
    pub _current_texture: PlainDword,
    /// 0x5AB94C: `g_no3dSearch`: not 0, no search for 3D cards (`forceno3dcard`).
    pub no_search: PlainDword,
    /// 0x5AB950: `dword_5AB950`: no 3D primary card (`ignoreprimary`).
    pub not_primary: PlainDword,
    /// 0x5AB954: `dword_5AB954`: `ignoresecondary`.
    pub ignore_secondary: PlainDword,
    /// 0x5AB958: `dword_5AB958`: `forcedx5`.
    pub force_dx5: PlainDword,
    /// 0x5AB95C: `dword_5AB95C`: `agpmode`.
    pub agp_mode: PlainDword,
    /// 0x5AB960: `dword_5AB960`: the 3D scene is begun.
    pub in_scene: PlainDword,
    /// 0x5AB964: `dword_5AB964`: the fade's start (`scene::draw_polygon`); now each view's (`scene::polygon_work::PolygonWork`); unused.
    pub fade_from: PlainDword,
    /// 0x5AB97C: `dword_5AB97C`: `force3dtrans`.
    pub force_trans: PlainDword,
    /// 0x5AB980: `dword_5AB980`: `force3dshadow`.
    pub force_shadow: PlainDword,
    /// 0x5AB984: `dword_5AB984`: `force3dalpha`.
    pub force_alpha: PlainDword,
    /// 0x5AB988: `dword_5AB988`: `force3dfogging`.
    pub force_fogging: PlainDword,
    /// 0x5AB98C: `dword_5AB98C` ..: the switches the graphics options read, set by `gfx::mark_3d_modes`.
    pub mode_switches: [PlainDword; 5],
    /// 0x5AB9A0: `dword_5AB9A0`: render state 0x1A's value.
    pub fog_state: PlainDword,
    /// 0x5AB9A4: `dword_5AB9A4`: the texture figure (`gfx::screen_mode_choices`).
    pub mips: PlainDword,
    /// 0x5AB9A8: `dword_5AB9A8`, `dword_5AB9AC`: the last two switches `gfx::mark_3d_modes` sets.
    pub mode_switches_end: [PlainDword; 2],
}

impl Cards3d {
    pub const fn new() -> Self {
        Cards3d {
            secondary_card_guid: PlainBytes::new([0; 16]),
            d3d_device: PlainDword::new(0x0),
            _kept_states: [const { PlainDword::new(0xfffffc43) }; 160],
            _current_texture: PlainDword::new(0xfffffc43),
            no_search: PlainDword::new(0x0),
            not_primary: PlainDword::new(0x0),
            ignore_secondary: PlainDword::new(0x0),
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

#[cfg(test)]
mod tests {

    #[test]
    fn cards3d_starts_as_the_games() {
        let _g = crate::test_support::test_lock();
        crate::test_support::first_values::assert_store_starts_as_the_games(
            "store:d3d.owner.D3D.cards",
        );
    }
}
