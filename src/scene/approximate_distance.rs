//! `scene::approximate_distance`: how long a line is, near enough, without a square root.
//!
//! Each of the three differences is taken over 256 (signed) and without its sign; the largest
//! times 0xF2, the middle one times 0x60 and the smallest times 0x4B are added. Callers:
//! `scene::add_point_lights` and the vertex light.

/// Returns the length and the two leftovers. Logic: `rc2k_core::scene::distance::approximate`.
pub fn approximate_distance(differences: [u32; 3]) -> [u32; 3] {
    let d = rc2k_core::scene::distance::approximate(differences.map(|d| d as i32));
    [d.length, d.middle, d.least]
}
