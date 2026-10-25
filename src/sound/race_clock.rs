//! The clock of the race's sounds: `dword_5124E8` the time of the last turn,
//! `dword_5124EC` the turns a second as a float, `dword_5124F0` the last turn's
//! milliseconds, `dword_5124F4` the turns counted. `sound::start_race_clock` (0x40F1E9)
//! sets it going and `sound::step_race_clock` (0x40F20E) says whether a turn is due.

use crate::sound::store::RACE_CLOCK as C;

fn load() -> rc2k_core::sound::clock::RaceClock {
    rc2k_core::sound::clock::RaceClock {
        last: C.last.get(),
        turns_a_second: C.turns_a_second.get(),
        step: C.step.get(),
        turns: C.turns.get(),
    }
}

fn store(c: &rc2k_core::sound::clock::RaceClock) {
    C.last.set(c.last);
    C.turns_a_second.set(c.turns_a_second);
    C.step.set(c.step);
    C.turns.set(c.turns);
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
