//! The 3D cards' and the Direct3D device's state (`gfx::find_3d_cards`, `platform_3d`,
//! `d3d::set_render_state`, `d3d::set_texture`, the 3D switches of `var\ral.cfg`).
//! `#[repr(C)]` struct of atomic cells with
//! the image's layout, first values the image's (`tests`); `moved::MOVED` points the image's
//! addresses at it (`home`): the settings table names the switches by IDA address.

use crate::session::{Bytes, Dword, with_runs};

/// The 3D cards, the device's render states and the 3D switches (IDA 0x5AB4B4 .. 0x5ABA18).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct Cards3d {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<4>,
    /// 0x5AB4B4: `g_secondaryCardGuid`: the secondary card's GUID.
    pub secondary_card_guid: Bytes<16>,
    /// 0x5AB4C4: Not named: 436 bytes (a gap, or data nothing reads here).
    pub _gap_5ab4c4: Bytes<436>,
    /// 0x5AB678: `g_d3dDevice`: the Direct3D device (a pointer).
    pub d3d_device: Dword,
    /// 0x5AB67C: Not named: 64 bytes (a gap, or data nothing reads here).
    pub _gap_5ab67c: Bytes<64>,
    /// 0x5AB6BC: `dword_5AB6BC`: each render state's last value (`d3d::set_render_state`).
    pub kept_states: [Dword; 160],
    /// 0x5AB93C: `g_currentTexture`: the texture set last (`d3d::set_texture`).
    pub current_texture: Dword,
    /// 0x5AB940: Not named: 12 bytes (a gap, or data nothing reads here).
    pub _gap_5ab940: Bytes<12>,
    /// 0x5AB94C: `g_no3dSearch`: not 0, no search for 3D cards (`forceno3dcard`).
    pub no_search: Dword,
    /// 0x5AB950: `dword_5AB950`: no 3D primary card (`ignoreprimary`).
    pub not_primary: Dword,
    /// 0x5AB954: `dword_5AB954`: `ignoresecondary`.
    pub ignore_secondary: Dword,
    /// 0x5AB958: `dword_5AB958`: `forcedx5`.
    pub force_dx5: Dword,
    /// 0x5AB95C: `dword_5AB95C`: `agpmode`.
    pub agp_mode: Dword,
    /// 0x5AB960: `dword_5AB960`: the 3D scene is begun.
    pub in_scene: Dword,
    /// 0x5AB964: `dword_5AB964`: the fade's start (`scene::draw_polygon`); now each view's (`scene::polygon_work::PolygonWork`); unused.
    pub fade_from: Dword,
    /// 0x5AB968: Not named: 20 bytes (a gap, or data nothing reads here).
    pub _gap_5ab968: Bytes<20>,
    /// 0x5AB97C: `dword_5AB97C`: `force3dtrans`.
    pub force_trans: Dword,
    /// 0x5AB980: `dword_5AB980`: `force3dshadow`.
    pub force_shadow: Dword,
    /// 0x5AB984: `dword_5AB984`: `force3dalpha`.
    pub force_alpha: Dword,
    /// 0x5AB988: `dword_5AB988`: `force3dfogging`.
    pub force_fogging: Dword,
    /// 0x5AB98C: `dword_5AB98C` ..: the switches the graphics options read, set by `gfx::mark_3d_modes`.
    pub mode_switches: [Dword; 5],
    /// 0x5AB9A0: `dword_5AB9A0`: render state 0x1A's value.
    pub fog_state: Dword,
    /// 0x5AB9A4: `dword_5AB9A4`: the texture figure (`gfx::screen_mode_choices`).
    pub mips: Dword,
    /// 0x5AB9A8: `dword_5AB9A8`, `dword_5AB9AC`: the last two switches `gfx::mark_3d_modes` sets.
    pub mode_switches_end: [Dword; 2],
    /// 0x5AB9B0: Not named: 104 bytes (a gap, or data nothing reads here).
    pub _gap_5ab9b0: Bytes<104>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: Bytes<8>,
}

impl Cards3d {
    pub const fn new() -> Self {
        Cards3d {
            _lead: Bytes::new([0; 4]),
            secondary_card_guid: Bytes::new([0; 16]),
            _gap_5ab4c4: Bytes::new(with_runs::<436>(&[(428, b"\x01")])),
            d3d_device: Dword::new(0x0),
            _gap_5ab67c: Bytes::new([0; 64]),
            kept_states: [const { Dword::new(0xfffffc43) }; 160],
            current_texture: Dword::new(0xfffffc43),
            _gap_5ab940: Bytes::new(with_runs::<12>(&[(
                0,
                b"C\xfc\xff\xffC\xfc\xff\xffC\xfc\xff\xff",
            )])),
            no_search: Dword::new(0x0),
            not_primary: Dword::new(0x0),
            ignore_secondary: Dword::new(0x0),
            force_dx5: Dword::new(0x0),
            agp_mode: Dword::new(0x0),
            in_scene: Dword::new(0x0),
            fade_from: Dword::new(0x0),
            _gap_5ab968: Bytes::new([0; 20]),
            force_trans: Dword::new(0xffffffff),
            force_shadow: Dword::new(0xffffffff),
            force_alpha: Dword::new(0xffffffff),
            force_fogging: Dword::new(0xffffffff),
            mode_switches: [
                Dword::new(0x1),
                Dword::new(0x1),
                Dword::new(0x1),
                Dword::new(0x1),
                Dword::new(0x0),
            ],
            fog_state: Dword::new(0x0),
            mips: Dword::new(0x0),
            mode_switches_end: [const { Dword::new(0x0) }; 2],
            _gap_5ab9b0: Bytes::new(with_runs::<104>(&[(4, b"\x01")])),
            _tail: Bytes::new([0; 8]),
        }
    }
}

impl Cards3d {
    pub const START: usize = 0x5AB4B4;
    pub const LEN: usize = 0x564;
}

#[cfg(test)]
mod tests {

    #[test]
    fn cards3d_starts_as_the_games() {
        let _g = crate::store::test_lock();
        crate::test_support::first_values::assert_store_starts_as_the_games(
            "store:d3d.owner.D3D.cards",
        );
    }
}
