//! The overlay and countdown part of the race block: the stage-bar parts (distances, two colour tables), the start lights, the countdown sounds and the position labels with their kept copies. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct OverlayBlock {
    /// +0x0 (0x6D150C): `g_overlayPartMiles`: the overlay parts' distances (`race::init_overlay`, `hud::draw_stage_bar`).
    pub overlay_part_miles: [Float; 64],
    /// +0x100 (0x6D160C): `dword_6D160C`: the parts' colours/kinds, first view (`race::init_overlay`, `race::frame_draw` COLOUR_BUFFERS, `hud::draw_stage_bar`).
    pub overlay_part_kinds_a: [Dword; 64],
    /// +0x200 (0x6D170C): `dword_6D170C`: the same, second view (`race::init_overlay`, `hud::draw_stage_bars`).
    pub overlay_part_kinds_b: [Dword; 64],
    /// +0x300 (0x6D180C): `flt_6D180C`: the polygon w factor (`scene::draw_polygon`).
    pub w_factor: Float,
    /// +0x304 (0x6D1810): `g_startLightsLit`: the lights lit, 0 .. 5 (`race::frame_draw`, `race::stage_loading`).
    pub start_lights_lit: Dword,
    /// +0x308 (0x6D1814): `g_countdownStart`: the countdown's start (`race::frame_draw`, `drive::countdown_beep`).
    pub countdown_start: Dword,
    /// +0x30c (0x6D1818): `dword_6D1818`: the sound of each countdown second (`drive::countdown_beep`).
    pub countdown_sounds: [Dword; 6],
    /// +0x324 (0x6D1830): `dword_6D1830`: view 0 forget flag (`race::frame_views`, `race::frame_draw`).
    pub forget_flag_0: Dword,
    /// +0x328 (0x6D1834): `dword_6D1834`: view 1 forget flag (`race::frame_views`, `race::frame_draw`).
    pub forget_flag_1: Dword,
    /// +0x32c (0x6D1838): `dword_6D1838` + `a64`: a 7-byte text, the label behind (`race::frame_draw`, `race::name_plates`, `physics::rank_split_time`, `race::race_init`).
    pub label_behind: [Dword; 2],
    /// +0x334 (0x6D1840): not named.
    pub unknown_334: Dword,
    /// +0x338 (0x6D1844): `dword_6D1844` + `a32`: a 7-byte text, the label ahead (same modules).
    pub label_ahead: [Dword; 2],
    /// +0x340 (0x6D184C): not named.
    pub unknown_340: Dword,
    /// +0x344 (0x6D1850): `dword_6D1850` + `a64_0`: the kept copy of label_behind (`race::name_plates`, `race::race_init`).
    pub kept_behind: [Dword; 2],
    /// +0x34c (0x6D1858): not named.
    pub unknown_34c: Dword,
    /// +0x350 (0x6D185C): `dword_6D185C` + `a32_0`: the kept copy of label_ahead (same).
    pub kept_ahead: [Dword; 2],
    /// +0x358 (0x6D1864): not named.
    pub unknown_358: [Dword; 3],
}

/// IDA 0x6D150C, the first byte of the piece.
pub const START: usize = 0x6D150C;
/// IDA 0x6D1870, the end of the piece.
pub const END: usize = 0x6D1870;

const _: () = assert!(END - START == core::mem::size_of::<OverlayBlock>());

/// Where IDA 0x6D150C is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().frame.panel_and_overlay.overlay))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            offset_of!(OverlayBlock, overlay_part_miles),
            0x6D150C - START
        );
        assert_eq!(offset_of!(OverlayBlock, countdown_sounds), 0x6D1818 - START);
        assert_eq!(offset_of!(OverlayBlock, kept_ahead), 0x6D185C - START);
    }
}
