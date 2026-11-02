//! The hook and self-test machinery for functions that only change game variables. Shadow
//! mode cannot simply run such a function twice, so it runs the Rust version, notes what it
//! left, puts the variables back, runs the original and compares.

/// A piece of game memory a function may change: (address, bytes).
pub type Region = (u32, usize);
