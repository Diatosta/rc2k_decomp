//! The clock of the race's sounds: `dword_5124E8` the time of the last turn,
//! `dword_5124EC` the turns a second as a float, `dword_5124F0` the last turn's
//! milliseconds, `dword_5124F4` the turns counted. `sound::start_race_clock` (0x40F1E9)
//! sets it going and `sound::step_race_clock` (0x40F20E) says whether a turn is due.

use crate::sound::store::RaceClock;

fn load(clock: &RaceClock) -> rc2k_core::sound::clock::RaceClock {
    rc2k_core::sound::clock::RaceClock {
        last: clock.last.get(),
        turns_a_second: clock.turns_a_second.get(),
        step: clock.step.get(),
        turns: clock.turns.get(),
    }
}

fn store(clock: &RaceClock, c: &rc2k_core::sound::clock::RaceClock) {
    clock.last.set(c.last);
    clock.turns_a_second.set(c.turns_a_second);
    clock.step.set(c.step);
    clock.turns.set(c.turns);
}

/// `sound::start_race_clock` (0x40F1E9). Converted: `rc2k_core::sound::clock`.
pub fn start_race_clock(now: u32, sound: &crate::sound::owner::Sound) {
    let clock = &sound.race.clock;
    let mut c = load(clock);
    c.start(now);
    store(clock, &c);
}

/// `sound::step_race_clock` (0x40F20E): true if 20 ms or more have passed since the last
/// turn, which then becomes this one. Converted: `rc2k_core::sound::clock`; nothing is
/// stored when no turn is due.
pub fn step_race_clock(now: u32, sound: &crate::sound::owner::Sound) -> bool {
    let clock = &sound.race.clock;
    let mut c = load(clock);
    let due = c.step(now);
    if due {
        store(clock, &c);
    }
    due
}
