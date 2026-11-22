//! Places the current car's sound object (in the race's sound objects): active flag 1; the car's
//! place less the listener's (exact integer arithmetic), turned by the view's turn matrix
//! (`scene::rotate_point`), each part times `one_over_65536` into the object's place; update
//! flag 1. A reading: the car's sound placed against the view.
//!
//! Refuses (before anything) a null sound object.

use crate::errors::Refused;
use crate::math::matrix3::rounded_value;
use crate::math::x87::{ext_times, ints_diff_80};
use crate::memory::plain_floats::FloatRow;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix::Matrix;
use crate::scene::rotate_point::rotate_point_by;

/// The exact x87 version (for `physics::sound_on_bit`).
pub(super) fn place_car_sound(
    numbers: &Numbers,
    view_turn: &Matrix,
    shared: &RaceShared,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let so = crate::sound::store::current_object(sound, &shared.work());
    so.borrow_mut().slot_b.start = 1;
    let d = |a: u32, b: u32| ints_diff_80(a as i32, b as i32);
    let listener = crate::race::heard_cars::listener_place(shared)
        .unwrap_or_else(|| shared.car().camera_position);
    let p = [
        d(shared.car().position.bits_at(0), listener.bits_at(0)),
        d(shared.car().position.bits_at(1), listener.bits_at(1)),
        d(shared.car().position.bits_at(2), listener.bits_at(2)),
    ];
    let out = rotate_point_by(&shared.heard_turn.get().turn(view_turn), p);
    let k = numbers.one_over_65536.get();
    for (n, e) in out.iter().enumerate() {
        so.borrow_mut().place_b[n] = rounded_value(ext_times(*e, k));
    }
    so.borrow_mut().new_revs = 1;
    Ok(())
}
