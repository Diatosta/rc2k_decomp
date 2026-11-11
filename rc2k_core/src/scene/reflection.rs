//! A shiny surface's reflection (from `scene::reflect_vector`
//! 0x451A9D). As in `math::vector`, what the x87 keeps on its stack is an `f64`, rounded
//! to `f32` where the original stores it: the same bits at the game's double precision.

use crate::math::vector;

/// The places of `scene::cached_reflection_coordinates`' tables (from
/// 0x451908): four tables of 0x400 dwords (the normal's number, the point's, the two
/// coordinates) at the normal's number & 0x3FF.
pub const CACHE_PLACES: u32 = 0x400;

/// A normal's place in the tables.
pub fn cache_place(normal: u32) -> usize {
    (normal & (CACHE_PLACES - 1)) as usize
}

/// The coordinates kept for the two numbers asked (normal, point), if both numbers kept
/// at their place are those; `kept` read only then.
pub fn cached(
    asked: [u32; 2],
    numbers: [u32; 2],
    kept: impl FnOnce() -> [u32; 2],
) -> Option<[u32; 2]> {
    (numbers == asked).then(kept)
}

/// `scene::reflect_vector`: d = v . n (the first two products added first, then the
/// third; kept unrounded); v times d, each stored; v + v - n, each stored; that through
/// the matrix `m` (the game's 4 x 4 layout as `vector::rotate_point` takes it), stored.
/// Returns the vector and d as stored.
pub fn reflect(v: [f32; 3], n: [f32; 3], m: &[f32; 15]) -> ([f32; 3], f32) {
    let (w, k) = (v.map(f64::from), n.map(f64::from));
    let d = (w[1] * k[1] + w[0] * k[0]) + w[2] * k[2];
    let scaled = w.map(|x| (x * d) as f32);
    let mirrored: [f64; 3] = std::array::from_fn(|i| {
        let s = scaled[i] as f64;
        ((s + s - k[i]) as f32) as f64
    });
    let turned = vector::rotate_point(m, mirrored);
    (turned.map(|x| x as f32), d as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache() {
        assert_eq!(cache_place(0x1234), 0x234);
        assert_eq!(cached([1, 2], [1, 2], || [7, 8]), Some([7, 8]));
        assert_eq!(cached([1, 2], [1, 3], || unreachable!()), None);
    }

    #[test]
    fn mirrored_and_turned() {
        let mut m = [0.0f32; 15];
        (m[0], m[5], m[10]) = (1.0, 1.0, 1.0);
        // (0, 0, 1) at the normal (0, 0, 1): d 1, 2 v - n = (0, 0, 1).
        assert_eq!(
            reflect([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], &m),
            ([0.0, 0.0, 1.0], 1.0)
        );
        // A matrix's place is added.
        m[12..15].copy_from_slice(&[1.0, 2.0, 3.0]);
        let (r, d) = reflect([0.6, -0.8, 0.0], [0.0, 1.0, 0.0], &m);
        assert_eq!(d, -0.8);
        let s = |x: f32| (x as f64 * -0.8f32 as f64) as f32;
        assert_eq!(r[0], ((s(0.6) as f64 * 2.0) as f32 + 1.0));
        assert_eq!(r[1], ((s(-0.8) as f64 * 2.0 - 1.0) as f32 + 2.0));
    }
}
