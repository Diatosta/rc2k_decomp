//! The constants and randoms of `make_masks` and the shapes' runs (IDA 0x89C994 .. 0x89CA30), out
//! of the marks' block (phase 7, typed state): the spans' and rings' constants, the eased
//! steps', the corners', the pixels' and the rows'.

use crate::session::{Dword, Float};

/// The constants and randoms of `make_masks` and the shapes' runs (IDA 0x89C994 .. 0x89CA30, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MaskConstants {
    /// 0x89C994: `dword_89C994`: the random (times 0xC5 plus 0x5D) of the moving time (`effects::run_shapes`).
    pub time_random: Dword,
    /// 0x89C998: `dword_89C998`: the random of whether the shapes move.
    pub move_random: Dword,
    /// 0x89C99C: `flt_89C99C`, `flt_89C9A0`: the frame ticks' rate (11360.25 each) when the shapes move (`effects::run_shapes`).
    pub rates: [Float; 2],
    /// 0x89C9A4: `flt_89C9A4` .. `flt_89C9AC`: 1.0 each, the rows' step (`effects::span_edges`, the third).
    pub ones: [Float; 3],
    /// 0x89C9B0: `flt_89C9B0` .. `flt_89C9B8`: 0.0175, 0.0175 and 0.5 (`effects::span_ring`).
    pub ring_scales: [Float; 3],
    /// 0x89C9BC: `flt_89C9BC`: 0.5, added to the row (`effects::map_shape_rows`).
    pub row_add: Float,
    /// 0x89C9C0: `flt_89C9C0`: 0.5, the eased steps' factor (`effects::smooth_steps`).
    pub ease: Float,
    /// 0x89C9C4: `dword_89C9C4`, `dword_89C9C8`: 0x80 each, the eased steps' counts (`effects::smooth_steps`).
    pub steps: [Dword; 2],
    /// 0x89C9CC: `dword_89C9CC` .. `dword_89C9E8`: the corner tables' turns, 0 and -pi / 2 (as bits) for each of two steps of each of two tables; the port reads the 0s (`effects::make_corners`).
    pub corner_angles: [Float; 8],
    /// 0x89C9EC: `dword_89C9EC`, `dword_89C9F0`: 0x100 and 0x80, a corner's scales' divisors (`effects::make_corners`).
    pub divisors: [Dword; 2],
    /// 0x89C9F4: `dword_89C9F4`, `dword_89C9F8`: 0x80, 0x80, what a corner's y is taken from (`effects::make_corners`).
    pub flips: [Dword; 2],
    /// 0x89C9FC: `flt_89C9FC` .. `flt_89CA08`: 256.0 each, what `effects::clamp_pair` holds a pair of corners to.
    pub pair_scales: [Float; 4],
    /// 0x89CA0C: `flt_89CA0C` .. `flt_89CA20`: two times three constants, 1.0, 1.0, 1.2 (`effects::map_pixel`, one set for each half).
    pub pixel_constants: [Float; 6],
    /// 0x89CA24: `flt_89CA24` .. `flt_89CA2C`: 1.0 each, the rows' lengths for `effects::scale_rows3`.
    pub row_lengths: [Float; 3],
}

impl MaskConstants {
    pub const fn new() -> Self {
        Self {
            time_random: Dword::new(0x0),
            move_random: Dword::new(0x0),
            rates: [Float::from_bits(0x46348400), Float::from_bits(0x46348400)],
            ones: [
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F800000),
            ],
            ring_scales: [
                Float::from_bits(0x3C8EFA35),
                Float::from_bits(0x3C8EFA35),
                Float::from_bits(0x3F000000),
            ],
            row_add: Float::from_bits(0x3F000000),
            ease: Float::from_bits(0x3F000000),
            steps: [Dword::new(0x80), Dword::new(0x80)],
            corner_angles: [
                Float::from_bits(0x0),
                Float::from_bits(0xBFC90FDB),
                Float::from_bits(0x0),
                Float::from_bits(0xBFC90FDB),
                Float::from_bits(0x0),
                Float::from_bits(0xBFC90FDB),
                Float::from_bits(0x0),
                Float::from_bits(0xBFC90FDB),
            ],
            divisors: [Dword::new(0x100), Dword::new(0x80)],
            flips: [Dword::new(0x80), Dword::new(0x80)],
            pair_scales: [
                Float::from_bits(0x43800000),
                Float::from_bits(0x43800000),
                Float::from_bits(0x43800000),
                Float::from_bits(0x43800000),
            ],
            pixel_constants: [
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F99999A),
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F99999A),
            ],
            row_lengths: [
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F800000),
                Float::from_bits(0x3F800000),
            ],
        }
    }
}

impl MaskConstants {
    pub const START: usize = 0x89C994;
    pub const LEN: usize = 0x9C;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(
            &raw const crate::effects::owner::effects().setup.mask_constants,
        ) as usize
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn mask_constants_starts_as_the_games() {
        assert_eq!(size_of::<MaskConstants>(), MaskConstants::LEN);
        let image = crate::moved::image_bytes(MaskConstants::START, MaskConstants::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(MaskConstants::home()).cast_const(),
                MaskConstants::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "MASK_CONSTANTS starts differently from the image"
        );
        assert_eq!(
            offset_of!(MaskConstants, rates),
            0x89C99C - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, ones),
            0x89C9A4 - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, ring_scales),
            0x89C9B0 - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, row_add),
            0x89C9BC - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, steps),
            0x89C9C4 - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, corner_angles),
            0x89C9CC - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, divisors),
            0x89C9EC - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, flips),
            0x89C9F4 - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, pair_scales),
            0x89C9FC - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, pixel_constants),
            0x89CA0C - MaskConstants::START
        );
        assert_eq!(
            offset_of!(MaskConstants, row_lengths),
            0x89CA24 - MaskConstants::START
        );
    }
}
