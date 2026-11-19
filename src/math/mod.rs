//! Small vector helpers, the game's random number streams (`next_random`, `seed_random`,
//! `stir_random`), and the arithmetic that reproduces the x87 unit's rounding (`x87_float::X` and
//! the operations in `x87`), on which replays depend bit for bit.

pub mod cell_x87;
pub mod cells;
pub mod engine_tables;
pub mod nearest_on_segment;
pub mod next_random;
pub mod normalise_vector;
pub mod rotate_float_point_about_y;
pub mod rotate_point_about_y;
pub mod rotate_point_about_y_ebx;
pub mod rotate_point_about_y_ecx;
pub mod rotate_scratch_about_y;
pub mod seed_random;
pub mod stir_random;
pub mod x87;
pub(crate) mod x87_float;
#[cfg(test)]
mod x87_frozen_a;
#[cfg(test)]
mod x87_frozen_b;
#[cfg(test)]
mod x87_frozen_c;
pub(crate) mod x87_mem;
#[cfg(test)]
mod x87_merged_tests;
#[cfg(test)]
#[allow(
    dead_code,
    reason = "the comparison test calls only the helpers that changed"
)]
mod x87_old;
#[cfg(test)]
mod x87_results_tests;
#[cfg(test)]
mod x87_typed_tests;
