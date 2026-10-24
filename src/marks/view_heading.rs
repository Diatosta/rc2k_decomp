//! `marks::view_heading` (0x4C4CEC): the view's heading: the angle between its forward
//! direction's level part (`flt_5EA3CC` +8, 0, +0x28, into `flt_91FF7E`) and
//! `flt_91FF8A` (`scene::angle_between_vectors`, both normalised in place), into
//! `flt_8F3975`; 2 pi less it when the level part's x (normalised) is not negative.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! One caller, `sub_4C3EEE`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::framework::rva;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::rounded;
use crate::scene::vector3::angle_between_vectors;

const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view's matrix
const LEVEL_RVA: Slot = Slot::at(0x91FF7E); // flt_91FF7E: three floats
const OTHER_RVA: Slot = Slot::at(0x91FF8A); // flt_91FF8A: three floats
const HEADING_RVA: Slot = Slot::at(0x8F3975); // g_marksViewHeading

/// The Rust version (see the header).
pub unsafe fn view_heading() {
    unsafe {
        let x = x87::loaded(game_read(VIEW_RVA + 8));
        let z = x87::loaded(game_read(VIEW_RVA + 0x28));
        game_write(LEVEL_RVA, [x, 0, z]);
        let level = game_addr(LEVEL_RVA) as u32;
        let other = game_addr(OTHER_RVA) as u32;
        let mut heading = rounded(angle_between_vectors(level, other));
        if game_read::<u32>(LEVEL_RVA) & 0x8000_0000 == 0 {
            heading = x87::two_pi_less(heading);
        }
        game_write(HEADING_RVA, heading);
    }
}
