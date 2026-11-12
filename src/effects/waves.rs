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
    pub const START: usize = 0x92001E;
    pub const LEN: usize = 0x500;
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
    pub const START: usize = 0x9205F6;
    pub const LEN: usize = 0x44;
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
    pub const START: usize = 0x9206C2;
    pub const LEN: usize = 0x10;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_tables_start_as_the_games() {
        assert_eq!(size_of::<MarksSines>(), MarksSines::LEN);
        assert_eq!(size_of::<WaveLevels>(), WaveLevels::LEN);
        assert_eq!(size_of::<WaveState>(), WaveState::LEN);
        assert!(
            bytes(&crate::effects::owner::effects().paths.sines)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.paths.sines/1280"
                ),
            "SINES starts differently from the image"
        );
        assert!(
            bytes(&crate::effects::owner::effects().paths.wave_levels)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.paths.wave_levels/68"
                ),
            "WAVE_LEVELS starts differently from the image"
        );
        assert!(
            bytes(&crate::effects::owner::effects().paths.wave)
                == crate::test_support::first_values::bytes("effects.owner.EFFECTS.paths.wave/16"),
            "WAVE starts differently from the image"
        );
    }
}
