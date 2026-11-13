//! The 3D reports switch (`reports.on`), out of RAL.EXE's image: IDA 0x4F52AC .. 0x4F52B0.
//! Direct3D's and DirectDraw's failures are fatal while it is 1; the film player, the mode trial
//! and the texture allocation clear it around calls that may fail.

use crate::session::PlainDword;

/// The 3D reports switch.
#[derive(rc2k_derive::Cells)]
pub struct Reports {
    /// `dword_4F52AC`: 1 while Direct3D's errors are reported (fatal).
    pub on: PlainDword,
}

/// The switch; the first value is the image's.
impl Reports {
    pub const fn new() -> Self {
        Reports {
            on: PlainDword::new(1),
        }
    }
}
