//! The clock of the race's sounds: `dword_5124E8` the time of the last turn,
//! `dword_5124EC` the turns a second as a float, `dword_5124F0` the last turn's
//! milliseconds, `dword_5124F4` the turns counted. `sound::start_race_clock` (0x40F1E9)
//! sets it going and `sound::step_race_clock` (0x40F20E) says whether a turn is due.

use crate::framework::{game_read, game_write, rva};

pub const LAST_RVA: usize = rva(0x5124E8); // dword_5124E8
pub const RATE_RVA: usize = rva(0x5124EC); // dword_5124EC
pub const STEP_RVA: usize = rva(0x5124F0); // dword_5124F0
pub const TURNS_RVA: usize = rva(0x5124F4); // dword_5124F4

fn load() -> rc2k_core::sound::clock::RaceClock {
    unsafe {
        rc2k_core::sound::clock::RaceClock {
            last: game_read(LAST_RVA),
            turns_a_second: f32::from_bits(game_read(RATE_RVA)),
            step: game_read(STEP_RVA),
            turns: game_read(TURNS_RVA),
        }
    }
}

fn store(c: &rc2k_core::sound::clock::RaceClock) {
    unsafe {
        game_write(LAST_RVA, c.last);
        game_write(RATE_RVA, c.turns_a_second.to_bits());
        game_write(STEP_RVA, c.step);
        game_write(TURNS_RVA, c.turns);
    }
}

/// `sound::start_race_clock` (0x40F1E9). Converted: `rc2k_core::sound::clock`.
pub unsafe fn start_race_clock(now: u32) {
    let mut c = load();
    c.start(now);
    store(&c);
}

/// `sound::step_race_clock` (0x40F20E): true if 20 ms or more have passed since the last
/// turn, which then becomes this one. Converted: `rc2k_core::sound::clock`; nothing is
/// stored when no turn is due.
pub unsafe fn step_race_clock(now: u32) -> bool {
    let mut c = load();
    let due = c.step(now);
    if due {
        store(&c);
    }
    due
}
