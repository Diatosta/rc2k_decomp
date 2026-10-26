//! `scene::approximate_distance` (0x44EB78): how long a line is, near enough, without a
//! square root.
//!
//! In:  eax, ebx, ecx = the three differences
//! Out: eax = the length; ebx and ecx = the two smaller parts' shares of it;
//!      everything else preserved
//!
//! Each difference over 256 (signed) and without its sign; the largest times 0xF2, the
//! middle one times 0x60 and the smallest times 0x4B, added.
//! Callers: `scene::add_point_lights` and 0x44F8C7.

// ---- Rust version ----

/// Returns the length and what is left in ebx and ecx. Converted:
/// `rc2k_core::scene::distance::approximate`.
pub fn approximate_distance(differences: [u32; 3]) -> [u32; 3] {
    let d = rc2k_core::scene::distance::approximate(differences.map(|d| d as i32));
    [d.length, d.middle, d.least]
}
