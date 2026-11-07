//! `physics::flip_cell_index` (0x43DB13): a 2 x 2 cell (ebx, edx, each 0 or 1) turned by
//! one of eight ways (eax & 7: 1 flips ebx, 2 flips edx, 3 both; 4 .. 7 swap the two
//! first, then 4 flips edx, 5 both, 6 none, 7 ebx), then edx = 2 edx + ebx.
//!
//! Takes: the way, and the cell
//!
//! Caller: `physics::wheel_surfaces` (twice).

/// The Rust version: (eax, ebx, edx) out.
// Converted: `rc2k_core::physics::ground::flip_cell_index`.
pub use rc2k_core::physics::ground::flip_cell_index;
