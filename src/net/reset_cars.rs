//! `net::reset_cars` (0x463E52): the eight cars' blocks (`g_carBlocks`) made ready for a
//! network race: +0xAA8 the car's number + 1, +0x684 4, +0x28 2, +0x34, +0x30, +0x20,
//! +0x3C 0x80, then `net::clear_car_state`; `dword_6238CC` and `dword_6238D0` 0.
//!
//! In:  nothing
//! Out: nothing (the original leaves ecx 8 and the rest kept; its one caller,
//!      `net::set_up_race`, still hands its registers on to `race::load_stage`, whose reads
//!      are not known, so it sets ecx 8 itself)
//! Caller: `net::set_up_race`.

use crate::net::clear_car_state::clear_car_state;
use crate::net::lobby::p;
use crate::race::work_store::RACE_WORK;
use rc2k_core::net::cars;

const AFTER: [usize; 2] = [0x6238CC, 0x6238D0]; // dword_6238CC, dword_6238D0

/// The Rust version.
pub fn reset_cars() {
    for car in 0..cars::CARS {
        let block = RACE_WORK.car_block(car as usize);
        // Converted: `rc2k_core::net::cars::reset_fields`.
        for (offset, v) in cars::reset_fields(car) {
            unsafe { ((block + offset) as *mut u32).write_unaligned(v) };
        }
        clear_car_state(block);
    }
    for x in AFTER {
        p(x, 0);
    }
}
