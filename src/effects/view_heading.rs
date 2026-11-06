//! `effects::view_heading` (0x4C4CEC): the view's heading: the angle between its forward
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

use crate::math::x87;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::rounded;
use crate::scene::vector3::angle_between_vectors;
use crate::session::write_bits;

/// The Rust version (see the header).
pub fn view_heading(frame: &Frame) {
    let x = x87::loaded(frame.matrices.view_place[2].bits());
    let z = x87::loaded(frame.matrices.view_place[10].bits());
    write_bits(
        &crate::effects::owner::effects().frame.draw_frame.level,
        [x, 0, z],
    );
    let mut heading = rounded(angle_between_vectors(
        &crate::effects::owner::effects().frame.draw_frame.level,
        &crate::effects::owner::effects().frame.draw_frame.other,
    ));
    if crate::effects::owner::effects().frame.draw_frame.level[0].bits() & 0x8000_0000 == 0 {
        heading = x87::two_pi_less(heading);
    }
    crate::effects::owner::effects()
        .frame
        .view_vars
        .heading
        .set_bits(heading);
}
