//! `effects::tilt_shape` (0x4C1194): the sine and cosine of `g_marksShapeTilt` into
//! `flt_811BC7` / `flt_811BCB`; the point's y and z (`flt_811BE3`, `flt_811BE7`) turned by
//! them (`x87::turn_yz`); then the point `flt_811BDF` .. made of length 1 into
//! `unk_811BD3` .., its length in `dword_811BCF` (`x87::unit_and_length`).
//!
//! Caller: `effects::shapes_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::math::x87;
use crate::session::{PlainDword, PlainFloat, read_bits, write_bits};

pub fn tilt_shape(drops: &WindscreenDrops) {
    let t = &drops.tilt;
    let (s, c) = x87::sine_and_cosine(&t.tilt);
    t.cosine.set(c);
    t.sine.set(s);
    let (y, z) = x87::turn_yz(&t.point[1], &t.point[2], s, c);
    t.point[1].set(y);
    t.point[2].set(z);
    let (unit, length) = x87::unit_and_length(read_bits(&t.point));
    t.length.set_bits(length.to_bits());
    write_bits(&t.unit, unit.map(f32::to_bits));
}

/// The shape tilt and the first view point (`effects::tilt_shape`, `effects::copy_first_point`, `effects::edges`) (IDA 0x811BC3 .. 0x811BEB).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShapeTilt {
    /// 0x811BC3: `g_marksShapeTilt`: the kind's float times 0.0175.
    pub tilt: PlainFloat,
    /// 0x811BC7: `flt_811BC7`: the sine of the tilt.
    pub sine: PlainFloat,
    /// 0x811BCB: `flt_811BCB`: the cosine of the tilt.
    pub cosine: PlainFloat,
    /// 0x811BCF: `dword_811BCF`: the point's length.
    pub length: PlainDword,
    /// 0x811BD3: `unk_811BD3` ..: the point made of length 1.
    pub unit: [PlainFloat; 3],
    /// 0x811BDF: `flt_811BDF` .. `flt_811BE7`: the first kept view point (copied by `effects::copy_first_point`, turned by the tilt, then also set to 0 by `effects::windscreen_drops_drift` and moved by the drift).
    pub point: [PlainFloat; 3],
}

impl ShapeTilt {
    pub const fn new() -> Self {
        ShapeTilt {
            tilt: PlainFloat::new(0.0),
            sine: PlainFloat::new(0.0),
            cosine: PlainFloat::new(0.0),
            length: PlainDword::new(0x0),
            unit: [
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
            ],
            point: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
        }
    }
}

impl ShapeTilt {
    pub const START: usize = 0x811BC3;
    pub const LEN: usize = 0x28;
}

#[cfg(test)]
mod shape_tilt_tests {
    use super::*;

    #[test]
    fn shape_tilt_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<ShapeTilt>(), ShapeTilt::LEN);
        // The x87 tests put their buffer's bytes here for a while (`typed_sync::with_overlays`).
        let _lock = crate::store::test_lock();
        let p = (&raw const crate::effects::owner::effects().windscreen_drops.tilt).cast::<u8>();
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
