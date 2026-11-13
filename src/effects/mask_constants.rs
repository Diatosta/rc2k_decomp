//! The constants and randoms of `make_masks` and the shapes' runs (IDA 0x89C994 .. 0x89CA30), out
//! of the marks' block (phase 7, typed state): the spans' and rings' constants, the eased
//! steps', the corners', the pixels' and the rows'.

use crate::session::{PlainDword, PlainFloat};

/// The constants and randoms of `make_masks` and the shapes' runs (IDA 0x89C994 .. 0x89CA30).
#[derive(rc2k_derive::Cells)]
pub struct MaskConstants {
    /// 0x89C994: `dword_89C994`: the random (times 0xC5 plus 0x5D) of the moving time (`effects::run_shapes`).
    pub time_random: PlainDword,
    /// 0x89C998: `dword_89C998`: the random of whether the shapes move.
    pub move_random: PlainDword,
    /// 0x89C99C: `flt_89C99C`, `flt_89C9A0`: the frame ticks' rate (11360.25 each) when the shapes move (`effects::run_shapes`).
    pub rates: [PlainFloat; 2],
    /// 0x89C9A4: `flt_89C9A4` .. `flt_89C9AC`: 1.0 each, the rows' step (`effects::span_edges`, the third).
    pub ones: [PlainFloat; 3],
    /// 0x89C9B0: `flt_89C9B0` .. `flt_89C9B8`: 0.0175, 0.0175 and 0.5 (`effects::span_ring`).
    pub ring_scales: [PlainFloat; 3],
    /// 0x89C9BC: `flt_89C9BC`: 0.5, added to the row (`effects::map_shape_rows`).
    pub row_add: PlainFloat,
    /// 0x89C9C0: `flt_89C9C0`: 0.5, the eased steps' factor (`effects::smooth_steps`).
    pub ease: PlainFloat,
    /// 0x89C9C4: `dword_89C9C4`, `dword_89C9C8`: 0x80 each, the eased steps' counts (`effects::smooth_steps`).
    pub steps: [PlainDword; 2],
    /// 0x89C9CC: `dword_89C9CC` .. `dword_89C9E8`: the corner tables' turns, 0 and -pi / 2 (as bits) for each of two steps of each of two tables; the port reads the 0s (`effects::make_corners`).
    pub corner_angles: [PlainFloat; 8],
    /// 0x89C9EC: `dword_89C9EC`, `dword_89C9F0`: 0x100 and 0x80, a corner's scales' divisors (`effects::make_corners`).
    pub divisors: [PlainDword; 2],
    /// 0x89C9F4: `dword_89C9F4`, `dword_89C9F8`: 0x80, 0x80, what a corner's y is taken from (`effects::make_corners`).
    pub flips: [PlainDword; 2],
    /// 0x89C9FC: `flt_89C9FC` .. `flt_89CA08`: 256.0 each, what `effects::clamp_pair` holds a pair of corners to.
    pub pair_scales: [PlainFloat; 4],
    /// 0x89CA0C: `flt_89CA0C` .. `flt_89CA20`: two times three constants, 1.0, 1.0, 1.2 (`effects::map_pixel`, one set for each half).
    pub pixel_constants: [PlainFloat; 6],
    /// 0x89CA24: `flt_89CA24` .. `flt_89CA2C`: 1.0 each, the rows' lengths for `effects::scale_rows3`.
    pub row_lengths: [PlainFloat; 3],
}

impl MaskConstants {
    pub const fn new() -> Self {
        Self {
            time_random: PlainDword::new(0x0),
            move_random: PlainDword::new(0x0),
            rates: [PlainFloat::new(11553.0), PlainFloat::new(11553.0)],
            ones: [
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
            ],
            ring_scales: [
                PlainFloat::new(0.017453292),
                PlainFloat::new(0.017453292),
                PlainFloat::new(0.5),
            ],
            row_add: PlainFloat::new(0.5),
            ease: PlainFloat::new(0.5),
            steps: [PlainDword::new(0x80), PlainDword::new(0x80)],
            corner_angles: [
                PlainFloat::new(0.0),
                PlainFloat::new(-1.5707964),
                PlainFloat::new(0.0),
                PlainFloat::new(-1.5707964),
                PlainFloat::new(0.0),
                PlainFloat::new(-1.5707964),
                PlainFloat::new(0.0),
                PlainFloat::new(-1.5707964),
            ],
            divisors: [PlainDword::new(0x100), PlainDword::new(0x80)],
            flips: [PlainDword::new(0x80), PlainDword::new(0x80)],
            pair_scales: [
                PlainFloat::new(256.0),
                PlainFloat::new(256.0),
                PlainFloat::new(256.0),
                PlainFloat::new(256.0),
            ],
            pixel_constants: [
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.2),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.2),
            ],
            row_lengths: [
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
            ],
        }
    }
}
