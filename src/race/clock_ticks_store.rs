//! The race clock's ticks: the text ("demof\build"), the 0x20 marks set to -1 at the start and end
//! of a race, and the last tick and tick count of the race sounds. Stops before the kind tables
//! (`race::kind_records_store`).
//!

use crate::memory::cells::ZeroCells;

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ClockTicks {
    /// The text "demof\\build" (12 bytes) and the bytes after it: not read.
    pub unused_text: [PlainDword; 6],
    /// 0x20 dwords set to -1 by `drive::race_loop` and `drive::finish_run`.
    pub marks: [PlainDword; 32],
    /// The scene clock at the last race-sound tick (`sound::race_sound_tick`,
    /// `drive::race_loop`, `drive::finish_run`).
    pub sound_tick_last: PlainDword,
    /// Set to the scene clock at the race's start (`drive::race_loop`,
    /// `drive::finish_run`).
    pub clock_a: PlainDword,
    /// The ticks counted (`sound::race_sound_tick`, `race::stage_loading`).
    pub sound_ticks: PlainDword,
    /// The scene clock when the stage was loaded (`race::stage_loading`).
    pub clock_b: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ClockTicks {
    pub const fn new() -> Self {
        Self {
            unused_text: [
                PlainDword::new(0x6F6D6564),
                PlainDword::new(0x75625C66),
                PlainDword::new(0x646C69),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            ..ZeroCells::ZERO
        }
    }
}
