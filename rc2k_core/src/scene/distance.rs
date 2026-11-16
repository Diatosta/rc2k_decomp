//! A line's length near enough without a square root, and the stage objects' radius.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Approximate {
    /// The approximate length.
    pub length: u32,
    /// The middle part's share of it.
    pub middle: u32,
    /// The smallest part's share of it.
    pub least: u32,
}

/// Each difference over 256 (arithmetic shift) without its sign; the largest times 0xF2,
/// the middle times 0x60 and the smallest times 0x4B, added, wrapping. The parts are ordered
/// by three unsigned compares, each swapping unless "above".
pub fn approximate(differences: [i32; 3]) -> Approximate {
    let [mut a, mut b, mut c] = differences.map(|d| (d >> 8).wrapping_abs() as u32);
    if a <= b {
        std::mem::swap(&mut a, &mut b);
    }
    if b <= c {
        std::mem::swap(&mut b, &mut c);
    }
    if a <= b {
        std::mem::swap(&mut a, &mut b);
    }
    let (middle, least) = (b.wrapping_mul(0x60), c.wrapping_mul(0x4B));
    Approximate {
        length: a
            .wrapping_mul(0xF2)
            .wrapping_add(middle)
            .wrapping_add(least),
        middle,
        least,
    }
}

/// The root of x^2 + z^2 rounded to a whole number (`f64` steps; below 2^50 the root is at
/// least 2^-28 from a half, so the result is the same at every precision).
pub fn radius_distance(x: i32, z: i32) -> i32 {
    let (x, z) = (x as f64, z as f64);
    crate::math::geometry::fistp64((x * x + z * z).sqrt())
}

/// The radius after a point at `distance`: raised to it when the distance less the
/// radius is not negative (signed, wrapping).
pub fn widened(distance: u32, radius: u32) -> Option<u32> {
    ((distance.wrapping_sub(radius) as i32) >= 0).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths() {
        assert_eq!(
            approximate([0x300, -0x100, 0x200]),
            Approximate {
                length: 3 * 0xF2 + 2 * 0x60 + 0x4B,
                middle: 2 * 0x60,
                least: 0x4B
            }
        );
        assert_eq!(approximate([0xFF, 0x80, 0]).length, 0);
        assert_eq!(approximate([0, -0xFF, 0]).length, 0xF2);
        // -1 >> 8 is -1: its opposite is 1.
        assert_eq!(approximate([-1, 0, 0]).length, 0xF2);
        // i32::MIN >> 8 has an opposite.
        assert_eq!(
            approximate([i32::MIN, 0, 0]).length,
            0x80_0000u32.wrapping_mul(0xF2)
        );
    }

    #[test]
    fn radius() {
        assert_eq!(radius_distance(3, 4), 5);
        assert_eq!(radius_distance(-3, 4), 5);
        assert_eq!(radius_distance(1, 1), 1);
        assert_eq!(radius_distance(0x7FFF, 0x7FFF), 46340);
        assert_eq!(widened(5, 5), Some(5));
        assert_eq!(widened(5, 7), None);
        assert_eq!(widened(5, 0x8000_0000), None);
    }
}
