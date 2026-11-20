//! Small vector helpers, the game's random number streams (`next_random`, `seed_random`,
//! `stir_random`), and the arithmetic that reproduces the x87 unit's rounding (`x87_float::X` and
//! the operations in `x87`), on which replays depend bit for bit.

pub(crate) mod cell_x87;
pub(crate) mod cells;
pub(crate) mod check_hash;
pub(crate) mod engine_tables;
pub(crate) mod matrix3;
pub(crate) mod nearest_on_segment;
pub(crate) mod next_random;
pub(crate) mod normalise_vector;
pub(crate) mod rotate_float_point_about_y;
pub(crate) mod rotate_point_about_y;
pub(crate) mod rotate_point_about_y_ebx;
pub(crate) mod rotate_point_about_y_ecx;
mod rotate_scratch_about_y;
pub(crate) mod seed_random;
pub(crate) mod stir_random;
pub(crate) mod x87;
#[cfg(test)]
mod x87_checks;
pub(crate) mod x87_float;
pub(crate) mod x87_mem;
