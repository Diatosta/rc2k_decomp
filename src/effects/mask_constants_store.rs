//! The constants and randoms of `make_masks` and the shapes' runs: the spans' and rings'
//! constants, the eased steps', the corners', the pixels' and the rows'.

/// The constants and randoms of `make_masks` and the shapes' runs.
pub struct MaskConstants {
    /// The random (times 0xC5 plus 0x5D) of the moving time (`effects::run_shapes`).
    pub time_random: u32,
    /// The random of whether the shapes move.
    pub move_random: u32,
    /// The frame ticks' rate (11360.25 each) when the shapes move (`effects::run_shapes`).
    pub rates: [f32; 2],
    /// 1.0 each, the rows' step (`effects::span_edges`, the third).
    pub ones: [f32; 3],
    /// 0.0175, 0.0175 and 0.5 (`effects::span_ring`).
    pub ring_scales: [f32; 3],
    /// 0.5, added to the row (`effects::map_shape_rows`).
    pub row_add: f32,
    /// 0.5, the eased steps' factor (`effects::smooth_steps`).
    pub ease: f32,
    /// 0x80 each, the eased steps' counts (`effects::smooth_steps`).
    pub steps: [u32; 2],
    /// The corner tables' turns, 0 and -pi / 2 (as bits) for each of two steps of each of two
    /// tables; the 0s are read (`effects::make_corners`).
    pub corner_angles: [f32; 8],
    /// 0x100 and 0x80, a corner's scales' divisors (`effects::make_corners`).
    pub divisors: [u32; 2],
    /// 0x80, 0x80, what a corner's y is taken from (`effects::make_corners`).
    pub flips: [u32; 2],
    /// 256.0 each, what `effects::clamp_pair` holds a pair of corners to.
    pub pair_scales: [f32; 4],
    /// Two times three constants, 1.0, 1.0, 1.2 (`effects::map_pixel`, one set for each half).
    pub pixel_constants: [f32; 6],
    /// 1.0 each, the rows' lengths for `effects::scale_rows3`.
    pub row_lengths: [f32; 3],
}

impl MaskConstants {
    pub const fn new() -> Self {
        Self {
            time_random: 0x0,
            move_random: 0x0,
            rates: [11553.0, 11553.0],
            ones: [1.0, 1.0, 1.0],
            ring_scales: [0.017453292, 0.017453292, 0.5],
            row_add: 0.5,
            ease: 0.5,
            steps: [0x80, 0x80],
            corner_angles: [
                0.0, -1.5707964, 0.0, -1.5707964, 0.0, -1.5707964, 0.0, -1.5707964,
            ],
            divisors: [0x100, 0x80],
            flips: [0x80, 0x80],
            pair_scales: [256.0, 256.0, 256.0, 256.0],
            pixel_constants: [1.0, 1.0, 1.2, 1.0, 1.0, 1.2],
            row_lengths: [1.0, 1.0, 1.0],
        }
    }
}
