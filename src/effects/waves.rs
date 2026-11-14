//! The marks' wave tables: `MarksSines`, `WaveLevels` (the lens flare's parts' levels) and
//! `WaveState` (the made-once flag and constants).

use crate::session::{PlainDword, PlainFloat};

/// The marks' sine table.
#[derive(rc2k_derive::Cells)]
pub struct MarksSines {
    /// 0x140 sines of 0, step, 2 step .. (`effects::make_wave_tables`; nothing reads them).
    pub table: [PlainFloat; 320],
}

impl MarksSines {
    /// The values the game starts with.
    pub const fn new() -> Self {
        MarksSines {
            table: [const { PlainFloat::new(0.0) }; 320],
        }
    }
}

/// The wave levels.
#[derive(rc2k_derive::Cells)]
pub struct WaveLevels {
    /// The lens flare's parts' places along the line: whole numbers at first, floats once
    /// `effects::make_wave_tables` has run (as bits).
    pub levels: [PlainDword; 17],
}

impl WaveLevels {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WaveLevels {
            levels: [
                PlainDword::new(0x0),
                PlainDword::new(0x9),
                PlainDword::new(0x2B),
                PlainDword::new(0x62),
                PlainDword::new(0xB4),
                PlainDword::new(0xBF),
                PlainDword::new(0xC6),
                PlainDword::new(0xE7),
                PlainDword::new(0x109),
                PlainDword::new(0x139),
                PlainDword::new(0x151),
                PlainDword::new(0x15C),
                PlainDword::new(0x15E),
                PlainDword::new(0x17A),
                PlainDword::new(0x181),
                PlainDword::new(0x1B2),
                PlainDword::new(0x200),
            ],
        }
    }
}

/// The wave tables' flag and constants.
#[derive(rc2k_derive::Cells)]
pub struct WaveState {
    /// 0, or the set mark once the tables are made.
    pub done: PlainDword,
    /// 256.0, what a level is divided by.
    pub scale: PlainFloat,
    /// 1.0, taken off the quotient.
    pub less: PlainFloat,
    /// Pi / 128, the sine table's step.
    pub sine_step: PlainFloat,
}

impl WaveState {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WaveState {
            done: PlainDword::new(0x0),
            scale: PlainFloat::new(256.0),
            less: PlainFloat::new(1.0),
            sine_step: PlainFloat::new(0.024543693),
        }
    }
}
