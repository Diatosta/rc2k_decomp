//! The 3D primary and secondary card switches first values: the bytes of RAL.EXE's 0x5AB4AC .. 0x5AB4B4 that are not zero (generated from the image; `cards::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
