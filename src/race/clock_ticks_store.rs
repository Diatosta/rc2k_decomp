//! The race's sound tick clock: the scene clock at the last race-sound tick.

pub struct ClockTicks {
    /// The scene clock at the last race-sound tick (`sound::race_sound_tick`,
    /// `drive::race_loop`, `drive::finish_run`).
    pub sound_tick_last: u32,
}

impl ClockTicks {
    pub const fn new() -> Self {
        ClockTicks { sound_tick_last: 0 }
    }
}
