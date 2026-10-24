//! The marks' hexagon batch drawn by d3d (the index and vertex counts, the fans, the records) first values: the bytes of RAL.EXE's 0x62423C .. 0x628A44 that are not zero (generated from the image; `hexagon_batch::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
