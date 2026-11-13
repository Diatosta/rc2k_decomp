//! `effects::view_motion` (0x4C4D40): the view's motion since the last frame. With the view
//! matrix: its third row (+0x20 ..) kept on the stack; its place (the origin turned by it,
//! `scene::rotate_point`, unrounded) less the last place into the motion and the stack; that
//! step made length 1 (`scene::normalise_vector3`); the third row's dot product with it kept
//! (0 for a view that has not moved: the original's NaN there, section 16);
//! the place (turned again) kept as the last; then the motion each over the frame's ticks
//! (`fidiv`; left whole in a frame of no ticks, the original's NaNs there, section 16).
//!
//! Caller: `effects::draw_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::errors::Refused;
use crate::math::cells::load;
use crate::math::x87;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::{extended, rounded_value};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{dot_product3, normalise};
use crate::session::PlainFloat;

pub fn view_motion(frame: &Frame, effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    let m = &frame.matrices.get().view_place;
    // The original's six floats on its stack: the third row, then the step.
    let local: [PlainFloat; 6] = std::array::from_fn(|_| PlainFloat::new(0.0));
    for k in 0..3 {
        local[k].set(x87::loaded(&m[8 + k]));
    }
    let zero = [extended(0); 3];
    let place = rotate_point(m, zero);
    for k in 0..3 {
        let step = x87::ext_less(place[k], &effects.frame.draw_frame.last[k]);
        effects.frame.draw_work.step[k].set(step);
        local[3 + k].set(step);
    }
    // A view that has not moved since the last frame (a stage's first frame, a replay's
    // camera held still before the start) has a step of length 0, which has no direction.
    // The original normalised it all the same: 1 / 0 times 0 made the three NaNs and the
    // facing a NaN (a 1999 slip; nothing in the game reads the facing). Here no motion faces
    // no way: 0.
    let moved = local[3..].iter().any(|s| s.get() != 0.0);
    let facing = if moved {
        normalise(&local[3..]);
        rounded_value(dot_product3(load(&local[..3]), load(&local[3..])))
    } else {
        0.0
    };
    effects.frame.view_vars.facing.set(facing);
    let place = rotate_point(m, zero);
    for (k, p) in place.iter().enumerate() {
        effects.frame.draw_frame.last[k].set(rounded_value(*p));
    }
    let ticks: i32 = effects.setup.clock.frame_ticks.get() as i32;
    // A frame of no ticks (a stage's first, from `race_init`, before the clock runs) has no
    // step per tick. The original divided by 0 all the same (NaNs for a step of 0,
    // infinities for the rest: a 1999 slip; nothing reads the step per tick). Here the
    // step is left whole.
    if ticks == 0 {
        return Ok(());
    }
    for item in effects.frame.draw_work.step.iter().take(3) {
        let a = &item;
        a.set(x87::over_int(a, ticks));
    }
    Ok(())
}
