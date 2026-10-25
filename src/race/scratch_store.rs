//! The scratch point, work vector and kept frame (IDA 0x5C5F48 .. 0x5C5FB8; phase 7, typed): the tangent scratch, the scratch point's three floats, the work vector and its squares (`math::normalise_vector`), and the locked surface's kept pitch and pixels. `#[repr(C)]` with the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
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
    pub unknown_38: [Dword; 12],
    /// +0x68 (0x5C5FB0): `dword_5C5FB0`: the locked surface's pitch (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pitch: Dword,
    /// +0x6c (0x5C5FB4): `dword_5C5FB4`: the locked surface's pixels (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pixels: Dword,
}

/// The piece: the race block's bytes at IDA 0x5C5F48 seen as `FrameScratch`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = FrameScratch;
    fn deref(&self) -> &FrameScratch {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const FrameScratch) }
    }
}

pub static FRAME_SCRATCH: InRaceBlock = InRaceBlock;

/// IDA 0x5C5F48, the first byte of the piece.
pub const START: usize = 0x5C5F48;
/// IDA 0x5C5FB8, the end of the piece.
pub const END: usize = 0x5C5FB8;

const _: () = assert!(END - START == core::mem::size_of::<FrameScratch>());

/// Where IDA 0x5C5F48 is now (in the race block).
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
        assert_eq!(offset_of!(FrameScratch, tangent_scratch), 0x5C5F48 - START);
        assert_eq!(offset_of!(FrameScratch, frame_pitch), 0x5C5FB0 - START);
    }
}
