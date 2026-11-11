//! `effects::view_heading` (0x4C4CEC): the view's heading: the angle between its forward
//! direction's level part (`SceneMatrices.view_place[0]` +8, 0, +0x28, into `DrawFrame.level[0]`) and
//! `DrawFrame.other[0]` (`scene::angle_between_vectors`, both normalised in place), into
//! `ViewVars.heading`; 2 pi less it when the level part's x (normalised) is not negative.
//!
//! One caller, `effects::draw_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::math::x87;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::rounded;
use crate::scene::vector3::angle_between_vectors;
use crate::session::write_bits;

pub fn view_heading(frame: &Frame, effects: &crate::effects::owner::Effects) {
    let effects_frame = &effects.frame;
    let x = x87::loaded(&frame.matrices.view_place[2]);
    let z = x87::loaded(&frame.matrices.view_place[10]);
    write_bits(
        &effects_frame.draw_frame.level,
        [x.to_bits(), 0, z.to_bits()],
    );
    let mut heading = rounded(angle_between_vectors(
        &effects_frame.draw_frame.level,
        &effects_frame.draw_frame.other,
    ));
    if !effects_frame.draw_frame.level[0].get().is_sign_negative() {
        heading = x87::two_pi_less(heading).to_bits();
    }
    effects_frame.view_vars.heading.set_bits(heading);
}
