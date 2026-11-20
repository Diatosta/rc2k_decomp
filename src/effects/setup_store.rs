//! The marks' clock, settings flag and clock constants.

use crate::memory::session::{PlainDword, PlainFloat};

/// The marks' clock (`effects::tick_clock`).
#[derive(rc2k_derive::Cells)]
pub struct MarksClock {
    /// 0 until `effects::tick_clock` has run once (`init_marks` clears it).
    pub started: PlainDword,
    /// The scene clock at the last tick.
    pub last: PlainDword,
    /// The clock's ticks since the last frame (0 unless the settings' `other_marks_shown` is 1).
    pub frame_ticks: PlainDword,
    /// The ticks' bits taken as a float, times a constant (a game bug, kept as is).
    pub ticks_scaled: PlainFloat,
    /// The ticks shifted right 4.
    pub sixteenth: PlainDword,
    /// The ticks shifted right 2.
    pub quarter: PlainDword,
    /// One over the ticks; the first tick's value is a constant.
    pub reciprocal: PlainFloat,
    /// `reciprocal` times a constant.
    pub reciprocal_scaled: PlainFloat,
    /// The scene clock now.
    pub now: PlainDword,
    /// The clock at the tick before.
    pub before: PlainDword,
    /// A copy of the settings' `fresh_marks_flags[1]` (`effects::frame_marks`), 1 from
    /// `init_marks`; the second is 1 from `init_marks` (nothing reads it).
    pub settings_copies: [PlainDword; 2],
}

impl MarksClock {
    pub const fn new() -> Self {
        Self {
            started: PlainDword::new(0x0),
            last: PlainDword::new(0x0),
            frame_ticks: PlainDword::new(0x0),
            ticks_scaled: PlainFloat::new(0.0),
            sixteenth: PlainDword::new(0x0),
            quarter: PlainDword::new(0x0),
            reciprocal: PlainFloat::new(0.0),
            reciprocal_scaled: PlainFloat::new(0.0),
            now: PlainDword::new(0x0),
            before: PlainDword::new(0x0),
            settings_copies: [PlainDword::new(0x0), PlainDword::new(0x0)],
        }
    }
}

/// The marks' settings flag: it is 1 once the game has handed `effects::init_marks` the
/// settings, and 0 before (nothing runs without them). The settings are the race's one
/// record, `EFFECTS_SETTINGS`.
#[derive(rc2k_derive::Cells)]
pub struct SettingsPointer {
    /// 1 when the settings are set, else 0.
    pub is_set: PlainDword,
}

impl SettingsPointer {
    pub const fn new() -> Self {
        Self {
            is_set: PlainDword::new(0),
        }
    }
}

/// The marks' clock constants.
#[derive(rc2k_derive::Cells)]
pub struct ClockK {
    /// The reciprocal the first tick starts with
    pub first_reciprocal: PlainFloat,
    /// The ticks' scale
    pub ticks_scale: PlainFloat,
}

impl ClockK {
    /// ClockK's first values.
    pub const fn new() -> Self {
        Self {
            first_reciprocal: PlainFloat::new(1e-6),
            ticks_scale: PlainFloat::new(0.1),
        }
    }
}
