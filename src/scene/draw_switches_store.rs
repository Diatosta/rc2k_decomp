//! The draw switches, second part (IDA 0x621900 .. 0x621A84; phase 7, typed): the far clip switch, the group table word, the loading group mask, the SSE and 3DNow switches, and a marks source word. `#[repr(C)]` with the image layout over the race block bytes. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawSwitches {
    /// +0x0 (0x621900): `dword_621900`: the far clip / own car switch (`scene::clip_polygon_by_depth`, `race::draw_view_overlays`).
    pub far_on: Dword,
    /// +0x4 (0x621904): `dword_621904` (`scene::draw_polygon`).
    pub clears_switch: Dword,
    /// +0x8 (0x621908): not named.
    pub unknown_8: [Dword; 11],
    /// +0x34 (0x621934): `dword_621934`: the group table word (`scene::list_object_face`).
    pub group_table: Dword,
    /// +0x38 (0x621938): not named.
    pub unknown_38: [Dword; 71],
    /// +0x154 (0x621A54): `g_loadingGroupMask` (`scene::add_object_shape`).
    pub loading_group_mask: Dword,
    /// +0x158 (0x621A58): `g_useSse` (`scene::draw_object`, `scene::draw_polygon`, `math::normalise_vector`, `physics::ground`, `race::frame_draw`).
    pub use_sse: Dword,
    /// +0x15c (0x621A5C): `g_use3dNow` (`scene::draw_object`).
    pub use_3dnow: Dword,
    /// +0x160 (0x621A60): not named.
    pub unknown_160: [Dword; 8],
    /// +0x180 (0x621A80): `dword_621A80` (`effects::draw_frame`).
    pub marks_source: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static DRAW_SWITCHES: DrawSwitches =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x621900, the first byte of the piece.
pub const START: usize = 0x621900;
/// IDA 0x621A84, the end of the piece.
pub const END: usize = 0x621A84;

const _: () = assert!(END - START == core::mem::size_of::<DrawSwitches>());

/// Where IDA 0x621900 is now (the piece's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(&DRAW_SWITCHES))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&DRAW_SWITCHES, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(DrawSwitches, far_on), 0x621900 - START);
        assert_eq!(offset_of!(DrawSwitches, marks_source), 0x621A80 - START);
    }
}
