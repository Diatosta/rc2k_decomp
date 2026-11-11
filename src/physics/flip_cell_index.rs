//! `physics::flip_cell_index` (0x43DB13): a 2 x 2 cell (two coordinates, each 0 or 1) turned by
//! one of eight ways (way & 7: 1 flips the first, 2 flips the second, 3 both; 4 .. 7 swap the two
//! first, then 4 flips the second, 5 both, 6 none, 7 the first), then the index is 2 x second +
//! first.
//!
//! Takes: the way, and the cell
//!
//! Caller: `physics::wheel_surfaces` (twice).

/// Returns the way's result, the first and the second coordinate.
// Converted: `rc2k_core::physics::ground::flip_cell_index`.
pub use rc2k_core::physics::ground::flip_cell_index;
