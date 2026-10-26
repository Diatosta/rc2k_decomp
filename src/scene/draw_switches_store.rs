//! The draw switches, second part (IDA 0x621900 .. 0x621A84; phase 7, typed): the far clip switch, the group table word, the loading group mask, the SSE and 3DNow switches, and a marks source word. `#[repr(C)]` with the image layout over the race block bytes. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
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
    /// +0x180 (0x621A80): `dword_621A80` (`marks::draw_frame`).
    pub marks_source: Dword,
}

/// The piece: the race block's bytes at IDA 0x621900 seen as `DrawSwitches`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = DrawSwitches;
    fn deref(&self) -> &DrawSwitches {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const DrawSwitches) }
    }
}

pub static DRAW_SWITCHES: InRaceBlock = InRaceBlock;

/// IDA 0x621900, the first byte of the piece.
pub const START: usize = 0x621900;
/// IDA 0x621A84, the end of the piece.
pub const END: usize = 0x621A84;

const _: () = assert!(END - START == core::mem::size_of::<DrawSwitches>());

/// Where IDA 0x621900 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
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
        assert_eq!(offset_of!(DrawSwitches, far_on), 0x621900 - START);
        assert_eq!(offset_of!(DrawSwitches, marks_source), 0x621A80 - START);
    }
}
