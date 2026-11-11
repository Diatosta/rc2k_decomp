//! `effects::view_motion` (0x4C4D40): the view's motion since the last frame. With the view
//! matrix: its third row (+0x20 ..) kept on the stack; its place (the origin turned by it,
//! `scene::rotate_point`, unrounded) less the last place into the motion and the stack; that
//! step made length 1 (`scene::normalise_vector3`); the third row's dot product with it kept;
//! the place (turned again) kept as the last; then the motion each over the frame's ticks
//! (`fidiv`).
//!
//! Caller: `effects::draw_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::errors::Refused;
use crate::math::cells::load;
use crate::math::x87;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{dot_product3, normalise};
use crate::session::Dword;

pub fn view_motion(frame: &Frame, effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    let m = &frame.matrices.view_place;
    // The original's six floats on its stack: the third row, then the step.
    let local: [Dword; 6] = std::array::from_fn(|_| Dword::new(0));
    for k in 0..3 {
        local[k].set(x87::loaded(&m[8 + k]).to_bits());
    }
    let zero = [extended(0); 3];
    let place = rotate_point(m, zero);
    for k in 0..3 {
        let step = x87::ext_less(place[k], effects.frame.draw_frame.last[k].bits());
        effects.frame.draw_work.step[k].set_bits(step.to_bits());
        local[3 + k].set(step.to_bits());
    }
    normalise(&local[3..]);
    effects
        .frame
        .view_vars
        .facing
        .set_bits(rounded(dot_product3(load(&local[..3]), load(&local[3..]))));
    let place = rotate_point(m, zero);
    for (k, p) in place.iter().enumerate() {
        effects.frame.draw_frame.last[k].set_bits(rounded(*p));
    }
    let ticks: i32 = effects.setup.clock.frame_ticks.get() as i32;
    for k in 0..3 {
        let a = &effects.frame.draw_work.step[k];
        a.set(x87::over_int(a, ticks));
    }
    Ok(())
}
