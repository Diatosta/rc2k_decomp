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
use crate::framework::rva;
use crate::math::x87_run::{ext_times, ints_diff_80};
use crate::scene::matrix3::rounded;
use crate::scene::rotate_point::rotate_point;
use crate::sound::store::ObjectsSlot;
use crate::store::game_addr;

fn g(x: impl crate::store::At) -> u32 {
    unsafe { crate::store::at_read(x) }
}

const VIEW_TURN: usize = 0x5EA24C; // g_viewTurnMatrix
const OBJECTS: ObjectsSlot = ObjectsSlot::at(0x512A10); // g_raceSoundObjects

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn object() -> u32 {
    g(OBJECTS + 4 * (g(0x71B3F8) & 7) as usize)
}

/// The exact x87 version (for `physics::sound_on_bit`).
pub unsafe fn place_car_sound() -> Result<(), Refused> {
    let o = object();
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    unsafe {
        put(o + 0x224, 1);
        let d = |a: usize, b: usize| ints_diff_80(g(a) as i32, g(b) as i32);
        let p = [
            d(0x71A10C, 0x71A67C),
            d(0x71A110, 0x71A680),
            d(0x71A114, 0x71A684),
        ];
        let out = rotate_point(game_addr(rva(VIEW_TURN)) as u32, p);
        let k = g(0x5C5DA0);
        for (n, e) in out.iter().enumerate() {
            put(o + 0x214 + 4 * n as u32, rounded(ext_times(*e, k)));
        }
        put(o + 0x10, 1);
    }
    Ok(())
}
