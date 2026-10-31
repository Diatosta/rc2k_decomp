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
pub struct Surfaces {
    /// `g_frontSurface` (`dword_4F32B4`): the front surface, 0 with none.
    pub front: Dword,
    /// `dword_4F32B8`: the back surface.
    pub back: Dword,
}

/// The handles; the first values are the image's.
pub static SURFACES: Surfaces = Surfaces {
    front: Dword::new(0),
    back: Dword::new(0),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x4F32B4).
pub fn home() -> usize {
    crate::pointers::game_address(&raw const SURFACES) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&SURFACES, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), SURFACES.front.addr() as usize);
        assert_eq!(
            crate::moved::place(START + 4),
            SURFACES.back.addr() as usize
        );
    }
}
