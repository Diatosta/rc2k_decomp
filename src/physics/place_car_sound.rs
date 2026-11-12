//! `physics::place_car_sound` (0x45CC5C): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) placed: +0x224 1; the car's place (`g_car`
//! +0x134 x, +0x138 y, +0x13C z) less +0x6A4 .. +0x6AC (`fild` / `fisub`, exact), turned
//! by `g_viewTurnMatrix` (`scene::rotate_point`), each part times `one_over_65536` (1/65536)
//! into +0x214, +0x218, +0x21C; +0x10 1. A reading: the car's
//! sound placed against the view (+0x6A4 .. the listener's place).
//!
//! Caller: `physics::sound_on_bit` (its tail). Refuses (before anything) a null sound object.

use crate::errors::Refused;
use crate::math::x87::{ext_times, ints_diff_80};
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix3::rounded;
use crate::scene::rotate_point::rotate_point;
use crate::session::Dword;

/// The exact x87 version (for `physics::sound_on_bit`).
pub fn place_car_sound(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    shared: &RaceShared,
) -> Result<(), Refused> {
    let so = crate::sound::store::current_object();
    so.slot_b.start.set(1);
    let d = |a: &Dword, b: &Dword| ints_diff_80(a.get() as i32, b.get() as i32);
    let p = [
        d(&shared.car().position[0], &shared.car().camera_position[0]),
        d(&shared.car().position[1], &shared.car().camera_position[1]),
        d(&shared.car().position[2], &shared.car().camera_position[2]),
    ];
    let out = rotate_point(crate::race::heard_turn::turn(matrices), p);
    let k = numbers.one_over_65536.bits();
    for (n, e) in out.iter().enumerate() {
        so.place_b[n].set_bits(rounded(ext_times(*e, k)));
    }
    so.new_revs.set(1);
    Ok(())
}
