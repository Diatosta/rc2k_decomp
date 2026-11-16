//! The clock of the race's sounds: `RaceClock.last` the time of the last turn,
//! `RaceClock.turns_a_second` the turns a second as a float, `RaceClock.step` the last turn's
//! milliseconds, `RaceClock.turns` the turns counted. `sound::start_race_clock` sets it going and
//! `sound::step_race_clock` says whether a turn is due.

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

/// `sound::start_race_clock`. The logic is `rc2k_core::sound::clock`.
pub fn start_race_clock(now: u32, sound: &crate::sound::owner::Sound) {
    let clock = sound.race.clock.get();
    let mut c = load(clock);
    c.start(now);
    store(clock, &c);
}

/// `sound::step_race_clock`: true if 20 ms or more have passed since the last turn, which then
/// becomes this one. The logic is `rc2k_core::sound::clock`; nothing is stored when no turn is due.
pub fn step_race_clock(now: u32, sound: &crate::sound::owner::Sound) -> bool {
    let clock = sound.race.clock.get();
    let mut c = load(clock);
    let due = c.step(now);
    if due {
        store(clock, &c);
    }
    due
}
