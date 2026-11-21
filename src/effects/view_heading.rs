//! `effects::view_heading`: the view's heading: the angle between its forward direction's level
//! part (the x and z of the third row of `SceneMatrices.view_place`, with 0 between, into
//! `DrawFrame.level[0]`) and `DrawFrame.other[0]` (`scene::angle_between_vectors`, both
//! normalised in place), into `ViewVars.heading`; 2 pi less it when the level part's x (normalised) is not negative. No level
//! part: the heading left as it was.

use crate::math::matrix3::rounded_value;
use crate::math::x87;
use crate::race::owner_frame::Frame;
use crate::scene::vector3::angle_between_vectors_plain;

pub fn view_heading(frame: &Frame, effects: &crate::effects::owner::Effects) {
    let effects_frame = &effects.frame;
    let view_place = frame.view_kept.borrow().view_place;
    let x = x87::loaded(view_place[2]);
    let z = x87::loaded(view_place[10]);
    effects_frame.draw_frame.borrow_mut().level = [x, 0.0, z];
    // A forward direction with no level part (the view looking straight up or down, or its
    // matrix not set yet: a stage's first frame) has no heading, and it is left as it was
    // (normalising the zero vector would give a heading from NaNs; nothing reads it).
    if x == 0.0 && z == 0.0 {
        return;
    }
    let mut kept = effects_frame.draw_frame.borrow_mut();
    let kept = &mut *kept;
    let mut heading = rounded_value(angle_between_vectors_plain(
        &mut kept.level,
        &mut kept.other,
        &effects.setup.work,
    ));
    if !kept.level[0].is_sign_negative() {
        heading = x87::two_pi_less(heading.to_bits());
    }
    effects_frame.view_vars.borrow_mut().heading = heading;
}
