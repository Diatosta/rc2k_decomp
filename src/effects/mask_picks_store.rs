//! The masks' picks and pixel tables.

use crate::memory::session::{PlainBytes, PlainDword};

/// The masks' shape picks and pixel tables.
#[derive(rc2k_derive::Cells)]
pub struct MaskPicks {
    /// 2 or 0 by the shape picked (A, B)
    pub flags: [PlainDword; 2],
    /// The shape index / the clock of the steps
    pub index: PlainDword,
    /// The shape entries
    pub table: [PlainDword; 256],
    /// The steps' values
    pub values: [PlainDword; 256],
    /// The shape whose mask is being drawn
    pub shapes: PlainDword,
    /// The mask being drawn (its shape's number)
    pub mask: PlainDword,
    /// The pixel table `map_shapes` was given (its shape's number)
    pub table_in_use: PlainDword,
    /// The corner table `map_shapes` was given (its shape's number)
    pub corners: PlainDword,
    /// The shapes move
    pub moving: PlainDword,
    /// The clock at which to choose again
    pub next: PlainDword,
    /// 9 masks (1, 3, 7 .. ) by the settings' `mask_number`
    pub masks: [PlainDword; 9],
    /// 0x80 rows of 0x400
    pub pixel_table_a: PlainBytes<131072>,
    /// The second pixel table
    pub pixel_table_b: PlainBytes<131072>,
}

/// The starting values of `table`.
#[rustfmt::skip]
const TABLE: [u32; 256] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 0xFFFFFF81, 0xFFFFFF82, 0xFFFFFF83,
    0xFFFFFF84, 0xFFFFFF85, 0xFFFFFF86, 0xFFFFFF87, 0xFFFFFF88, 0xFFFFFF89, 0xFFFFFF8A, 0xFFFFFF8B,
    0xFFFFFF8C, 0xFFFFFF8D, 0xFFFFFF8E, 0xFFFFFF8F, 0xFFFFFF90, 0xFFFFFF91, 0xFFFFFF92, 0xFFFFFF93,
    0xFFFFFF94, 0xFFFFFF95, 0xFFFFFF96, 0xFFFFFF97, 0xFFFFFF98, 0xFFFFFF99, 0xFFFFFF9A, 0xFFFFFF9B,
    0xFFFFFF9C, 0xFFFFFF9D, 0xFFFFFF9E, 0xFFFFFF9F, 0xFFFFFFA0, 0xFFFFFFA1, 0xFFFFFFA2, 0xFFFFFFA3,
    0xFFFFFFA4, 0xFFFFFFA5, 0xFFFFFFA6, 0xFFFFFFA7, 0xFFFFFFA8, 0xFFFFFFA9, 0xFFFFFFAA, 0xFFFFFFAB,
    0xFFFFFFAC, 0xFFFFFFAD, 0xFFFFFFAE, 0xFFFFFFAF, 0xFFFFFFB0, 0xFFFFFFB1, 0xFFFFFFB2, 0xFFFFFFB3,
    0xFFFFFFB4, 0xFFFFFFB5, 0xFFFFFFB6, 0xFFFFFFB7, 0xFFFFFFB8, 0xFFFFFFB9, 0xFFFFFFBA, 0xFFFFFFBB,
    0xFFFFFFBC, 0xFFFFFFBD, 0xFFFFFFBE, 0xFFFFFFBF, 0xFFFFFFC0, 0xFFFFFFC1, 0xFFFFFFC2, 0xFFFFFFC3,
    0xFFFFFFC4, 0xFFFFFFC5, 0xFFFFFFC6, 0xFFFFFFC7, 0xFFFFFFC8, 0xFFFFFFC9, 0xFFFFFFCA, 0xFFFFFFCB,
    0xFFFFFFCC, 0xFFFFFFCD, 0xFFFFFFCE, 0xFFFFFFCF, 0xFFFFFFD0, 0xFFFFFFD1, 0xFFFFFFD2, 0xFFFFFFD3,
    0xFFFFFFD4, 0xFFFFFFD5, 0xFFFFFFD6, 0xFFFFFFD7, 0xFFFFFFD8, 0xFFFFFFD9, 0xFFFFFFDA, 0xFFFFFFDB,
    0xFFFFFFDC, 0xFFFFFFDD, 0xFFFFFFDE, 0xFFFFFFDF, 0xFFFFFFE0, 0xFFFFFFE1, 0xFFFFFFE2, 0xFFFFFFE3,
    0xFFFFFFE4, 0xFFFFFFE5, 0xFFFFFFE6, 0xFFFFFFE7, 0xFFFFFFE8, 0xFFFFFFE9, 0xFFFFFFEA, 0xFFFFFFEB,
    0xFFFFFFEC, 0xFFFFFFED, 0xFFFFFFEE, 0xFFFFFFEF, 0xFFFFFFF0, 0xFFFFFFF1, 0xFFFFFFF2, 0xFFFFFFF3,
    0xFFFFFFF4, 0xFFFFFFF5, 0xFFFFFFF6, 0xFFFFFFF7, 0xFFFFFFF8, 0xFFFFFFF9, 0xFFFFFFFA, 0xFFFFFFFB,
    0xFFFFFFFC, 0xFFFFFFFD, 0xFFFFFFFE, 0xFFFFFFFF, 0,
];

/// The starting values of `values`.
#[rustfmt::skip]
const VALUES: [u32; 256] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 127, 126, 125, 124, 123, 122, 121, 120,
    119, 118, 117, 116, 115, 114, 113, 112, 111, 110, 109, 108, 107, 106, 105, 104, 103, 102, 101,
    100, 99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88, 87, 86, 85, 84, 83, 82, 81, 80, 79, 78, 77,
    76, 75, 74, 73, 72, 71, 70, 69, 68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53,
    52, 51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29,
    28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3,
    2, 1, 0,
];

/// The starting values of `masks`.
#[rustfmt::skip]
const MASKS: [u32; 9] = [
    255, 127, 63, 31, 15, 7, 3, 1, 0,
];

impl MaskPicks {
    pub const fn new() -> Self {
        MaskPicks {
            flags: [const { PlainDword::new(0x0) }; 2],
            index: PlainDword::new(0x0),
            table: PlainDword::array(TABLE),
            values: PlainDword::array(VALUES),
            shapes: PlainDword::new(0x0),
            mask: PlainDword::new(0x0),
            table_in_use: PlainDword::new(0x0),
            corners: PlainDword::new(0x0),
            moving: PlainDword::new(0x1),
            next: PlainDword::new(0x0),
            masks: PlainDword::array(MASKS),
            pixel_table_a: PlainBytes::new([0; 131072]),
            pixel_table_b: PlainBytes::new([0; 131072]),
        }
    }
}

#[cfg(test)]
mod image_bytes_tests {
    use super::*;

    /// The record's whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_mask_picks_keep_the_bytes_of_the_image() {
        let bytes: Vec<u8> = crate::memory::cells::flatten(&MaskPicks::new());
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 264264);
        assert_eq!(hash, 0x4530_ac9f_7b42_88ed);
    }
}
