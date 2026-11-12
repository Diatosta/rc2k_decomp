//! The marks' wave tables: `MarksSines`
//! (IDA 0x92001E .. 0x92051E), `WaveLevels` (0x9205F6 .. 0x92063A: the lens flare's parts'
//! levels) and `WaveState` (0x9206C2 .. 0x9206D2: the made-once flag and constants).

use crate::session::{PlainDword, PlainFloat};

/// The marks' sine table (IDA 0x92001E .. 0x92051E).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksSines {
    /// 0x92001E: `g_marksSines`: 0x140 sines of 0, step, 2 step .. (`effects::make_wave_tables`; nothing ported reads them).
    pub table: [PlainFloat; 320],
}

impl MarksSines {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        MarksSines {
            table: [const { PlainFloat::new(0.0) }; 320],
        }
    }
}

/// The wave levels (IDA 0x9205F6 .. 0x92063A).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WaveLevels {
    /// 0x9205F6: `g_marksWaveLevels`: the lens flare's parts' places along the line: whole numbers at first, floats once `effects::make_wave_tables` has run (as bits).
    pub levels: [PlainDword; 17],
}

impl WaveLevels {
    /// The values the game starts with (the image's bytes).
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

/// The wave tables' flag and constants (IDA 0x9206C2 .. 0x9206D2).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WaveState {
    /// 0x9206C2: `g_marksWaveDone`: 0, or `nullsub_23`'s address once the tables are made.
    pub done: PlainDword,
    /// 0x9206C6: `flt_9206C6`: 256.0, what a level is divided by.
    pub scale: PlainFloat,
    /// 0x9206CA: `flt_9206CA`: 1.0, taken off the quotient.
    pub less: PlainFloat,
    /// 0x9206CE: `flt_9206CE`: pi / 128, the sine table's step.
    pub sine_step: PlainFloat,
}

impl WaveState {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WaveState {
            done: PlainDword::new(0x0),
            scale: PlainFloat::new(256.0),
            less: PlainFloat::new(1.0),
            sine_step: PlainFloat::new(0.024543693),
        }
    }
}
