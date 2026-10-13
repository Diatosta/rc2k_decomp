//! The clock of the race's sounds: `dword_5124E8` the time of the last turn,
//! `dword_5124EC` the turns a second as a float, `dword_5124F0` the last turn's
//! milliseconds, `dword_5124F4` the turns counted. `sound::start_race_clock` (0x40F1E9)
//! sets it going and `sound::step_race_clock` (0x40F20E) says whether a turn is due.

use crate::framework::{game_read, game_write, rva};
use crate::service::harness::{Region, at};

pub const LAST_RVA: usize = rva(0x5124E8); // dword_5124E8
pub const RATE_RVA: usize = rva(0x5124EC); // dword_5124EC
pub const STEP_RVA: usize = rva(0x5124F0); // dword_5124F0
pub const TURNS_RVA: usize = rva(0x5124F4); // dword_5124F4

pub fn regions() -> [Region; 4] {
    [at(LAST_RVA), at(RATE_RVA), at(STEP_RVA), at(TURNS_RVA)]
}

/// `sound::start_race_clock` (0x40F1E9).
pub unsafe fn start_race_clock(now: u32) {
    unsafe {
        game_write(LAST_RVA, now);
        game_write(RATE_RVA, 50f32.to_bits());
        game_write(TURNS_RVA, 0u32);
    }
}

/// `sound::step_race_clock` (0x40F20E): true if 20 ms or more have passed since the last
/// turn, which then becomes this one.
pub unsafe fn step_race_clock(now: u32) -> bool {
    unsafe {
        let last: u32 = game_read(LAST_RVA);
        let passed = now.wrapping_sub(last);
        if passed < 0x14 {
            return false;
        }
        game_write(LAST_RVA, last.wrapping_add(passed));
        let step = passed.min(1000);
        game_write(STEP_RVA, step);
        // A whole number of at most 50: the float holds it exactly.
        game_write(RATE_RVA, ((1000 / step) as f32).to_bits());
        game_write(TURNS_RVA, game_read::<u32>(TURNS_RVA).wrapping_add(1));
        true
    }
}
