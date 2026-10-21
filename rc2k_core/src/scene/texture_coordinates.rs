//! A polygon's texture coordinates from whole numbers to floats (converted 2026-10-21
//! from `scene::scale_texture_coordinates` 0x44E2FB): a value that is not 0 becomes a
//! float (`fild`, `fstp`: one rounding to a float, the same at every precision) whose bits
//! then lose 0x0C000000, the number over 2^24.

/// The float's bits less this: its exponent less 24.
pub const OVER_2_24: u32 = 0x0C00_0000;

/// A coordinate's float (the work variable `flt_5183C4`), `None` for 0 (left as it is).
pub fn float_of(v: u32) -> Option<u32> {
    (v != 0).then(|| (v as i32 as f32).to_bits())
}

/// A coordinate scaled: 0 stays 0, else its float's bits less `OVER_2_24`.
pub fn scaled(v: u32) -> u32 {
    float_of(v).map_or(0, |f| f.wrapping_sub(OVER_2_24))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinates() {
        assert_eq!(scaled(0), 0);
        assert_eq!(f32::from_bits(scaled(0x80_0000)), 0.5);
        assert_eq!(f32::from_bits(scaled(-0x100_0000i32 as u32)), -1.0);
        assert_eq!(float_of(0x0100_0001), Some(16777216f32.to_bits()));
        // A small number's float less the exponent wraps (as the original's `sub`).
        assert_eq!(scaled(1), 0x3F80_0000 - OVER_2_24);
    }
}
