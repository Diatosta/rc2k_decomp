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

use crate::marks::frame_state::{DRAW_FRAME, VIEW_VARS};
use crate::math::x87;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::rounded;
use crate::scene::vector3::angle_between_vectors;
use crate::session::{address_of, write_bits};

/// The Rust version (see the header).
pub fn view_heading() {
    let x = x87::loaded(MATRICES.view_place[2].bits());
    let z = x87::loaded(MATRICES.view_place[10].bits());
    write_bits(&DRAW_FRAME.level, [x, 0, z]);
    let level = address_of(&DRAW_FRAME.level) as u32;
    let other = address_of(&DRAW_FRAME.other) as u32;
    let mut heading = rounded(angle_between_vectors(level, other));
    if DRAW_FRAME.level[0].bits() & 0x8000_0000 == 0 {
        heading = x87::two_pi_less(heading);
    }
    VIEW_VARS.heading.set_bits(heading);
}
