//! The DirectDraw front and back surface handles, out of RAL.EXE's image (phase 5; typed in
//! phase 7, `notes/conversion.md`): IDA 0x4F32B4 .. 0x4F32BC. The platform layer sets both
//! to 1 ("there are surfaces"); the clear functions test the front one.

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F32B4;
/// The block's length.
pub const LEN: usize = 8;

/// The surface handles.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Surfaces {
    /// `g_frontSurface` (`dword_4F32B4`): the front surface, 0 with none.
    pub front: Dword,
    /// `dword_4F32B8`: the back surface.
    pub back: Dword,
}

impl Surfaces {
    /// The handles; the first values are the image's.
    pub const fn new() -> Self {
        Surfaces {
            front: Dword::new(0),
            back: Dword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::owner::gfx;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&gfx().drawing.surfaces, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        let drawing = &gfx().drawing;
        assert_eq!(
            crate::moved::place(START),
            drawing.surfaces.front.addr() as usize
        );
        assert_eq!(
            crate::moved::place(START + 4),
            drawing.surfaces.back.addr() as usize
        );
    }
}
