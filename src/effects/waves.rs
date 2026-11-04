//! The marks' wave tables, out of the marks' block (phase 7, typed state): `MarksSines`
//! (IDA 0x92001E .. 0x92051E), `WaveLevels` (0x9205F6 .. 0x92063A: the lens flare's parts'
//! levels) and `WaveState` (0x9206C2 .. 0x9206D2: the made-once flag and constants).

use crate::session::{Dword, Float};

/// The marks' sine table (IDA 0x92001E .. 0x92051E, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksSines {
    /// 0x92001E: `g_marksSines`: 0x140 sines of 0, step, 2 step .. (`effects::make_wave_tables`; nothing ported reads them).
    pub table: [Float; 320],
}

pub static SINES: MarksSines = MarksSines {
    table: [const { Float::from_bits(0x0) }; 320],
};

impl MarksSines {
    pub const START: usize = 0x92001E;
    pub const LEN: usize = 0x500;
}

/// The wave levels (IDA 0x9205F6 .. 0x92063A, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WaveLevels {
    /// 0x9205F6: `g_marksWaveLevels`: the lens flare's parts' places along the line: whole numbers at first, floats once `effects::make_wave_tables` has run (as bits).
    pub levels: [Dword; 17],
}

pub static WAVE_LEVELS: WaveLevels = WaveLevels {
    levels: [
        Dword::new(0x0),
        Dword::new(0x9),
        Dword::new(0x2B),
        Dword::new(0x62),
        Dword::new(0xB4),
        Dword::new(0xBF),
        Dword::new(0xC6),
        Dword::new(0xE7),
        Dword::new(0x109),
        Dword::new(0x139),
        Dword::new(0x151),
        Dword::new(0x15C),
        Dword::new(0x15E),
        Dword::new(0x17A),
        Dword::new(0x181),
        Dword::new(0x1B2),
        Dword::new(0x200),
    ],
};

impl WaveLevels {
    pub const START: usize = 0x9205F6;
    pub const LEN: usize = 0x44;
}

/// The wave tables' flag and constants (IDA 0x9206C2 .. 0x9206D2, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WaveState {
    /// 0x9206C2: `g_marksWaveDone`: 0, or `nullsub_23`'s address once the tables are made.
    pub done: Dword,
    /// 0x9206C6: `flt_9206C6`: 256.0, what a level is divided by.
    pub scale: Float,
    /// 0x9206CA: `flt_9206CA`: 1.0, taken off the quotient.
    pub less: Float,
    /// 0x9206CE: `flt_9206CE`: pi / 128, the sine table's step.
    pub sine_step: Float,
}

pub static WAVE: WaveState = WaveState {
    done: Dword::new(0x0),
    scale: Float::from_bits(0x43800000),
    less: Float::from_bits(0x3F800000),
    sine_step: Float::from_bits(0x3CC90FDB),
};

impl WaveState {
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
        let image = crate::moved::image_bytes;
        assert!(
            bytes(&SINES) == image(MarksSines::START, MarksSines::LEN),
            "SINES starts differently from the image"
        );
        assert!(
            bytes(&WAVE_LEVELS) == image(WaveLevels::START, WaveLevels::LEN),
            "WAVE_LEVELS starts differently from the image"
        );
        assert!(
            bytes(&WAVE) == image(WaveState::START, WaveState::LEN),
            "WAVE starts differently from the image"
        );
    }
}
