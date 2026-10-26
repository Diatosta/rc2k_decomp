//! A line's length near enough, without a square root (converted 2026-10-21 from
//! `scene::approximate_distance` 0x44EB78): the lamps' and the race's distances; and the
//! stage objects' radius (from `scene::widen_object_radius` 0x426A7F), with one.

/// The length and the two smaller parts' shares of it (what the original leaves in ebx
/// and ecx).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Approximate {
    pub length: u32,
    pub middle: u32,
    pub least: u32,
}

/// Each difference over 256 (an arithmetic shift) without its sign (the one number with
/// no opposite stays as it is); the largest times 0xF2, the middle one times 0x60 and the
/// smallest times 0x4B, added, wrapping. The parts are put in order by the original's
/// three unsigned compares, each changing places unless "above".
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

/// `widen_object_radius`' distance: `fistp` of the root of x^2 + z^2 (each step as an
/// `f64`: the x87's at double precision; for x^2 + z^2 under 2^50 the root is at least
/// 2^-28 from a half, so the whole number is the same at every precision).
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
        // -1 >> 8 is -1: its opposite 1
        assert_eq!(approximate([-1, 0, 0]).length, 0xF2);
        // i32::MIN >> 8 has an opposite
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
