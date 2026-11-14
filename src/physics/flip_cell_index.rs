//! Turns a 2 x 2 cell (two coordinates, each 0 or 1) one of eight ways (way & 7: 1 flips the
//! first, 2 flips the second, 3 both; 4 .. 7 swap the two first, then 4 flips the second, 5
//! both, 6 none, 7 the first); the index is 2 x second + first.
//!
//! Caller: `physics::wheel_surfaces`, twice.

/// Returns the way's result, the first and the second coordinate.
// Core: `rc2k_core::physics::ground::flip_cell_index`.
pub use rc2k_core::physics::ground::flip_cell_index;
