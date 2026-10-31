//! `effects::tilt_shape` (0x4C1194): the sine and cosine of `g_marksShapeTilt` into
//! `flt_811BC7` / `flt_811BCB`; the point's y and z (`flt_811BE3`, `flt_811BE7`) turned by
//! them (`x87::turn_yz`); then the point `flt_811BDF` .. made of length 1 into
//! `unk_811BD3` .., its length in `dword_811BCF` (`x87::unit_and_length`).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1148`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::math::x87;
use crate::session::{Dword, Float, read_bits, write_bits};

/// The Rust version (see the header).
pub fn tilt_shape() {
    let t = &SHAPE_TILT;
    let (s, c) = x87::sine_and_cosine(t.tilt.bits());
    t.cosine.set_bits(c);
    t.sine.set_bits(s);
    let (y, z) = x87::turn_yz(t.point[1].bits(), t.point[2].bits(), s, c);
    t.point[1].set_bits(y);
    t.point[2].set_bits(z);
    let (unit, length) = x87::unit_and_length(read_bits(&t.point));
    t.length.set_bits(length);
    write_bits(&t.unit, unit);
}

/// The shape tilt and the first view point (`effects::tilt_shape`, `effects::copy_first_point`, `effects::edges`) (IDA 0x811BC3 .. 0x811BEB, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct ShapeTilt {
    /// 0x811BC3: `g_marksShapeTilt`: the kind's float times 0.0175.
    pub tilt: Float,
    /// 0x811BC7: `flt_811BC7`: the sine of the tilt.
    pub sine: Float,
    /// 0x811BCB: `flt_811BCB`: the cosine of the tilt.
    pub cosine: Float,
    /// 0x811BCF: `dword_811BCF`: the point's length.
    pub length: Dword,
    /// 0x811BD3: `unk_811BD3` ..: the point made of length 1.
    pub unit: [Float; 3],
    /// 0x811BDF: `flt_811BDF` .. `flt_811BE7`: the first kept view point (copied by `effects::copy_first_point`, turned by the tilt, then also set to 0 by `effects::windscreen_drops_drift` and moved by the drift).
    pub point: [Float; 3],
}

pub static SHAPE_TILT: ShapeTilt = ShapeTilt {
    tilt: Float::from_bits(0x0),
    sine: Float::from_bits(0x0),
    cosine: Float::from_bits(0x0),
    length: Dword::new(0x0),
    unit: [
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
    ],
    point: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
};

impl ShapeTilt {
    pub const START: usize = 0x811BC3;
    pub const LEN: usize = 0x28;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const SHAPE_TILT) as usize
    }
}

#[cfg(test)]
mod shape_tilt_tests {
    use super::*;

    #[test]
    fn shape_tilt_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<ShapeTilt>(), ShapeTilt::LEN);
        let p = (&raw const SHAPE_TILT).cast::<u8>();
        let ours: Vec<u8> = (0..size_of::<ShapeTilt>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(ShapeTilt::START, ShapeTilt::LEN),
            "SHAPE_TILT starts differently from the image"
        );
        assert_eq!(offset_of!(ShapeTilt, tilt), 0x811BC3 - ShapeTilt::START);
        assert_eq!(offset_of!(ShapeTilt, sine), 0x811BC7 - ShapeTilt::START);
        assert_eq!(offset_of!(ShapeTilt, cosine), 0x811BCB - ShapeTilt::START);
        assert_eq!(offset_of!(ShapeTilt, length), 0x811BCF - ShapeTilt::START);
        assert_eq!(offset_of!(ShapeTilt, unit), 0x811BD3 - ShapeTilt::START);
        assert_eq!(offset_of!(ShapeTilt, point), 0x811BDF - ShapeTilt::START);
    }
}
