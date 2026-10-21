//! A line's length near enough, without a square root (converted 2026-10-21 from
//! `scene::approximate_distance` 0x44EB78): the lamps' and the race's distances.

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
}
