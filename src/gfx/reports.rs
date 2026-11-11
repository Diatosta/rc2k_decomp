//! The 3D reports switch (`reports.on`), out of RAL.EXE's image: IDA 0x4F52AC .. 0x4F52B0.
//! Direct3D's and DirectDraw's failures are fatal while it is 1; the film player, the mode trial
//! and the texture allocation clear it around calls that may fail.

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F52AC;
/// The block's length.
pub const LEN: usize = 4;

/// The 3D reports switch.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Reports {
    /// `dword_4F52AC`: 1 while Direct3D's errors are reported (fatal).
    pub on: Dword,
}

/// The switch; the first value is the image's.
impl Reports {
    pub const fn new() -> Self {
        Reports { on: Dword::new(1) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&crate::gfx::owner::GFX.reports, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::GFX.reports.on.addr() as usize
        );
    }
}
