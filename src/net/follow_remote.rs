//! `net::follow_remote` (0x464883): a remote car's step in a network race: while it has
//! steps of a glide left (the current car's block +0xFCC, set to 32 by
//! `net::apply_message`), one less, and a 32nd of the way (+0xFC0 .. +0xFC8) added to its
//! place (+0x134 .. +0x13C).
//!
//! In:  nothing
//! Out: nothing (the original leaves eax +0xFC8 after a glide step; its one caller reads no
//!      register after it: `lq -s 0x464883`, DF only)
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Calls nothing.

use crate::net::store::NET;
use crate::net::{CAR, a};
use rc2k_core::net::race;

/// The Rust version.
pub fn follow_remote() {
    let car = a(CAR);
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    let wr = |at: u32, v: u32| unsafe { (at as *mut u32).write_volatile(v) };
    // Converted: `rc2k_core::net::race::glide` (the block read only with the race on).
    let three = |at: u32| [rd(car + at), rd(car + at + 4), rd(car + at + 8)];
    if NET.session.get() == 0 {
        return;
    }
    let steps = rd(car + race::GLIDE_STEPS);
    if steps == 0 {
        return;
    }
    let step = three(race::WANTED);
    if let Some((left, place)) = race::glide(true, steps, three(race::PLACE), step) {
        wr(car + race::GLIDE_STEPS, left);
        for (k, v) in (0..).zip(place) {
            wr(car + race::PLACE + 4 * k, v);
        }
    }
}
