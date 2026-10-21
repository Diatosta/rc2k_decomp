//! The depth a face is sorted by (converted 2026-10-21 from `scene::face_sort_key`
//! 0x450564): its points' depths (each point's record 0x1000 + its number's low 12 bits,
//! +0x18) added one by one, each sum stored as a float, times one over the vertices
//! (`flt_5C6950`), rounded to a whole number. As in `math::geometry`, what the x87 keeps on
//! its stack is an `f64`: the same at the game's double precision.

use crate::Invalid;
use crate::math::geometry::fistp64;

/// The factors' table (`flt_5C6950`): 0x100 floats.
pub const FACTORS: u32 = 0x100;
/// The most points a face's key is made from (the records' second half).
pub const MOST_POINTS: u32 = 0x1000;

/// Refuses no face, no records, a count of 0 (the original goes through 2^32) or past the
/// records, and a vertex count past the factors.
pub fn check(face: bool, records: bool, count: u32, vertices: u32) -> Result<(), Invalid> {
    if !face || !records || count == 0 || count > MOST_POINTS || vertices >= FACTORS {
        return Err(Invalid(
            "no face, no records, or a count the key cannot be made for",
        ));
    }
    Ok(())
}

/// A point's record's offset (0x20 bytes a record).
pub fn record_offset(point: u32) -> u32 {
    ((point & 0xFFF) + 0x1000) << 5
}

/// The key: the depths summed (`fld depth; fadd sum; fstp sum`, from 0), times the factor
/// (`fistp`).
pub fn key(depths: impl IntoIterator<Item = f32>, factor: f32) -> i32 {
    let sum = depths
        .into_iter()
        .fold(0.0f32, |sum, d| (d as f64 + sum as f64) as f32);
    fistp64(sum as f64 * factor as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        assert!(check(true, true, 0, 1).is_err());
        assert!(check(true, true, 1, 0x100).is_err());
        assert!(check(false, true, 1, 1).is_err());
        assert!(check(true, true, 0x1000, 0xFF).is_ok());
        assert_eq!(record_offset(0x7031), 0x1031 << 5);
        assert_eq!(key([9000.0, 4096.5, 1.0], 1.0 / 3.0), 4366);
        assert_eq!(key([16777216.0, 1.0], 1.0), 16777216);
        assert_eq!(key([], 0.5), 0);
    }
}
