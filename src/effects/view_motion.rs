//! `effects::view_motion`: the view's motion since the last frame. With the view matrix: its third
//! row kept; its place (the origin turned by it, `scene::rotate_point`, unrounded) less
//! the last place into the motion; that step made length 1 (`scene::vector3::normalise`); the third
//! row's dot product with it kept (0 for a view that has not moved); the place (turned again) kept
//! as the last; then the motion each over the frame's ticks (left whole in a frame of no ticks).

use crate::errors::Refused;
use crate::math::cells::load;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::memory::session::PlainFloat;
use crate::race::owner_frame::Frame;
use crate::scene::rotate_point::rotate_point_by;
use crate::scene::vector3::{dot_product3, normalise};

pub fn view_motion(frame: &Frame, effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    let m = frame.view_kept.borrow().view_place;
    // Six floats: the third row, then the step.
    let local: [PlainFloat; 6] = std::array::from_fn(|_| PlainFloat::new(0.0));
    for k in 0..3 {
        local[k].set(x87::loaded(m[8 + k]));
    }
    let zero = [extended(0); 3];
    let place = rotate_point_by(&m, zero);
    let last = effects.frame.draw_frame.borrow().last;
    for k in 0..3 {
        let step = x87::ext_less(place[k], last[k]);
        effects.frame.draw_work.borrow_mut().step[k] = step;
        local[3 + k].set(step);
    }
    // A view that has not moved since the last frame (a stage's first frame, a replay's
    // camera held still before the start) has a step of length 0, which has no direction:
    // no motion faces no way, 0 (normalising it would give NaNs; nothing reads the facing).
    let moved = local[3..].iter().any(|s| s.get() != 0.0);
    let facing = if moved {
        normalise(&local[3..]);
        rounded_value(dot_product3(load(&local[..3]), load(&local[3..])))
    } else {
        0.0
    };
    effects.frame.view_vars.borrow_mut().facing = facing;
    let place = rotate_point_by(&m, zero);
    for (k, p) in place.iter().enumerate() {
        effects.frame.draw_frame.borrow_mut().last[k] = rounded_value(*p);
    }
    let ticks: i32 = effects.setup.clock.borrow().frame_ticks as i32;
    // A frame of no ticks (a stage's first, from `race_init`, before the clock runs) has no
    // step per tick: the step is left whole (dividing by 0 would give a slip nothing reads).
    if ticks == 0 {
        return Ok(());
    }
    for item in effects.frame.draw_work.borrow_mut().step.iter_mut().take(3) {
        *item = x87::over_int(*item, ticks);
    }
    Ok(())
}
