//! `effects::view_heading`: the view's heading: the angle between its forward direction's level
//! part (`SceneMatrices.view_place[0]` +8, 0, +0x28, into `DrawFrame.level[0]`) and
//! `DrawFrame.other[0]` (`scene::angle_between_vectors`, both normalised in place), into
//! `ViewVars.heading`; 2 pi less it when the level part's x (normalised) is not negative. No level
//! part: the heading left as it was.
//!
//! One caller, `effects::draw_frame`.

use crate::math::x87;
use crate::memory::session::write_bits;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::rounded_value;
use crate::scene::vector3::angle_between_vectors;

pub fn view_heading(frame: &Frame, effects: &crate::effects::owner::Effects) {
    let effects_frame = &effects.frame;
    let x = x87::loaded(&frame.matrices.get().view_place[2]);
    let z = x87::loaded(&frame.matrices.get().view_place[10]);
    write_bits(
        &effects_frame.draw_frame.level,
        [x.to_bits(), 0, z.to_bits()],
    );
    // A forward direction with no level part (the view looking straight up or down, or its
    // matrix not set yet: a stage's first frame) has no heading, and it is left as it was
    // (the earlier code normalised the zero vector all the same and got a heading from NaNs, a
    // slip: nothing in the game reads the heading).
    if x == 0.0 && z == 0.0 {
        return;
    }
    let mut heading = rounded_value(angle_between_vectors(
        &effects_frame.draw_frame.level,
        &effects_frame.draw_frame.other,
        &effects.setup.work,
    ));
    if !effects_frame.draw_frame.level[0].get().is_sign_negative() {
        heading = x87::two_pi_less(heading.to_bits());
    }
    effects_frame.view_vars.heading.set(heading);
}
