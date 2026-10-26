//! `physics::place_car_sound` (0x45CC5C): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) placed: +0x224 1; the car's place (`g_car`
//! +0x134 x, +0x138 y, +0x13C z) less +0x6A4 .. +0x6AC (`fild` / `fisub`, exact), turned
//! by `g_viewTurnMatrix` (`scene::rotate_point`), each part times `flt_5C5DA0` (1/65536)
//! into +0x214, +0x218, +0x21C; +0x10 1. All in `pusha` / `popa`. A reading: the car's
//! sound placed against the view (+0x6A4 .. the listener's place).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller returns)
//!
//! Caller: `sub_441324` (its tail). Refuses (before anything) a null sound object.

use crate::errors::{BadInput, Refused};
use crate::math::x87_run::{ext_times, ints_diff_80};
use crate::physics::car_store::CAR;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::work_store::RACE_WORK;
use crate::scene::matrix3::rounded;
use crate::scene::rotate_point::rotate_point;
use crate::session::Dword;

fn object() -> u32 {
    crate::sound::store::RACE_SOUNDS.table[(RACE_WORK.current_car.get() & 7) as usize].get()
}

/// The exact x87 version (for `physics::sound_on_bit`).
pub fn place_car_sound() -> Result<(), Refused> {
    let o = object();
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let so = crate::sound::store::SoundObject::at(o)?;
    so.slot_b.start.set(1);
    let d = |a: &Dword, b: &Dword| ints_diff_80(a.get() as i32, b.get() as i32);
    let p = [
        d(&CAR.position[0], &CAR.camera_position[0]),
        d(&CAR.position[1], &CAR.camera_position[1]),
        d(&CAR.position[2], &CAR.camera_position[2]),
    ];
    let out = rotate_point(MATRICES.view_turn_matrix[0].addr(), p);
    let k = NUMBERS.one_over_65536.bits();
    for (n, e) in out.iter().enumerate() {
        so.place_b[n].set_bits(rounded(ext_times(*e, k)));
    }
    so.new_revs.set(1);
    Ok(())
}
