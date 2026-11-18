//! Places the current car's sound object (in the race's sound objects): active flag 1; the car's
//! place less the listener's (exact integer arithmetic), turned by the view's turn matrix
//! (`scene::rotate_point`), each part times `one_over_65536` into the object's place; update
//! flag 1. A reading: the car's sound placed against the view.
//!
//! Refuses (before anything) a null sound object.

use crate::errors::Refused;
use crate::math::x87::{ext_times, ints_diff_80};
use crate::memory::session::PlainDword;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix3::rounded_value;
use crate::scene::rotate_point::rotate_point;

/// The exact x87 version (for `physics::sound_on_bit`).
pub fn place_car_sound(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    shared: &RaceShared,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let so = crate::sound::store::current_object(sound, shared.work.get());
    so.slot_b.start.set(1);
    let d = |a: &PlainDword, b: &PlainDword| ints_diff_80(a.get() as i32, b.get() as i32);
    let listener = crate::race::heard_cars::listener(shared, shared.car());
    let p = [
        d(&shared.car().position[0], &listener.camera_position[0]),
        d(&shared.car().position[1], &listener.camera_position[1]),
        d(&shared.car().position[2], &listener.camera_position[2]),
    ];
    let out = rotate_point(shared.heard_turn.get().turn(matrices), p);
    let k = numbers.one_over_65536.get();
    for (n, e) in out.iter().enumerate() {
        so.place_b[n].set(rounded_value(ext_times(*e, k)));
    }
    so.new_revs.set(1);
    Ok(())
}
