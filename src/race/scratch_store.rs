//! The scratch point, work vector and kept frame (IDA 0x5C5F48 .. 0x5C5FB8; phase 7, typed): the tangent scratch, the scratch point's three floats, the work vector and its squares (`math::normalise_vector`), and the locked surface's kept pitch and pixels. `#[repr(C)]` with the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FrameScratch {
    /// +0x0 (0x5C5F48): `dword_5C5F48`: the tangent's doubled number (`scene::fill_engine_tables`).
    pub tangent_scratch: Dword,
    /// +0x4 (0x5C5F4C): `g_scratchZ`: the scratch point's z (`math::turn_scratch_xz`).
    pub scratch_z: Float,
    /// +0x8 (0x5C5F50): `g_scratchX`: the scratch point's x (`math::turn_scratch_xz`).
    pub scratch_x: Float,
    /// +0xc (0x5C5F54): `g_scratchY`: the scratch point's y (`math::turn_point_xz`).
    pub scratch_y: Float,
    /// +0x10 (0x5C5F58): not named.
    pub unknown_10: [Dword; 2],
    /// +0x18 (0x5C5F60): `g_workVector`: three floats and a fourth (`math::normalise_vector`, `physics::wall_contacts`, `race::norms`); taken in place.
    pub work_vector: [Dword; 4],
    /// +0x28 (0x5C5F70): `xmmword_5C5F70`: the four floats squared (`math::normalise_vector`).
    pub work_squares: [Dword; 4],
    /// +0x38 (0x5C5F80): not named.
    pub unknown_38: [Dword; 6],
    /// +0x50 (0x5C5F98): `g_debugCursorX`: where the debug font's last print ended
    /// (`debug::draw::print`, a print at (0, 0) goes on from here).
    pub debug_cursor_x: Dword,
    /// +0x54 (0x5C5F9C): `g_debugCursorY`: its row.
    pub debug_cursor_y: Dword,
    /// +0x58 (0x5C5FA0): not named.
    pub unknown_58: [Dword; 4],
    /// +0x68 (0x5C5FB0): `dword_5C5FB0`: the locked surface's pitch (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pitch: Dword,
    /// +0x6c (0x5C5FB4): `dword_5C5FB4`: the locked surface's pixels (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pixels: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static FRAME_SCRATCH: FrameScratch =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x5C5F48, the first byte of the piece.
pub const START: usize = 0x5C5F48;
/// IDA 0x5C5FB8, the end of the piece.
pub const END: usize = 0x5C5FB8;

const _: () = assert!(END - START == core::mem::size_of::<FrameScratch>());

/// Where IDA 0x5C5F48 is now (its own static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&FRAME_SCRATCH))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&FRAME_SCRATCH, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(FrameScratch, tangent_scratch), 0x5C5F48 - START);
        assert_eq!(offset_of!(FrameScratch, frame_pitch), 0x5C5FB0 - START);
    }
}
